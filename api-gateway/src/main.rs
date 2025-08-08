use axum::{
    extract::{Query, State},
    http::{StatusCode, Method},
    response::Json,
    routing::{get, post},
    Router,
};
use scylla::{Session, SessionBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_http::cors::{CorsLayer, Any};
use tracing::{info, error};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub from_endpoint: String,
    pub to_endpoint: String,
    pub amount: f64,
    pub timestamp: i64,
    pub signature: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionStats {
    pub total_transactions: i64,
    pub total_volume: f64,
    pub average_transaction: f64,
    pub endpoints: Vec<EndpointStats>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EndpointStats {
    pub endpoint_id: String,
    pub transaction_count: i64,
    pub total_sent: f64,
    pub total_received: f64,
    pub balance_change: f64,
}

#[derive(Clone)]
pub struct AppState {
    session: Session,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("api_gateway=debug,info")
        .init();

    info!("Starting API Gateway...");

    // Connect to ScyllaDB with retry logic
    let session = connect_to_scylla().await?;
    
    // Initialize database schema
    init_database(&session).await?;

    let state = AppState { session };

    // Build our application with routes
    let app = Router::new()
        .route("/api/transactions", get(get_transactions))
        .route("/api/transactions", post(create_transaction))
        .route("/api/transactions/:id", get(get_transaction_by_id))
        .route("/api/stats", get(get_stats))
        .route("/api/endpoints/:id/stats", get(get_endpoint_stats))
        .route("/health", get(health_check))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers(Any)
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    info!("🚀 API Gateway running on http://0.0.0.0:3001");
    
    axum::serve(listener, app).await?;
    Ok(())
}

async fn connect_to_scylla() -> Result<Session, Box<dyn std::error::Error>> {
    let scylla_host = std::env::var("SCYLLA_HOST").unwrap_or_else(|_| "127.0.0.1:9042".to_string());
    info!("Connecting to ScyllaDB at {}", scylla_host);

    let session = SessionBuilder::new()
        .known_node(&scylla_host)
        .build()
        .await?;

    info!("✅ Connected to ScyllaDB");
    Ok(session)
}

async fn init_database(session: &Session) -> Result<(), Box<dyn std::error::Error>> {
    info!("Initializing database schema...");

    // Create keyspace
    session
        .query(
            "CREATE KEYSPACE IF NOT EXISTS transactions 
             WITH REPLICATION = {
                 'class': 'SimpleStrategy',
                 'replication_factor': 1
             }",
            &[],
        )
        .await?;

    // Create transactions table
    session
        .query(
            "CREATE TABLE IF NOT EXISTS transactions.tx_log (
                 id UUID PRIMARY KEY,
                 from_endpoint TEXT,
                 to_endpoint TEXT,
                 amount DOUBLE,
                 timestamp BIGINT,
                 signature TEXT,
                 status TEXT
             )",
            &[],
        )
        .await?;

    // Create index for timestamp-based queries
    session
        .query(
            "CREATE INDEX IF NOT EXISTS tx_timestamp_idx 
             ON transactions.tx_log (timestamp)",
            &[],
        )
        .await?;

    info!("✅ Database schema initialized");
    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "api-gateway"
    }))
}

