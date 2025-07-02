use bevy::prelude::*;
use bevy::log::info;
use bevy_egui::{egui, EguiContexts};

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

fn receive_messages(mut client: ResMut<SocketClient>, mut log: ResMut<ChatLog>) {
    while let Some(msg) = client.try_recv() {
        info!("Received message: {msg}");
        log.messages.push(msg);
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
                    info!("Sending message: {input_clone}");
                    client.send(input_clone.clone());
                    log.messages.push(format!("Me: {}", input_clone));
                    log.input.clear();
                }
            }
        });
    });
}