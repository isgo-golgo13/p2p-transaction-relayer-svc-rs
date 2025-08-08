# P2P Transaction Service (Rust, Rust Dioxus WASM)
P2P Transaction Relayer Service using Rust, TLS WebSockets, Rust Dioxus, React.js and D3.js and ScyllaDB


## Project Structure
```shell
p2p-transaction-system/
├── docker-compose.yml
├── signaling-server/          # Node.js WebSocket signaling server
│   ├── Dockerfile
│   ├── package.json
│   └── src/
│       └── server.js
├── tx-endpoint/               # Rust/Dioxus WASM app
│   ├── Cargo.toml
│   ├── Dockerfile
│   └── src/
│       ├── lib.rs
│       ├── tx_endpoint.rs
│       └── p2p_connection.rs
├── dashboard/                 # React + D3.js dashboard
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

