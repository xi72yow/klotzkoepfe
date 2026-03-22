use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::render::render_resource::ShaderType;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dPlugin};

use crate::components::*;
use crate::constants::*;
use super::ground_decals::{DecalStamp, GroundDecalMap};

/// Uniform-Daten fuer den Explosions-Shader
#[derive(Clone, Copy, ShaderType)]
pub struct ExplosionParams {
    pub color_inner: LinearRgba,
    pub color_outer: LinearRgba,
    pub progress: f32,
    pub level: f32,
    pub _padding1: f32,
    pub _padding2: f32,
}

/// Material2d fuer Explosionen
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ExplosionMaterial {
    #[uniform(0)]
    pub params: ExplosionParams,
}

impl Material2d for ExplosionMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://klotzkoepfe/shaders/explosion.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

impl std::fmt::Debug for ExplosionParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExplosionParams")
            .field("progress", &self.progress)
            .field("level", &self.level)
            .finish()
    }
}

/// Plugin registrieren
pub fn explosion_plugin(app: &mut App) {
    app.add_plugins(Material2dPlugin::<ExplosionMaterial>::default());
}

/// Shader-Explosion spawnen (ersetzt die alte spawn_explosion fuer Granaten/Raketen/Minen)
pub fn spawn_shader_explosion(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ExplosionMaterial>,
    pos: Vec3,
    radius: f32,
    damage: f32,
    level: u32,
) {
    let level_f = level.max(1) as f32;
    let lifetime = EXPLOSION_LIFETIME + 0.15 * (level_f - 1.0);

    let material = materials.add(ExplosionMaterial {
        params: ExplosionParams {
            color_inner: LinearRgba::new(1.0, 0.95, 0.7, 1.0),
            color_outer: LinearRgba::new(1.0, 0.3, 0.05, 0.9),
            progress: 0.0,
            level: level_f,
            _padding1: 0.0,
            _padding2: 0.0,
        },
    });

    // Mesh etwas groesser als Radius damit Shockwave-Ring Platz hat
    let mesh_size = radius * 2.5 * (1.0 + 0.3 * level_f);
    let mesh = meshes.add(Rectangle::new(mesh_size, mesh_size));

    commands.spawn((
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_translation(pos.truncate().extend(15.0)),
        ShaderExplosion {
            lifetime: Timer::from_seconds(lifetime, TimerMode::Once),
            damage,
            radius,
            damaged: false,
            level,
        },
    ));
}

/// Update: animiert die Shader-Explosionen
pub fn update_shader_explosions(
    mut commands: Commands,
    time: Res<Time>,
    mut decal_map: ResMut<GroundDecalMap>,
    mut materials: ResMut<Assets<ExplosionMaterial>>,
    mut query: Query<(
        Entity,
        &mut ShaderExplosion,
        &MeshMaterial2d<ExplosionMaterial>,
        &Transform,
    )>,
) {
    for (entity, mut explosion, material_handle, transform) in query.iter_mut() {
        explosion.lifetime.tick(time.delta());
        let progress = explosion.lifetime.fraction();

        // Material-Uniforms updaten
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.params.progress = progress;
        }

        if explosion.lifetime.is_finished() {
            // Brandfleck stempeln
            let pos = transform.translation.truncate();
            decal_map.pending_stamps.push(DecalStamp::Burn {
                position: pos,
                radius: explosion.radius,
            });
            commands.entity(entity).despawn();
        }
    }
}
