use std::collections::VecDeque;

use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::NoFrustumCulling,
    light::NotShadowCaster,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

const SLIP_THRESHOLD: f32 = 0.8;
const MIN_LOAD: f32 = 100.0;
const MIN_DISTANCE: f32 = 0.12;
const MAX_DISTANCE: f32 = 2.5;
const HALF_WIDTH: f32 = 0.14;
const MAX_SEGMENTS: usize = 4096;

#[derive(Clone, Copy)]
pub(crate) struct SkidAnchor {
    point: Vec3,
    normal: Vec3,
    side: Vec3,
    opacity: f32,
}

struct SkidSegment {
    positions: [Vec3; 4],
    normals: [Vec3; 4],
    opacity: [f32; 4],
}

/// One bounded geometry buffer shared by all vehicles. Anchors remain per wheel.
#[derive(Resource, Default)]
pub(crate) struct SkidMarks {
    segments: VecDeque<SkidSegment>,
    dirty: bool,
}

impl SkidMarks {
    /// Called for every physics contact sample, before the tire forces are solved.
    pub(crate) fn sample(
        &mut self,
        anchor: &mut Option<SkidAnchor>,
        point: Vec3,
        normal: Vec3,
        lateral_speed: f32,
        load: f32,
    ) {
        let slip = lateral_speed.abs();
        if slip < SLIP_THRESHOLD || load < MIN_LOAD {
            *anchor = None;
            return;
        }
        let strength = ((slip - SLIP_THRESHOLD) / 5.0).clamp(0.0, 1.0);
        let opacity = (0.15 + strength * 0.65) * (load / 2000.0).clamp(0.0, 1.0);
        let current = SkidAnchor { point, normal, side: Vec3::ZERO, opacity };
        let Some(previous) = *anchor else {
            *anchor = Some(current);
            return;
        };
        let distance = point.distance(previous.point);
        // Do not bridge resets, sudden contact changes, or very large physics steps.
        if distance > MAX_DISTANCE || normal.dot(previous.normal) < 0.7 {
            *anchor = Some(current);
            return;
        }
        if distance < MIN_DISTANCE { return; }

        // Width is perpendicular to the slide path, including pure sideways slides.
        let side = normal.cross(point - previous.point).normalize_or_zero() * HALF_WIDTH;
        if side.length_squared() < 1.0e-6 {
            *anchor = Some(current);
            return;
        }
        if previous.side != Vec3::ZERO && previous.side.dot(side) < 0.0 {
            *anchor = Some(current);
            return;
        }
        // Share endpoint edges between successive segments for continuous curves.
        let previous_side = if previous.side == Vec3::ZERO { side } else { previous.side };
        let start = previous.point + previous.normal * 0.025;
        let end = point + normal * 0.025;
        self.segments.push_back(SkidSegment {
            positions: [start - previous_side, start + previous_side, end - side, end + side],
            normals: [previous.normal, previous.normal, normal, normal],
            opacity: [previous.opacity, previous.opacity, opacity, opacity],
        });
        if self.segments.len() > MAX_SEGMENTS { self.segments.pop_front(); }
        self.dirty = true;
        *anchor = Some(SkidAnchor { side, ..current });
    }
}

#[derive(Resource)]
struct SkidMesh(Handle<Mesh>);

pub(crate) struct SkidMarksPlugin;

impl Plugin for SkidMarksPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SkidMarks>()
            .add_systems(Startup, setup_skid_mesh)
            .add_systems(Update, update_skid_mesh);
    }
}

fn setup_skid_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new())
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<[f32; 3]>::new())
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, Vec::<[f32; 4]>::new())
            .with_inserted_indices(Indices::U32(Vec::new())),
    );
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.spawn((
        Name::new("Persistent tire skid marks"),
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material),
        Transform::default(),
        // World-space geometry grows away from its initially empty bounds.
        NoFrustumCulling,
        NotShadowCaster,
    ));
    commands.insert_resource(SkidMesh(mesh));
}

fn update_skid_mesh(
    mut marks: ResMut<SkidMarks>,
    handle: Res<SkidMesh>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if !marks.dirty { return; }
    let Some(mut mesh) = meshes.get_mut(&handle.0) else { return; };
    let count = marks.segments.len();
    let mut positions = Vec::with_capacity(count * 4);
    let mut normals = Vec::with_capacity(count * 4);
    let mut colors = Vec::with_capacity(count * 4);
    let mut indices = Vec::with_capacity(count * 6);
    for segment in &marks.segments {
        let base = positions.len() as u32;
        positions.extend(segment.positions.map(|point| point.to_array()));
        normals.extend(segment.normals.map(|normal| normal.to_array()));
        colors.extend(segment.opacity.map(|alpha| [0.018, 0.016, 0.014, alpha]));
        indices.extend_from_slice(&[base, base + 2, base + 1, base + 1, base + 2, base + 3]);
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    marks.dirty = false;
}
