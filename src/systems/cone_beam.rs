use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};

use crate::components::*;
use crate::constants::*;
use crate::resources::*;

// ===================== Shader Material =====================

#[derive(Clone, Copy, ShaderType, Debug)]
pub struct ConeBeamParams {
    pub color_inner: LinearRgba,
    pub color_outer: LinearRgba,
    pub time: f32,
    pub intensity: f32,
    pub cone_angle: f32,
    pub beam_type: f32, // 0.0 = flame, 1.0 = freeze
    // Treffer-Daten: bis zu 8 normalisierte Distanzen (0..1) wo Zombies im Strahl sind
    pub hit_distances: [Vec4; 2], // 8 floats als 2x vec4
    pub hit_count: f32,
    pub _pad0: f32,
    pub _pad1: f32,
    pub _pad2: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ConeBeamMaterial {
    #[uniform(0)]
    pub params: ConeBeamParams,
}

impl Material2d for ConeBeamMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://klotzkoepfe/shaders/cone_beam.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

// ===================== Component =====================

const MAX_HITS: usize = 8;

#[derive(Component)]
pub struct ConeBeam {
    pub owner_id: PlayerId,
    pub beam_type: ConeBeamType,
    pub damage_timer: Timer,
    pub current_intensity: f32,
    // Treffer-Distanzen (normalisiert 0..1, sortiert)
    pub hit_distances: [f32; MAX_HITS],
    pub hit_count: u32,
    // Smooth-Werte fuer sanftes Ein/Ausblenden der Treffer
    pub smooth_hit_distances: [f32; MAX_HITS],
    pub smooth_hit_count: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ConeBeamType {
    Flame,
    Freeze,
}

// ===================== Spawn / Despawn =====================

pub fn cone_beam_spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ConeBeamMaterial>>,
    player_query: Query<(&Player, &Children, Entity), Without<ConeBeam>>,
    beam_query: Query<&ConeBeam>,
) {
    for (player, children, _player_entity) in player_query.iter() {
        let beam_type = match player.weapon {
            WeaponType::Flamethrower => ConeBeamType::Flame,
            WeaponType::FreezeGun => ConeBeamType::Freeze,
            _ => continue,
        };

        // Pruefen ob schon ein Beam fuer diesen Spieler existiert
        let has_beam = beam_query.iter().any(|b| b.owner_id == player.id && b.beam_type == beam_type);
        if has_beam { continue; }

        // Mesh groesser als sichtbarer Effekt -> Padding damit Noise nicht abgeschnitten wird
        let (color_inner, color_outer, cone_angle, length, width, bt) = match beam_type {
            ConeBeamType::Flame => (
                LinearRgba::new(1.0, 0.95, 0.4, 1.0),
                LinearRgba::new(1.0, 0.15, 0.0, 0.8),
                0.15,
                170.0,  // 130 Effekt + 40 Padding
                120.0,  // 80 Effekt + 40 Padding
                0.0,
            ),
            ConeBeamType::Freeze => (
                LinearRgba::new(0.85, 0.97, 1.0, 1.0),
                LinearRgba::new(0.15, 0.4, 0.95, 0.7),
                0.25,
                200.0,  // 150 Effekt + 50 Padding
                170.0,  // 120 Effekt + 50 Padding
                1.0,
            ),
        };

        let material = materials.add(ConeBeamMaterial {
            params: ConeBeamParams {
                color_inner,
                color_outer,
                time: 0.0,
                intensity: 0.0,
                cone_angle,
                beam_type: bt,
                hit_distances: [Vec4::ZERO; 2],
                hit_count: 0.0,
                _pad0: 0.0,
                _pad1: 0.0,
                _pad2: 0.0,
            },
        });

        let mesh = meshes.add(Rectangle::new(length, width));

        commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_xyz(0.0, 0.0, 15.0),
            ConeBeam {
                owner_id: player.id,
                beam_type,
                damage_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                current_intensity: 0.0,
                hit_distances: [0.0; MAX_HITS],
                hit_count: 0,
                smooth_hit_distances: [0.0; MAX_HITS],
                smooth_hit_count: 0.0,
            },
        ));
    }
}

/// Despawnt Beams wenn der Spieler die Waffe gewechselt hat
pub fn cone_beam_despawn(
    mut commands: Commands,
    beam_query: Query<(Entity, &ConeBeam)>,
    player_query: Query<&Player>,
) {
    for (entity, beam) in beam_query.iter() {
        let player_has_weapon = player_query.iter().any(|p| {
            p.id == beam.owner_id && match beam.beam_type {
                ConeBeamType::Flame => p.weapon == WeaponType::Flamethrower,
                ConeBeamType::Freeze => p.weapon == WeaponType::FreezeGun,
            }
        });
        if !player_has_weapon {
            commands.entity(entity).try_despawn();
        }
    }
}

