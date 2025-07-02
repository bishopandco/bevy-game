use bevy::prelude::*;
use bevy::log::info;
use bevy_egui::{egui, EguiContexts};

use crate::multiplayer::ChatMessage;
use crate::socket_client::SocketClient;

#[derive(Resource, Default)]
pub struct ChatLog {
    pub messages: Vec<String>,
    pub input: String,
}

pub struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChatLog>()
            .add_systems(Update, receive_messages)
            .add_systems(bevy_egui::EguiContextPass, chat_ui);
    }
}

fn receive_messages(mut events: EventReader<ChatMessage>, mut log: ResMut<ChatLog>) {
    for ev in events.read() {
        info!("Received chat message: {}", ev.0);
        log.messages.push(ev.0.clone());
    }
}

fn chat_ui(mut ctxs: EguiContexts, mut log: ResMut<ChatLog>, client: Res<SocketClient>) {
    let ctx = ctxs.ctx_mut();
    egui::Window::new("Chat").show(ctx, |ui| {
        egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
            for msg in &log.messages {
                ui.label(msg);
            }
        });
        ui.horizontal(|ui| {
            let send = ui.text_edit_singleline(&mut log.input).lost_focus() &&
                ui.input(|i| i.key_pressed(egui::Key::Enter));
            if ui.button("Send").clicked() || send {
                if !log.input.is_empty() {
                    let input_clone = log.input.clone();
                    info!("Sending message: {}", input_clone);
                    client.send(input_clone.clone());
                    log.messages.push(format!("Me: {}", input_clone));
                    log.input.clear();
                }
            }
        });
    });
}