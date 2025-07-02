use bevy::{
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderRef},
        view::{Layer, RenderLayers},
    },
    math::primitives::Rectangle,
    sprite::{Mesh2d, MeshMaterial2d},
};

use crate::hud::HUD_LAYER;
use crate::input::CollisionEvent;

#[derive(AsBindGroup, TypePath, Debug, Clone)]
pub struct RippleMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub color_texture: Handle<Image>,
    #[uniform(2)]
    pub time: f32,
    #[uniform(2)]
    pub intensity: f32,
}

impl Material2d for RippleMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/ripple.wgsl".into()
    }
}

#[derive(Component)]
struct RippleOverlay;

#[derive(Resource, Default)]
struct RippleState {
    time: f32,
    intensity: f32,
}

pub struct RipplePlugin;

impl Plugin for RipplePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<RippleMaterial>::default())
            .insert_resource(RippleState::default())
            .add_systems(Startup, setup_ripple)
            .add_systems(Update, (update_ripple, handle_collision));
    }
}

fn setup_ripple(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<RippleMaterial>>,
    asset_server: Res<AssetServer>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let size = window.unwrap().resolution.physical_size();
    let mesh = meshes.add(Mesh::from(Rectangle::new(
        size.x as f32,
        size.y as f32,
    )));
    let texture = asset_server.load("starfield.png");
    let material = materials.add(RippleMaterial {
        color_texture: texture,
        time: 0.0,
        intensity: 0.0,
    });
    commands.spawn((
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
        RippleOverlay,
        RenderLayers::layer(HUD_LAYER as Layer),
    ));
}

fn update_ripple(
    time: Res<Time>,
    mut state: ResMut<RippleState>,
    mut materials: ResMut<Assets<RippleMaterial>>,
    q: Query<&Handle<RippleMaterial>, With<RippleOverlay>>,
) {
    state.time += time.delta_seconds();
    state.intensity = (state.intensity - time.delta_seconds()).max(0.0);
    for handle in &q {
        if let Some(mat) = materials.get_mut(handle) {
            mat.time = state.time;
            mat.intensity = state.intensity;
        }
    }
}

fn handle_collision(
    mut ev: EventReader<CollisionEvent>,
    mut state: ResMut<RippleState>,
) {
    if !ev.is_empty() {
        state.intensity = 1.0;
        ev.clear();
    }
}