// ===================== Update Visual =====================

pub fn cone_beam_update(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut materials: ResMut<Assets<ConeBeamMaterial>>,
    mut beam_query: Query<(
        &mut ConeBeam,
        &mut Transform,
        &MeshMaterial2d<ConeBeamMaterial>,
    )>,
    player_query: Query<(&Player, &Transform, &Children), Without<ConeBeam>>,
    arm_query: Query<(&PlayerArm, &Transform), (Without<Player>, Without<ConeBeam>)>,
) {
    for (mut beam, mut transform, material_handle) in beam_query.iter_mut() {
        // Finde den zugehoerigen Spieler
        let Some((player, pt, children)) = player_query.iter().find(|(p, _, _)| p.id == beam.owner_id) else {
            continue;
        };

        let wants_shoot = match player.id {
            PlayerId::P1 => keyboard.pressed(KeyCode::Space),
            PlayerId::P2 => keyboard.pressed(KeyCode::Enter),
        };

        let is_active = wants_shoot && !player.reloading && player.ammo > 0;

        // Waffen-Arm-Position (aus aktueller Arm-Transform lesen)
        let mut weapon_arm_pos = Vec2::new(9.5, -4.0);
        for child in children.iter() {
            if let Ok((arm, arm_t)) = arm_query.get(child) {
                if arm.has_weapon {
                    weapon_arm_pos = arm_t.translation.truncate();
                }
            }
        }

        let player_pos = pt.translation.truncate();
        let tip = super::player::weapon_tip(player, player_pos, weapon_arm_pos);
        let facing = player.facing;
        let angle = facing.y.atan2(facing.x);

        // Beam-Mesh an Waffentip positionieren
        // Shader hat UV-Padding: sichtbarer Effekt startet erst bei UV.x ~0.12/0.10
        // Mesh muss zurueckversetzt werden damit Effekt am Laufende startet
        let (length, _width, uv_padding) = match beam.beam_type {
            ConeBeamType::Flame => (170.0, 120.0, 0.12),
            ConeBeamType::Freeze => (200.0, 170.0, 0.10),
        };
        let center = tip + facing * (length * 0.5 - length * uv_padding);
        transform.translation.x = center.x;
        transform.translation.y = center.y;
        transform.rotation = Quat::from_rotation_z(angle);

        // Smooth intensity ramp: ~0.15s anlauf, ~0.25s ablauf
        let dt = time.delta_secs();
        let target = if is_active { 1.0_f32 } else { 0.0_f32 };
        let ramp_speed = if is_active { 7.0 } else { 4.0 };
        beam.current_intensity = beam.current_intensity + (target - beam.current_intensity) * (ramp_speed * dt).min(1.0);

        // Smooth hit distances: sanft interpolieren fuer flackerfreie Treffer-Effekte
        let hit_lerp = (12.0 * dt).min(1.0);
        let fade_lerp = (6.0 * dt).min(1.0);
        let target_count = if is_active { beam.hit_count as f32 } else { 0.0 };
        beam.smooth_hit_count += (target_count - beam.smooth_hit_count) * hit_lerp;
        for i in 0..MAX_HITS {
            if (i as u32) < beam.hit_count && is_active {
                beam.smooth_hit_distances[i] += (beam.hit_distances[i] - beam.smooth_hit_distances[i]) * hit_lerp;
            } else {
                // Treffer ausblenden: Distanz auf 1.0 schieben (= Strahlende, unsichtbar)
                beam.smooth_hit_distances[i] += (1.0 - beam.smooth_hit_distances[i]) * fade_lerp;
            }
        }

        // Material updaten mit Treffer-Daten
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.params.time = time.elapsed_secs();
            material.params.intensity = beam.current_intensity;
            material.params.hit_count = beam.smooth_hit_count;
            material.params.hit_distances = [
                Vec4::new(
                    beam.smooth_hit_distances[0],
                    beam.smooth_hit_distances[1],
                    beam.smooth_hit_distances[2],
                    beam.smooth_hit_distances[3],
                ),
                Vec4::new(
                    beam.smooth_hit_distances[4],
                    beam.smooth_hit_distances[5],
                    beam.smooth_hit_distances[6],
                    beam.smooth_hit_distances[7],
                ),
            ];
        }

        beam.damage_timer.tick(time.delta());
    }
}

