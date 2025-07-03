use bevy::prelude::*;

#[derive(Component)]
pub struct HpText {
    timer: Timer,
}

impl HpText {
    pub fn new(duration: f32) -> Self {
        Self { timer: Timer::from_seconds(duration, TimerMode::Once) }
    }
}

pub struct HpTextPlugin;

impl Plugin for HpTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, hp_text_system);
    }
}

fn hp_text_system(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut TextColor, &mut Transform, &mut HpText)>,
) {
    for (entity, mut color, mut tf, mut hp_text) in &mut q {
        hp_text.timer.tick(time.delta());
        let pct = hp_text.timer.fraction_remaining();
        color.0 = color.0.with_alpha(pct);
        tf.translation.y += time.delta_secs() * 0.5;
        if hp_text.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}