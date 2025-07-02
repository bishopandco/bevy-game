use bevy::prelude::*;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio::runtime::Runtime;
use futures_util::StreamExt;
use tokio_tungstenite::connect_async;
use rustls::crypto::CryptoProvider;

use crate::globals::GameParams;

/// Resource holding the Tokio runtime and connection status.
#[derive(Resource)]
pub struct SocketClient {
    runtime: Runtime,
    connected: Arc<AtomicBool>,
}

impl Default for SocketClient {
    fn default() -> Self {
        CryptoProvider::install_default().expect("failed to install CryptoProvider");
        Self {
            runtime: Runtime::new().expect("failed to create Tokio runtime"),
            connected: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl SocketClient {
    /// Returns true if the client is currently connected.
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
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

fn connect_socket(client: Res<SocketClient>, params: Res<GameParams>) {
    let url = params.socketUrl.clone();
    let status = client.connected.clone();
    client.runtime.spawn(async move {
        match connect_async(&url).await {
            Ok((mut ws, _)) => {
                status.store(true, Ordering::SeqCst);
                while let Some(msg) = ws.next().await {
                    if msg.is_err() {
                        break;
                    }
                }
                status.store(false, Ordering::SeqCst);
            }
            Err(e) => {
                eprintln!("Socket connection error: {e}");
            }
        }
    });
}

