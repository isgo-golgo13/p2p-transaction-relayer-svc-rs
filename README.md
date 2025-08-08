# P2P Transaction Service (Rust, Rust Dioxus WASM)
P2P Transaction Relayer Service using Rust, TLS WebSockets, Rust Dioxus, React.js and D3.js and ScyllaDB


## Project Structure
```shell
p2p-transaction-relayer-svc-rs/
├── docker-compose.yml
├── ws-signaling-server/          # Node.js WebSocket signaling server
│   ├── Dockerfile
│   ├── package.json
│   └── src/
│       └── server.js
├── tx-endpoint/               # Rust Dioxus WASM app
│   ├── Cargo.toml
│   ├── Dockerfile
│   └── src/
│       ├── lib.rs
│       ├── tx_endpoint.rs
│       └── p2p_connection.rs
├── dashboard/                 # React.js, D3.js dashboard
│   ├── Dockerfile
│   ├── package.json
│   └── src/
│       ├── App.js
│       ├── TxEndpointCard.js
│       └── TransactionGraph.js
├── api-gateway/              # REST API for ScyllaDB
│   ├── Cargo.toml
│   ├── Dockerfile
│   └── src/
│       └── main.rs
└── scripts/
    ├── setup.sh
    ├── run-v1.sh
    └── run-v2.sh
```


## Project Creation

```shell
# Create the main project directory
mkdir p2p-transaction-relayer-svc
cd p2p-transaction-relayer-svc

# Create all subdirectories
mkdir -p ws-signaling-server/src
mkdir -p tx-endpoint-v1/src
mkdir -p tx-endpoint-v2/src
mkdir -p dashboard
mkdir -p api-gateway/src
mkdir -p scripts
```


## Create WebSockets Signaling Server Project (JavaScript Node.js)

```shell
cd ws-signaling-server
npm init -y
npm install ws express cors
```


## Create API Gateway Project (Rust)

```shell
cd ../api-gateway
cargo init --name api-gateway
```

The `Cargo.toml` for the api-gateway project is as shown.

```toml
[package]
name = "api-gateway"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
axum = "0.7"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
scylla = "0.12"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
anyhow = "1.0"
```