async fn get_transactions(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<Transaction>>, StatusCode> {
    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<i32>().ok())
        .unwrap_or(100);

    let endpoint = params.get("endpoint");

    let (query, values): (String, Vec<scylla::frame::value::Value>) = if let Some(ep) = endpoint {
        (
            "SELECT id, from_endpoint, to_endpoint, amount, timestamp, signature, status 
             FROM transactions.tx_log WHERE from_endpoint = ? OR to_endpoint = ? LIMIT ? ALLOW FILTERING".to_string(),
            vec![ep.clone().into(), ep.clone().into(), limit.into()]
        )
    } else {
        (
            "SELECT id, from_endpoint, to_endpoint, amount, timestamp, signature, status 
             FROM transactions.tx_log LIMIT ?".to_string(),
            vec![limit.into()]
        )
    };

    let rows = state
        .session
        .query(query, values)
        .await
        .map_err(|e| {
            error!("Database query error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut transactions = Vec::new();
    
    if let Some(rows) = rows.rows {
        for row in rows {
            if let Ok((id, from_endpoint, to_endpoint, amount, timestamp, signature, status)) = 
                row.into_typed::<(Uuid, String, String, f64, i64, String, String)>() {
                transactions.push(Transaction {
                    id: id.to_string(),
                    from_endpoint,
                    to_endpoint,
                    amount,
                    timestamp,
                    signature,
                    status,
                });
            }
        }
    }

    // Sort by timestamp descending (newest first)
    transactions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(Json(transactions))
}

async fn get_transaction_by_id(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Transaction>, StatusCode> {
    let tx_id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let rows = state
        .session
        .query(
            "SELECT id, from_endpoint, to_endpoint, amount, timestamp, signature, status 
             FROM transactions.tx_log WHERE id = ?",
            (tx_id,),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(rows) = rows.rows {
        if let Some(row) = rows.into_iter().next() {
            if let Ok((id, from_endpoint, to_endpoint, amount, timestamp, signature, status)) = 
                row.into_typed::<(Uuid, String, String, f64, i64, String, String)>() {
                return Ok(Json(Transaction {
                    id: id.to_string(),
                    from_endpoint,
                    to_endpoint,
                    amount,
                    timestamp,
                    signature,
                    status,
                }));
            }
        }
    }

    Err(StatusCode::NOT_FOUND)
}

async fn create_transaction(
    State(state): State<AppState>,
    Json(transaction): Json<Transaction>,
) -> Result<StatusCode, StatusCode> {
    let tx_id = Uuid::parse_str(&transaction.id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    state
        .session
        .query(
            "INSERT INTO transactions.tx_log (id, from_endpoint, to_endpoint, amount, timestamp, signature, status)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            (
                tx_id,
                transaction.from_endpoint,
                transaction.to_endpoint,
                transaction.amount,
                transaction.timestamp,
                transaction.signature,
                transaction.status,
            ),
        )
        .await
        .map_err(|e| {
            error!("Failed to insert transaction: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("✅ Transaction {} created", transaction.id);
    Ok(StatusCode::CREATED)
}

async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<TransactionStats>, StatusCode> {
    // Get total transaction count and volume
    let total_rows = state
        .session
        .query(
            "SELECT COUNT(*), SUM(amount) FROM transactions.tx_log",
            &[],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (total_transactions, total_volume) = if let Some(rows) = total_rows.rows {
        if let Some(row) = rows.into_iter().next() {
            if let Ok((count, sum)) = row.into_typed::<(i64, Option<f64>)>() {
                (count, sum.unwrap_or(0.0))
            } else {
                (0, 0.0)
            }
        } else {
            (0, 0.0)
        }
    } else {
        (0, 0.0)
    };

    let average_transaction = if total_transactions > 0 {
        total_volume / total_transactions as f64
    } else {
        0.0
    };

    // Get endpoint statistics
    let endpoint_rows = state
        .session
        .query(
            "SELECT from_endpoint, to_endpoint, amount FROM transactions.tx_log",
            &[],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut endpoint_map: HashMap<String, EndpointStats> = HashMap::new();

    if let Some(rows) = endpoint_rows.rows {
        for row in rows {
            if let Ok((from_endpoint, to_endpoint, amount)) = 
                row.into_typed::<(String, String, f64)>() {
                
                // Update sender stats
                let sender_stats = endpoint_map.entry(from_endpoint.clone()).or_insert(EndpointStats {
                    endpoint_id: from_endpoint.clone(),
                    transaction_count: 0,
                    total_sent: 0.0,
                    total_received: 0.0,
                    balance_change: 0.0,
                });
                sender_stats.transaction_count += 1;
                sender_stats.total_sent += amount;
                sender_stats.balance_change -= amount;

                // Update receiver stats
                let receiver_stats = endpoint_map.entry(to_endpoint.clone()).or_insert(EndpointStats {
                    endpoint_id: to_endpoint.clone(),
                    transaction_count: 0,
                    total_sent: 0.0,
                    total_received: 0.0,
                    balance_change: 0.0,
                });
                receiver_stats.total_received += amount;
                receiver_stats.balance_change += amount;
            }
        }
    }

    let endpoints: Vec<EndpointStats> = endpoint_map.into_values().collect();

    Ok(Json(TransactionStats {
        total_transactions,
        total_volume,
        average_transaction,
        endpoints,
    }))
}

async fn get_endpoint_stats(
    State(state): State<AppState>,
    axum::extract::Path(endpoint_id): axum::extract::Path<String>,
) -> Result<Json<EndpointStats>, StatusCode> {
    let rows = state
        .session
        .query(
            "SELECT from_endpoint, to_endpoint, amount FROM transactions.tx_log 
             WHERE from_endpoint = ? OR to_endpoint = ? ALLOW FILTERING",
            (&endpoint_id, &endpoint_id),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut stats = EndpointStats {
        endpoint_id: endpoint_id.clone(),
        transaction_count: 0,
        total_sent: 0.0,
        total_received: 0.0,
        balance_change: 0.0,
    };

    if let Some(rows) = rows.rows {
        for row in rows {
            if let Ok((from_endpoint, to_endpoint, amount)) = 
                row.into_typed::<(String, String, f64)>() {
                
                stats.transaction_count += 1;
                
                if from_endpoint == endpoint_id {
                    stats.total_sent += amount;
                    stats.balance_change -= amount;
                }
                
                if to_endpoint == endpoint_id {
                    stats.total_received += amount;
                    stats.balance_change += amount;
                }
            }
        }
    }

    Ok(Json(stats))
}
