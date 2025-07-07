use bevy::prelude::*;

use crate::goals::{FinishGoal, StartGoal};
use crate::globals::Controlled;

pub struct LapTimerPlugin;

impl Plugin for LapTimerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LapTimer::default())
            .add_systems(Update, lap_timer_system);
    }
}

#[derive(Resource, Default)]
pub struct LapTimer {
    pub last_lap: Option<f32>,
    pub best_lap: Option<f32>,
    running: bool,
    start_time: f64,
    in_start: bool,
    in_finish: bool,
}

const GOAL_RADIUS: f32 = 2.0;

fn lap_timer_system(
    time: Res<Time>,
    mut timer: ResMut<LapTimer>,
    player_q: Query<&Transform, With<Controlled>>,
    start_q: Query<&Transform, With<StartGoal>>,
    finish_q: Query<&Transform, With<FinishGoal>>,
) {
    let Ok(player_tf) = player_q.single() else { return; };
    let Ok(start_tf) = start_q.single() else { return; };
    let Ok(finish_tf) = finish_q.single() else { return; };

    let player_pos = player_tf.translation;

    let in_start = player_pos.distance(start_tf.translation) < GOAL_RADIUS;
    if in_start {
        if !timer.in_start {
            timer.running = true;
            timer.start_time = time.elapsed_secs_f64();
            info!("Lap started at {:.2} seconds", timer.start_time);
        }
        timer.in_start = true;
    } else {
        timer.in_start = false;
    }

    if timer.running {
        let in_finish = player_pos.distance(finish_tf.translation) < GOAL_RADIUS;
        if in_finish && !timer.in_finish {
            let lap = (time.elapsed_secs_f64() - timer.start_time) as f32;
            timer.last_lap = Some(lap);
            if let Some(best) = timer.best_lap {
                if lap < best {
                    timer.best_lap = Some(lap);
                    info!("New best lap: {:.2} seconds", lap);
                }
            } else {
                timer.best_lap = Some(lap);
            }
            timer.running = false;
        }
        timer.in_finish = in_finish;
    } else {
        timer.in_finish = false;
    }
}
