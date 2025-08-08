use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use wasm_bindgen::prelude::*;

mod tx_endpoint;
mod websocket_connection;

use tx_endpoint::TxEndpoint;
use websocket_connection::WebSocketConnection;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub timestamp: u64,
    pub signature: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignalingMessage {
    pub message_type: String,
    pub room_id: Option<String>,
    pub peer_id: Option<String>,
    pub target_peer: Option<String>,
    pub transaction: Option<Transaction>,
    pub peers: Option<Vec<String>>,
}

fn main() {
    console_error_panic_hook::set_once();
    dioxus_web::launch(app);
}

fn app(cx: Scope) -> Element {
    // Get endpoint ID from URL or default
    let endpoint_id = use_state(cx, || {
        web_sys::window()
            .and_then(|w| w.location().search().ok())
            .and_then(|search| {
                if search.starts_with("?id=") {
                    Some(search[4..].to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "endpoint-1".to_string())
    });

    let tx_endpoint = use_state(cx, || TxEndpoint::new(&endpoint_id.get()));
    let connection = use_state(cx, || WebSocketConnection::new());
    let transactions = use_state(cx, HashMap::<String, Transaction>::new);
    let connected_peers = use_state(cx, Vec::<String>::new);
    let connection_status = use_state(cx, || "Disconnected".to_string());
    let error_message = use_state(cx, || "".to_string());

    // Auto-connect on component mount
    use_effect(cx, (), {
        let connection = connection.clone();
        let endpoint_id = endpoint_id.get().clone();
        let connection_status = connection_status.clone();
        let connected_peers = connected_peers.clone();
        let transactions = transactions.clone();
        let error_message = error_message.clone();
        
        move |_| {
            async move {
                web_sys::console::log_1(&"Initializing connection...".into());
                
                let result = connection.with_mut(|conn| {
                    conn.connect(
                        &endpoint_id,
                        Box::new({
                            let connection_status = connection_status.clone();
                            let connected_peers = connected_peers.clone();
                            let transactions = transactions.clone();
                            let error_message = error_message.clone();
                            
                            move |msg: SignalingMessage| {
                                handle_signaling_message(
                                    msg,
                                    &connection_status,
                                    &connected_peers,
                                    &transactions,
                                    &error_message,
                                );
                            }
                        }),
                    )
                });

                if let Err(e) = result {
                    error_message.set(format!("Connection failed: {:?}", e));
                }
            }
        }
    });

    render! {
        div {
            class: "tx-endpoint-container",
            style: "padding: 20px; max-width: 1000px; margin: 0 auto; font-family: 'Segoe UI', system-ui, sans-serif;",
            
            header {
                style: "background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 20px; border-radius: 12px; margin-bottom: 20px; text-align: center;",
                h1 { 
                    style: "margin: 0; font-size: 2rem;",
                    "Transaction Endpoint: {endpoint_id}" 
                }
                p {
                    style: "margin: 10px 0 0 0; opacity: 0.9;",
                    "WebSocket P2P Version"
                }
            }
            
            // Error display
            if !error_message.is_empty() {
                div {
                    style: "background: #fee; border: 1px solid #fcc; color: #c33; padding: 10px; border-radius: 8px; margin-bottom: 20px;",
                    "{error_message}"
                    button {
                        style: "float: right; background: none; border: none; color: #c33; cursor: pointer;",
                        onclick: move |_| error_message.set("".to_string()),
                        "×"
                    }
                }
            }
            
            div {
                style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin-bottom: 20px;",
                
                // Connection Status Panel
                div {
                    class: "status-panel",
                    style: "background: #f8f9fa; border: 1px solid #dee2e6; padding: 20px; border-radius: 12px;",
                    
                    h3 { 
                        style: "margin-top: 0; color: #495057;",
                        "Connection Status" 
                    }
                    
                    div {
                        style: "display: flex; align-items: center; margin-bottom: 15px;",
                        div {
                            style: format!(
                                "width: 12px; height: 12px; border-radius: 50%; margin-right: 10px; background: {};",
                                if connection_status.get() == "Connected" { "#28a745" } else { "#dc3545" }
                            ),
                        }
                        span {
                            style: "font-weight: 600;",
                            "{connection_status}"
                        }
                    }
                    
                    p { 
                        style: "margin: 5px 0; color: #6c757d;",
                        "Connected Peers: {connected_peers.len()}" 
                    }
                    
                    if !connected_peers.is_empty() {
                        ul {
                            style: "margin: 10px 0; padding-left: 20px; color: #495057;",
                            connected_peers.iter().map(|peer| render! {
                                li { 
                                    key: "{peer}",
                                    style: "margin: 5px 0;",
                                    "👤 {peer}"
                                }
                            })
                        }
                    }
                }
                
                // Endpoint Info Panel
                div {
                    class: "endpoint-info",
                    style: "background: #e3f2fd; border: 1px solid #bbdefb; padding: 20px; border-radius: 12px;",
                    
                    h3 { 
                        style: "margin-top: 0; color: #1565c0;",
                        "Endpoint Info" 
                    }
                    p { 
                        style: "margin: 5px 0; font-size: 1.2rem; font-weight: 600; color: #1976d2;",
                        "Balance: ${tx_endpoint.balance:.2}" 
                    }
                    p { 
                        style: "margin: 5px 0; color: #1565c0;",
                        "Total Transactions: {tx_endpoint.transaction_count}" 
                    }
                }
            }
            
            // Transaction Controls
            div {
                class: "transaction-controls",
                style: "background: linear-gradient(135deg, #4caf50 0%, #45a049 100%); color: white; padding: 20px; border-radius: 12px; margin-bottom: 20px;",
                
                h3 { 
                    style: "margin-top: 0;",
                    "Send Transaction" 
                }
                
                div {
                    style: "display: flex; gap: 10px; align-items: center; flex-wrap: wrap;",
                    
                    select {
                        style: "padding: 10px; border: none; border-radius: 6px; font-size: 1rem;",
                        option { value: "", "Select Peer" }
                        connected_peers.iter().map(|peer| render! {
                            option { 
                                key: "{peer}",
                                value: "{peer}",
                                "{peer}"
                            }
                        })
                    }
                    
                    input {
                        r#type: "number",
                        placeholder: "Amount",
                        step: "0.01",
                        min: "0.01",
                        style: "padding: 10px; border: none; border-radius: 6px; font-size: 1rem; width: 120px;",
                    }
                    
                    button {
                        style: "background: rgba(255,255,255,0.2); color: white; border: 1px solid rgba(255,255,255,0.3); padding: 10px 20px; border-radius: 6px; cursor: pointer; font-size: 1rem; font-weight: 600;",
                        onclick: move |event| {
                            if let Some(form) = event.target().and_then(|t| t.closest("div")) {
                                if let Ok(form_elem) = form.dyn_into::<web_sys::HtmlElement>() {
                                    let select = form_elem.query_selector("select").unwrap().unwrap();
                                    let input = form_elem.query_selector("input").unwrap().unwrap();
                                    
                                    let select_elem = select.dyn_into::<web_sys::HtmlSelectElement>().unwrap();
                                    let input_elem = input.dyn_into::<web_sys::HtmlInputElement>().unwrap();
                                    
                                    let to_peer = select_elem.value();
                                    let amount_str = input_elem.value();
                                    
                                    if !to_peer.is_empty() && !amount_str.is_empty() {
                                        if let Ok(amount) = amount_str.parse::<f64>() {
                                            if amount > 0.0 && amount <= tx_endpoint.balance {
                                                let tx = Transaction {
                                                    id: Uuid::new_v4().to_string(),
                                                    from: endpoint_id.get().clone(),
                                                    to: to_peer,
                                                    amount,
                                                    timestamp: js_sys::Date::now() as u64,
                                                    signature: format!("sig_{}", tx_endpoint.transaction_count),
                                                    status: "pending".to_string(),
                                                };
                                                
                                                // Update local endpoint state
                                                tx_endpoint.with_mut(|ep| {
                                                    let _ = ep.process_transaction(&tx);
                                                });
                                                
                                                // Add to local transactions
                                                transactions.with_mut(|txs| {
                                                    txs.insert(tx.id.clone(), tx.clone());
                                                });
                                                
                                                // Send via WebSocket
                                                connection.with_mut(|conn| {
                                                    if let Err(e) = conn.send_transaction(&tx) {
                                                        error_message.set(format!("Failed to send transaction: {:?}", e));
                                                    }
                                                });
                                                
                                                // Clear form
                                                select_elem.set_value("");
                                                input_elem.set_value("");
                                            } else {
                                                error_message.set("Invalid amount or insufficient balance".to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        "Send Transaction"
                    }
                    
                    button {
                        style: "background: rgba(255,255,255,0.2); color: white; border: 1px solid rgba(255,255,255,0.3); padding: 10px 20px; border-radius: 6px; cursor: pointer; font-size: 1rem;",
                        onclick: move |_| {
                            if !connected_peers.is_empty() {
                                let random_peer = &connected_peers[0]; // Use first peer for demo
                                let tx = Transaction {
                                    id: Uuid::new_v4().to_string(),
                                    from: endpoint_id.get().clone(),
                                    to: random_peer.clone(),
                                    amount: 10.0,
                                    timestamp: js_sys::Date::now() as u64,
                                    signature: format!("sig_{}", tx_endpoint.transaction_count),
                                    status: "pending".to_string(),
                                };
                                
                                tx_endpoint.with_mut(|ep| {
                                    let _ = ep.process_transaction(&tx);
                                });
                                
                                transactions.with_mut(|txs| {
                                    txs.insert(tx.id.clone(), tx.clone());
                                });
                                
                                connection.with_mut(|conn| {
                                    let _ = conn.send_transaction(&tx);
                                });
                            }
                        },
                        "Send Test $10"
                    }
                }
            }
            
            // Transaction Log
            div {
                class: "transaction-log",
                style: "background: white; border: 1px solid #dee2e6; border-radius: 12px; padding: 20px;",
                
                h3 { 
                    style: "margin-top: 0; color: #495057;",
                    "Transaction Log ({transactions.len()})" 
                }
                
                div {
                    style: "max-height: 400px; overflow-y: auto;",
                    
                    if transactions.is_empty() {
                        div {
                            style: "text-align: center; color: #6c757d; padding: 40px;",
                            "No transactions yet. Send one to get started!"
                        }
                    } else {
                        transactions.iter().rev().take(10).map(|(id, tx)| render! {
                            div {
                                key: "{id}",
                                style: format!(
                                    "border-left: 4px solid {}; background: #f8f9fa; margin: 10px 0; padding: 15px; border-radius: 0 8px 8px 0;",
                                    if tx.from == *endpoint_id.get() { "#dc3545" } else { "#28a745" }
                                ),
                                
                                div {
                                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;",
                                    strong {
                                        style: "color: #495057;",
                                        if tx.from == *endpoint_id.get() { "📤 Sent" } else { "📥 Received" }
                                    }
                                    span {
                                        style: format!(
                                            "background: {}; color: white; padding: 2px 8px; border-radius: 12px; font-size: 0.8rem;",
                                            match tx.status.as_str() {
                                                "confirmed" => "#28a745",
                                                "pending" => "#ffc107",
                                                "failed" => "#dc3545",
                                                _ => "#6c757d"
                                            }
                                        ),
                                        "{tx.status}"
                                    }
                                }
                                
                                p { 
                                    style: "margin: 5px 0; color: #495057;",
                                    "Amount: ${tx.amount:.2}" 
                                }
                                p { 
                                    style: "margin: 5px 0; color: #6c757d; font-size: 0.9rem;",
                                    "{tx.from} → {tx.to}" 
                                }
                                p { 
                                    style: "margin: 5px 0; color: #6c757d; font-size: 0.8rem;",
                                    "{format_timestamp(tx.timestamp)}"
                                }
                                p { 
                                    style: "margin: 5px 0; color: #6c757d; font-size: 0.8rem; font-family: monospace;",
                                    "{tx.id[..8]}..."
                                }
                            }
                        })
                    }
                }
            }
        }
    }
}

fn handle_signaling_message(
    msg: SignalingMessage,
    connection_status: &UseState<String>,
    connected_peers: &UseState<Vec<String>>,
    transactions: &UseState<HashMap<String, Transaction>>,
    error_message: &UseState<String>,
) {
    web_sys::console::log_1(&format!("Handling message: {:?}", msg.message_type).into());
    
    match msg.message_type.as_str() {
        "welcome" => {
            connection_status.set("Connected".to_string());
        },
        "room-joined" => {
            connection_status.set("Connected".to_string());
            if let Some(peers) = msg.peers {
                connected_peers.set(peers);
            }
        },
        "peer-joined" => {
            if let Some(peer_id) = msg.peer_id {
                connected_peers.with_mut(|peers| {
                    if !peers.contains(&peer_id) {
                        peers.push(peer_id);
                    }
                });
            }
        },
        "peer-left" => {
            if let Some(peer_id) = msg.peer_id {
                connected_peers.with_mut(|peers| {
                    peers.retain(|p| p != &peer_id);
                });
            }
        },
        "transaction-broadcast" => {
            if let Some(tx) = msg.transaction {
                transactions.with_mut(|txs| {
                    txs.insert(tx.id.clone(), tx);
                });
            }
        },
        "error" => {
            error_message.set("Connection error occurred".to_string());
        },
        _ => {
            web_sys::console::log_1(&format!("Unknown message type: {}", msg.message_type).into());
        }
    }
}

fn format_timestamp(timestamp: u64) -> String {
    let date = js_sys::Date::new(&(timestamp.into()));
    date.to_locale_string("en-US", &js_sys::Object::new()).as_string().unwrap_or_default()
}

