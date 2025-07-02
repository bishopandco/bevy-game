use bevy::prelude::*;
use bevy::log::info;
use serde_json::json;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use tokio::runtime::Runtime;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use rustls::crypto::{ring::default_provider as ring_provider, CryptoProvider};
use tokio_tungstenite::tungstenite::Utf8Bytes;
use crate::globals::GameParams;

/// Resource holding the Tokio runtime and connection status.
#[derive(Resource)]
pub struct SocketClient {
    runtime: Runtime,
    connected: Arc<AtomicBool>,
    sender: Option<UnboundedSender<String>>,
    receiver: Option<UnboundedReceiver<String>>,
}

impl Default for SocketClient {
    fn default() -> Self {
        ring_provider()
            .install_default()
            .expect("failed to install CryptoProvider");
        Self {
            runtime: Runtime::new().expect("failed to create Tokio runtime"),
            connected: Arc::new(AtomicBool::new(false)),
            sender: None,
            receiver: None,
        }
    }
}

impl SocketClient {
    /// Returns true if the client is currently connected.
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    /// Sends a chat message over the socket using the `sendMessage` action.
    pub fn send(&self, text: String) {
        if !self.is_connected() {
            info!("WebSocket is not open");
            return;
        }

        if let Some(tx) = &self.sender {
            let payload = json!({
                "action": "sendMessage",
                "data": text,
            })
            .to_string();

            info!("Queueing outgoing message: {}", payload);
            let _ = tx.send(payload);
        }
    }

    /// Sends a raw json payload.
    pub fn send_json(&self, value: serde_json::Value) {
        if !self.is_connected() {
            info!("WebSocket is not open");
            return;
        }

        if let Some(tx) = &self.sender {
            let payload = value.to_string();
            info!("Queueing outgoing message: {}", payload);
            let _ = tx.send(payload);
        }
    }

    /// Attempts to receive a text message from the socket.
    pub fn try_recv(&mut self) -> Option<String> {
        if let Some(rx) = &mut self.receiver {
            match rx.try_recv() {
                Ok(msg) => {
                    info!("Incoming message: {}", msg);
                    Some(msg)
                },
                Err(_) => None,
            }
        } else {
            None
        }
    }
}

/// Plugin that establishes a WebSocket connection on startup.
pub struct SocketClientPlugin;

impl Plugin for SocketClientPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SocketClient>()
            .add_systems(Startup, connect_socket);
    }
}

fn connect_socket(mut client: ResMut<SocketClient>, params: Res<GameParams>) {
    let url = params.socket_url.clone();
    let status = client.connected.clone();

    let (tx_in, mut rx_in) = unbounded_channel::<String>();
    let (tx_out, rx_out) = unbounded_channel::<String>();

    client.sender = Some(tx_in);
    client.receiver = Some(rx_out);

    client.runtime.spawn(async move {
        match connect_async(&url).await {
            Ok((ws, _)) => {
                info!("Socket connected to {}", url);
                status.store(true, Ordering::SeqCst);
                let (mut write, mut read) = ws.split();

                let send_task = tokio::spawn(async move {
                    while let Some(msg) = rx_in.recv().await {
                        if write.send(Message::Text(Utf8Bytes::from(msg))).await.is_err() {
                            break;
                        }
                    }
                });

                let recv_task = tokio::spawn(async move {
                    while let Some(Ok(msg)) = read.next().await {
                        if let Ok(text) = msg.into_text() {
                            let _ = tx_out.send(text.to_string());
                        }
                    }
                });

                let _ = tokio::join!(send_task, recv_task);
                status.store(false, Ordering::SeqCst);
                info!("Socket disconnected");
            }
            Err(e) => {
                eprintln!("Socket connection error: {e}");
            }
        }
    });
}

