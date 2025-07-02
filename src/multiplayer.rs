use bevy::prelude::*;
use serde::Deserialize;
use serde_json::json;

use crate::{socket_client::SocketClient, input::Player};

#[derive(Component)]
pub struct RemotePlayer {
    pub id: String,
}

#[derive(Resource, Default)]
pub struct PlayerMap(pub std::collections::HashMap<String, Entity>);

#[derive(Deserialize)]
#[serde(tag = "action")]
enum IncomingMessage {
    #[serde(rename = "playerJoin")]
    PlayerJoin { id: String },
    #[serde(rename = "playerMove")]
    PlayerMove { id: String, position: [f32; 3] },
    #[serde(rename = "sendMessage")]
    Chat { data: String },
}

pub struct MultiplayerPlugin;

impl Plugin for MultiplayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerMap>()
            .add_event::<ChatMessage>()
            .add_systems(Update, (
                process_messages,
                broadcast_position,
            ));
    }
}

#[derive(Event)]
pub struct ChatMessage(pub String);

fn process_messages(
    mut client: ResMut<SocketClient>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut players: ResMut<PlayerMap>,
    mut chat: EventWriter<ChatMessage>,
    mut query: Query<&mut Transform, With<RemotePlayer>>,
) {
    while let Some(msg) = client.try_recv() {
        if let Ok(parsed) = serde_json::from_str::<IncomingMessage>(&msg) {
            match parsed {
                IncomingMessage::PlayerJoin { id } => {
                    if players.0.contains_key(&id) {
                        continue;
                    }
                    let mesh = meshes.add(Cuboid::new(0.25, 0.25, 0.25));
                    let entity = commands
                        .spawn((
                            Mesh3d(mesh),
                            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
                            Transform::default(),
                            GlobalTransform::default(),
                            RemotePlayer { id: id.clone() },
                        ))
                        .id();
                    players.0.insert(id, entity);
                }
                IncomingMessage::PlayerMove { id, position } => {
                    if let Some(&entity) = players.0.get(&id) {
                        if let Ok(mut tf) = query.get_mut(entity) {
                            tf.translation = Vec3::new(position[0], position[1], position[2]);
                        }
                    }
                }
                IncomingMessage::Chat { data } => {
                    chat.send(ChatMessage(data));
                }
            }
        }
    }
}

fn broadcast_position(
    client: Res<SocketClient>,
    q: Query<&Transform, (With<Player>, Without<RemotePlayer>)>,
) {
    if !client.is_connected() {
        return;
    }

    if let Ok(tf) = q.get_single() {
        let pos = tf.translation;
        client.send_json(json!({
            "action": "playerMove",
            "position": [pos.x, pos.y, pos.z],
        }));
    }
}
