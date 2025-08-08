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
├── ws-tx-endpoint/               # Rust Dioxus WASM app
│   ├── Cargo.toml
│   ├── Dockerfile
│   └── src/
│       ├── lib.rs
│       ├── tx-endpoint.rs
│       └── p2p-connection.rs
├── tx-status/                 # React.js, D3.js dashboard
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
mkdir -p ws-tx-endpoint/src
mkdir -p wrtc-tx-endpoint/src
mkdir -p tx-status
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

```shell
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


## Create Transaction (Tx) Endpoint Project (Rust Dioxus, WebSockets)

This uses WebSockets with required Signaling Server (SS) to have peers join a room and initiate the joint
connections on initialization. For P2P services using only WebSockets the Signaling Server is required. For true P2P without the Signaling Server, version 2 (included in the project repo as `tx-endpoint-v2`) uses WebRTC. WebRTS offers channel streams and does NOT need a initiating Signaling Server to group join the peers.

```shell
cd ../ws-tx-endpoint
cargo init --name ws-x-endpoint --lib
```

The `Cargo.toml` for the ws-tx-endpoint project is as shown.

```shell
[package]
name = "tx-endpoint-v1"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
dioxus = "0.4"
dioxus-web = "0.4"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
web-sys = { version = "0.3", features = [
  "console",
  "WebSocket",
  "MessageEvent",
  "CloseEvent",
  "ErrorEvent",
  "Location",
  "Window",
  "Document",
  "Element",
  "HtmlElement",
] }
uuid = { version = "1.0", features = ["v4", "js"] }
gloo-timers = { version = "0.3", features = ["futures"] }
futures = "0.3"

[dependencies.web-sys]
version = "0.3"
features = [
  "console",
  "WebSocket",
  "MessageEvent",
  "CloseEvent",
  "ErrorEvent",
  "BinaryType",
]
```


## Create React.js D3.js Transaction Events Status Dash 

```shell
cd ../tx-status
npm create vite@latest . -- --template react
npm install
npm install d3 axios
npm install -D @types/d3
```

The React.js `dash` project.json file is as shown.

```json
{
  "name": "p2p-tx-status",
  "private": true,
  "version": "0.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "lint": "eslint . --ext js,jsx --report-unused-disable-directives --max-warnings 0",
    "preview": "vite preview",
    "start": "vite"
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "d3": "^7.8.5",
    "axios": "^1.6.0"
  },
  "devDependencies": {
    "@types/react": "^18.2.37",
    "@types/react-dom": "^18.2.15",
    "@vitejs/plugin-react": "^4.1.0",
    "eslint": "^8.53.0",
    "eslint-plugin-react": "^7.33.2",
    "eslint-plugin-react-hooks": "^4.6.0",
    "eslint-plugin-react-refresh": "^0.4.4",
    "vite": "^4.5.0",
    "@types/d3": "^7.4.3"
  }
}
```


## Create Transaction (Tx) Endpoint Project (Rust Dioxus, WebRTC)

```shell
cd ../wrtc-tx-endpoint
cargo init --name wrtc-tx-endpoint --lib
```

The cargo.toml for the `wrtc-tx-endpoint` is as shown.

```shell
[package]
name = "wrtc-tx-endpoint"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[features]
default = ["webrtc"]
webrtc = []

[dependencies]
dioxus = "0.4"
dioxus-web = "0.4"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
web-sys = { version = "0.3", features = [
  "console",
  "WebSocket",
  "MessageEvent",
  "CloseEvent",
  "ErrorEvent",
  "RtcPeerConnection",
  "RtcConfiguration",
  "RtcDataChannel",
  "RtcDataChannelEvent",
  "RtcPeerConnectionIceEvent",
  "RtcSessionDescription",
  "RtcSessionDescriptionInit",
  "RtcIceCandidate",
  "RtcIceCandidateInit",
  "RtcDataChannelInit",
  "RtcDataChannelState",
  "Location",
  "Window",
] }
uuid = { version = "1.0", features = ["v4", "js"] }
gloo-timers = { version = "0.3", features = ["futures"] }
futures = "0.3"
```




