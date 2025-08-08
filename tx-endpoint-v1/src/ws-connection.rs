use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebSocket, MessageEvent, CloseEvent, ErrorEvent};
use crate::{Transaction, SignalingMessage};

pub struct WebSocketConnection {
    ws: Option<WebSocket>,
    endpoint_id: String,
    message_handler: Option<Box<dyn Fn(SignalingMessage)>>,
}

impl WebSocketConnection {
    pub fn new() -> Self {
        Self {
            ws: None,
            endpoint_id: String::new(),
            message_handler: None,
        }
    }

    pub fn connect(
        &mut self,
        endpoint_id: &str,
        message_handler: Box<dyn Fn(SignalingMessage)>,
    ) -> Result<(), JsValue> {
        self.endpoint_id = endpoint_id.to_string();
        self.message_handler = Some(message_handler);

        let signaling_url = std::env::var("SIGNALING_SERVER")
            .unwrap_or_else(|_| "ws://localhost:8080".to_string());

        web_sys::console::log_1(&format!("Connecting to {}", signaling_url).into());

        let ws = WebSocket::new(&signaling_url)?;
        
        // Set up message handler
        let message_handler_clone = self.message_handler.as_ref().unwrap();
        let onmessage_callback = {
            let handler = unsafe {
                std::mem::transmute::<&dyn Fn(SignalingMessage), &'static dyn Fn(SignalingMessage)>(
                    message_handler_clone.as_ref()
                )
            };
            
            Closure::wrap(Box::new(move |e: MessageEvent| {
                if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                    let message_str: String = txt.into();
                    web_sys::console::log_1(&format!("Received: {}", message_str).into());
                    
                    if let Ok(msg) = serde_json::from_str::<SignalingMessage>(&message_str) {
                        handler(msg);
                    } else {
                        web_sys::console::error_1(&"Failed to parse message".into());
                    }
                }
            }) as Box<dyn FnMut(_)>)
        };
        
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        // Set up open handler
        let endpoint_id_clone = self.endpoint_id.clone();
        let onopen_callback = Closure::wrap(Box::new(move |_| {
            web_sys::console::log_1(&"WebSocket connected".into());
            
            // Join the transaction room
            let join_message = SignalingMessage {
                message_type: "join".to_string(),
                room_id: Some("transaction-room".to_string()),
                peer_id: Some(endpoint_id_clone.clone()),
                target_peer: None,
                transaction: None,
                peers: None,
            };

            if let Ok(msg_str) = serde_json::to_string(&join_message) {
                // We need access to ws here, but it's moved
                web_sys::console::log_1(&format!("Would send: {}", msg_str).into());
            }
        }) as Box<dyn FnMut(_)>);
        
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        
        // Store websocket reference for sending join message
        let ws_for_join = ws.clone();
        let endpoint_id_for_join = self.endpoint_id.clone();
        
        // Set timeout to send join message after connection opens
        let join_callback = Closure::wrap(Box::new(move || {
            let join_message = serde_json::json!({
                "type": "join",
                "roomId": "transaction-room",
                "peerId": endpoint_id_for_join
            });
            
            if let Ok(msg_str) = serde_json::to_string(&join_message) {
                let _ = ws_for_join.send_with_str(&msg_str);
                web_sys::console::log_1(&"Sent join message".into());
            }
        }) as Box<dyn FnMut()>);
        
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                join_callback.as_ref().unchecked_ref(),
                100,
            )?;
        join_callback.forget();
        onopen_callback.forget();

        // Set up close handler
        let onclose_callback = Closure::wrap(Box::new(move |e: CloseEvent| {
            web_sys::console::log_1(&format!("WebSocket closed: {}", e.code()).into());
        }) as Box<dyn FnMut(_)>);
        
        ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();

        // Set up error handler
        let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
            web_sys::console::error_1(&format!("WebSocket error: {:?}", e).into());
        }) as Box<dyn FnMut(_)>);
        
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        self.ws = Some(ws);
        Ok(())
    }

    pub fn send_transaction(&mut self, tx: &Transaction) -> Result<(), JsValue> {
        if let Some(ws) = &self.ws {
            let message = SignalingMessage {
                message_type: "transaction".to_string(),
                room_id: Some("transaction-room".to_string()),
                peer_id: Some(self.endpoint_id.clone()),
                target_peer: None,
                transaction: Some(tx.clone()),
                peers: None,
            };

            let message_str = serde_json::to_string(&message)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
            
            ws.send_with_str(&message_str)?;
            web_sys::console::log_1(&format!("Sent transaction: {}", tx.id).into());
        }
        Ok(())
    }
}
