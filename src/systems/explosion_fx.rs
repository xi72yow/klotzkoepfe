use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::render::render_resource::ShaderType;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dPlugin};

use crate::components::*;
use crate::constants::*;
use super::ground_decals::{DecalStamp, GroundDecalMap};

// ===================== Muzzle Flash =====================

/// Uniform-Daten fuer den Muzzle-Flash-Shader
#[derive(Clone, Copy, ShaderType, Debug)]
pub struct MuzzleFlashParams {
    pub color_inner: LinearRgba,
    pub color_outer: LinearRgba,
    pub progress: f32,
    pub intensity: f32,
    pub _padding1: f32,
    pub _padding2: f32,
}

/// Material2d fuer Muzzle Flash
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct MuzzleFlashMaterial {
    #[uniform(0)]
    pub params: MuzzleFlashParams,
}

impl Material2d for MuzzleFlashMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://klotzkoepfe/shaders/muzzle_flash.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// Component fuer aktive Muzzle Flashes
#[derive(Component)]
pub struct MuzzleFlash {
    pub lifetime: Timer,
    pub owner_id: PlayerId,
    pub flash_size: f32,
}

/// Spawnt einen Muzzle-Flash an der Waffenmuendung
pub fn spawn_muzzle_flash(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<MuzzleFlashMaterial>,
    player: &Player,
    player_pos: Vec2,
    color_inner: LinearRgba,
    color_outer: LinearRgba,
    size: f32,
) {
    let tip = weapon_tip_pos(player, player_pos);
    spawn_muzzle_flash_at(commands, meshes, materials, player, tip, color_inner, color_outer, size);
}

pub fn spawn_muzzle_flash_at(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<MuzzleFlashMaterial>,
    player: &Player,
    tip_pos: Vec2,
    color_inner: LinearRgba,
    color_outer: LinearRgba,
    size: f32,
) {
    let facing = player.facing;
    let angle = facing.y.atan2(facing.x);
    let lifetime = 0.1;

    let offset = facing * size * 0.3;
    let pos = tip_pos + offset;

    let material = materials.add(MuzzleFlashMaterial {
        params: MuzzleFlashParams {
            color_inner,
            color_outer,
            progress: 0.0,
            intensity: 1.0,
            _padding1: 0.0,
            _padding2: 0.0,
        },
    });

    let mesh = meshes.add(Rectangle::new(size * 2.5, size * 2.0));

    commands.spawn((
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_translation(pos.extend(16.0))
            .with_rotation(Quat::from_rotation_z(angle)),
        MuzzleFlash {
            lifetime: Timer::from_seconds(lifetime, TimerMode::Once),
            owner_id: player.id,
            flash_size: size,
        },
    ));
}

/// Berechnet die Waffentip-Position fuer einen Spieler
fn weapon_tip_pos(player: &Player, player_pos: Vec2) -> Vec2 {
    let facing = player.facing;
    let ws_size = player.weapon.sprite_size();
    let arm_offset_x = 9.5; // ARM_OFFSET_X aus player.rs
    let weapon_end_x = facing.x * (arm_offset_x + ws_size.x + 2.0);
    let weapon_end_y = -2.0 + facing.y * (arm_offset_x + ws_size.y);
    player_pos + Vec2::new(weapon_end_x, weapon_end_y)
}

/// Update: animiert Muzzle Flashes, verankert an Waffe, despawnt
pub fn update_muzzle_flashes(
    mut commands: Commands,
    time: Res<Time>,
    mut materials: ResMut<Assets<MuzzleFlashMaterial>>,
    mut query: Query<(
        Entity,
        &mut MuzzleFlash,
        &mut Transform,
        &MeshMaterial2d<MuzzleFlashMaterial>,
    )>,
    player_query: Query<(&Player, &Transform, &Children), Without<MuzzleFlash>>,
    arm_query: Query<(&PlayerArm, &Transform), (Without<Player>, Without<MuzzleFlash>)>,
) {
    for (entity, mut flash, mut transform, material_handle) in query.iter_mut() {
        flash.lifetime.tick(time.delta());
        let progress = flash.lifetime.fraction();

        // Flash an Waffentip verankern
        for (player, pt, children) in player_query.iter() {
            if player.id == flash.owner_id {
                let mut weapon_arm_pos = Vec2::new(9.5, -2.0);
                for child in children.iter() {
                    if let Ok((arm, arm_t)) = arm_query.get(child) {
                        if arm.has_weapon {
                            weapon_arm_pos = Vec2::new(arm_t.translation.x, arm_t.translation.y);
                        }
                    }
                }
                let tip = super::player::weapon_tip(player, pt.translation.truncate(), weapon_arm_pos);
                let facing = player.facing;
                let offset = facing * flash.flash_size * 0.3;
                transform.translation.x = tip.x + offset.x;
                transform.translation.y = tip.y + offset.y;
                let angle = facing.y.atan2(facing.x);
                transform.rotation = Quat::from_rotation_z(angle);
                break;
            }
        }

        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.params.progress = progress;
        }

        if flash.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

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
    let mesh_size = radius * 2.0;
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
        // Brandfleck beim ersten Frame stempeln (waehrend Explosion sichtbar)
        if !explosion.damaged {
            let pos = transform.translation.truncate();
            decal_map.pending_stamps.push(DecalStamp::Burn {
                position: pos,
                radius: explosion.radius,
            });
        }

        explosion.lifetime.tick(time.delta());
        let progress = explosion.lifetime.fraction();

        // Material-Uniforms updaten
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.params.progress = progress;
        }

        if explosion.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
