use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};
use rand::RngExt;

use crate::components::*;

// ===================== Shader Material =====================

#[derive(Clone, Copy, ShaderType, Debug)]
pub struct ElementalParams {
    pub burn_intensity: f32,
    pub freeze_intensity: f32,
    pub time: f32,
    pub freeze_flash: f32,
    pub seed: f32,
    pub _pad1: f32,
    pub _pad2: f32,
    pub _pad3: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ElementalOverlayMaterial {
    #[uniform(0)]
    pub params: ElementalParams,
}

impl Material2d for ElementalOverlayMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://klotzkoepfe/shaders/elemental_overlay.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

// ===================== Component =====================

#[derive(Component)]
pub struct ElementalOverlay {
    pub zombie_entity: Entity,
    pub freeze_flash: f32,
    pub was_fully_frozen: bool,
    pub seed: f32,
}

// ===================== Systems =====================

/// Spawnt Overlay-Meshes fuer Zombies mit Feuer/Eis-Effekten
pub fn elemental_overlay_spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ElementalOverlayMaterial>>,
    zombie_query: Query<(Entity, &Zombie, &Health), Without<ElementalOverlay>>,
    overlay_query: Query<&ElementalOverlay>,
) {
    for (zombie_entity, zombie, health) in zombie_query.iter() {
        let burn_factor = (zombie.fire_visual / (health.max * 0.5)).clamp(0.0, 1.0);
        let freeze_factor = (zombie.freeze_visual / (health.max * 0.5)).clamp(0.0, 1.0);

        if burn_factor < 0.03 && freeze_factor < 0.03 { continue; }

        let has_overlay = overlay_query.iter().any(|o| o.zombie_entity == zombie_entity);
        if has_overlay { continue; }

        let seed: f32 = rand::rng().random_range(0.0..1000.0);

        let material = materials.add(ElementalOverlayMaterial {
            params: ElementalParams {
                burn_intensity: burn_factor,
                freeze_intensity: freeze_factor,
                time: 0.0,
                freeze_flash: 0.0,
                seed,
                _pad1: 0.0,
                _pad2: 0.0,
                _pad3: 0.0,
            },
        });

        let mesh = meshes.add(Rectangle::new(50.0, 50.0));

        commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_xyz(0.0, 0.0, 20.0),
            ElementalOverlay {
                zombie_entity,
                freeze_flash: 0.0,
                was_fully_frozen: false,
                seed,
            },
        ));
    }
}

/// Updated Overlay-Position und Shader-Uniforms
pub fn elemental_overlay_update(
    time: Res<Time>,
    mut materials: ResMut<Assets<ElementalOverlayMaterial>>,
    mut overlay_query: Query<(
        &mut ElementalOverlay,
        &mut Transform,
        &MeshMaterial2d<ElementalOverlayMaterial>,
    )>,
    zombie_query: Query<(&Zombie, &Health, &Transform), Without<ElementalOverlay>>,
) {
    let dt = time.delta_secs();

    for (mut overlay, mut transform, material_handle) in overlay_query.iter_mut() {
        let Ok((zombie, health, zt)) = zombie_query.get(overlay.zombie_entity) else {
            continue;
        };

        transform.translation.x = zt.translation.x;
        transform.translation.y = zt.translation.y;
        transform.scale = zt.scale;

        let burn_factor = (zombie.fire_visual / (health.max * 0.5)).clamp(0.0, 1.0);
        let freeze_factor = (zombie.freeze_visual / (health.max * 0.5)).clamp(0.0, 1.0);

        // Freeze-Flash: einmal ausloesen wenn voll vereist
        let is_fully_frozen = freeze_factor >= 0.95;
        if is_fully_frozen && !overlay.was_fully_frozen {
            overlay.freeze_flash = 1.0;
        }
        overlay.was_fully_frozen = is_fully_frozen;

        // Flash klingt schnell ab (~0.4s)
        overlay.freeze_flash = (overlay.freeze_flash - dt * 2.5).max(0.0);

        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.params.burn_intensity = burn_factor;
            material.params.freeze_intensity = freeze_factor;
            material.params.time = time.elapsed_secs();
            material.params.freeze_flash = overlay.freeze_flash;
            material.params.seed = overlay.seed;
        }
    }
}

/// Entfernt Overlays wenn Zombie tot oder Effekte abgeklungen
pub fn elemental_overlay_despawn(
    mut commands: Commands,
    overlay_query: Query<(Entity, &ElementalOverlay)>,
    zombie_query: Query<(&Zombie, &Health), Without<ElementalOverlay>>,
) {
    for (entity, overlay) in overlay_query.iter() {
        let should_despawn = match zombie_query.get(overlay.zombie_entity) {
            Ok((zombie, health)) => {
                let burn = (zombie.fire_visual / (health.max * 0.5)).clamp(0.0, 1.0);
                let freeze = (zombie.freeze_visual / (health.max * 0.5)).clamp(0.0, 1.0);
                burn < 0.02 && freeze < 0.02
            }
            Err(_) => true,
        };
        if should_despawn {
            commands.entity(entity).try_despawn();
        }
    }
}
