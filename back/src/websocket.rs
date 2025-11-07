use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
    Extension,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WsMessage {
    BalanceUpdate {
        user: String,
        balance: u64,
    },
    DepositNotification {
        user: String,
        amount: u64,
        signature: String,
    },
    WithdrawNotification {
        user: String,
        amount: u64,
        signature: String,
    },
    TvlUpdate {
        tvl: u64,
    },
}

pub struct WebSocketManager {
    tx: broadcast::Sender<String>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub fn broadcast(&self, message: WsMessage) {
        let json = serde_json::to_string(&message).unwrap();
        let _ = self.tx.send(json);
    }

    pub async fn handle_socket(self: Arc<Self>, socket: WebSocket) {
        let (mut sender, mut receiver) = socket.split();
        let mut rx = self.tx.subscribe();

        // Send messages to client
        let send_task = tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if sender.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }
        });

        // Receive messages from client (for ping/pong)
        let recv_task = tokio::spawn(async move {
            while let Some(Ok(msg)) = receiver.next().await {
                if let Message::Close(_) = msg {
                    break;
                }
            }
        });

        // Wait for either task to finish
        tokio::select! {
            _ = send_task => {},
            _ = recv_task => {},
        }
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(ws_manager): Extension<Arc<WebSocketManager>>,
) -> Response {
    ws.on_upgrade(move |socket| ws_manager.handle_socket(socket))
}