// ===================== AoE Damage =====================

pub fn cone_beam_damage(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    settings: Res<GameSettings>,
    score: Res<Score>,
    mut beam_query: Query<&mut ConeBeam>,
    player_query: Query<(&Player, &Transform, &Children), Without<ConeBeam>>,
    arm_query: Query<(&PlayerArm, &Transform), (Without<Player>, Without<ConeBeam>, Without<Zombie>)>,
    mut zombie_query: Query<(Entity, &Transform, &mut Health, &mut Zombie), Without<Player>>,
    mut commands: Commands,
) {
    for mut beam in beam_query.iter_mut() {
        if !beam.damage_timer.just_finished() { continue; }

        let Some((player, pt, children)) = player_query.iter().find(|(p, _, _)| p.id == beam.owner_id) else {
            continue;
        };

        let wants_shoot = match player.id {
            PlayerId::P1 => keyboard.pressed(KeyCode::Space),
            PlayerId::P2 => keyboard.pressed(KeyCode::Enter),
        };

        let lvl = settings.weapon_level(player.weapon, score.points);
        let ws = settings.weapon_at_level(player.weapon, lvl);

        // Waffen-Arm-Position (aus aktueller Arm-Transform lesen)
        let mut weapon_arm_pos = Vec2::new(9.5, -4.0);
        for child in children.iter() {
            if let Ok((arm, arm_t)) = arm_query.get(child) {
                if arm.has_weapon {
                    weapon_arm_pos = arm_t.translation.truncate();
                }
            }
        }

        let player_pos = pt.translation.truncate();
        let tip = super::player::weapon_tip(player, player_pos, weapon_arm_pos);
        let facing = player.facing;

        let (range, half_angle) = match beam.beam_type {
            ConeBeamType::Flame => (ws.range.max(100.0), 0.35_f32),
            ConeBeamType::Freeze => (ws.range.max(120.0), 0.55_f32),
        };

        // Treffer-Distanzen sammeln
        let mut hit_dists: Vec<f32> = Vec::new();

        if !wants_shoot || player.reloading || player.ammo == 0 {
            // Nicht aktiv: keine Treffer
            beam.hit_distances = [0.0; MAX_HITS];
            beam.hit_count = 0;
            continue;
        }

        // Alle Zombies im Kegel treffen
        for (zombie_entity, zt, mut health, mut zombie) in zombie_query.iter_mut() {
            let zombie_pos = zt.translation.truncate();
            let to_zombie = zombie_pos - tip;
            let dist = to_zombie.length();

            if dist > range || dist < 1.0 { continue; }

            // Winkel pruefen
            let angle_to_zombie = to_zombie.normalize().dot(facing).acos();
            if angle_to_zombie > half_angle { continue; }

            // Normalisierte Distanz fuer Shader (0 = Muendung, 1 = max Range)
            hit_dists.push(dist / range);

            let push_dir = to_zombie.normalize();

            match beam.beam_type {
                ConeBeamType::Flame => {
                    let dmg = ws.damage * 0.1;
                    health.current -= dmg;
                    zombie.fire_visual += dmg * 4.0;
                    if let Ok(mut ec) = commands.get_entity(zombie_entity) {
                        ec.try_insert(Burning {
                            damage_per_second: ws.damage * 0.3,
                            timer: Timer::from_seconds(2.0, TimerMode::Once),
                            tick_timer: Timer::from_seconds(0.25, TimerMode::Repeating),
                        });
                        ec.try_insert(Knockback {
                            velocity: push_dir * 180.0,
                            duration: Timer::from_seconds(0.12, TimerMode::Once),
                        });
                    }
                }
                ConeBeamType::Freeze => {
                    let dmg = ws.damage * 0.03;
                    health.current -= dmg;
                    zombie.freeze_visual += dmg * 8.0;
                    zombie.speed_modifier = if ws.slow_factor > 0.0 { ws.slow_factor } else { 0.15 };
                    zombie.freeze_timer = Timer::from_seconds(
                        if ws.slow_duration > 0.0 { ws.slow_duration } else { 3.0 },
                        TimerMode::Once,
                    );
                }
            }
        }

        // Sortiert speichern (naechste zuerst)
        hit_dists.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        beam.hit_count = hit_dists.len().min(MAX_HITS) as u32;
        beam.hit_distances = [0.0; MAX_HITS];
        for (i, d) in hit_dists.iter().take(MAX_HITS).enumerate() {
            beam.hit_distances[i] = *d;
        }
    }
}
