use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::render::render_resource::PrimitiveTopology;
use bevy::mesh::Indices;
use bevy::asset::RenderAssetUsages;
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
    pub hit_distances: [Vec4; 2],
    pub hit_count: f32,
    pub move_speed: f32,
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
/// Anzahl Punkte entlang des Strahls
const CHAIN_POINTS: usize = 5;
/// Anzahl Mesh-Segmente (mehr = glattere Kurve)
const MESH_SEGMENTS: usize = 8;
/// Groesse des History-Ringpuffers
const HISTORY_SIZE: usize = 64;

/// Ein Frame der Emissionsgeschichte: wo war die Waffentip, wohin zeigte der Spieler
#[derive(Clone, Copy)]
struct EmissionFrame {
    tip: Vec2,
    facing: Vec2,
    time: f32,
}

impl Default for EmissionFrame {
    fn default() -> Self {
        Self { tip: Vec2::ZERO, facing: Vec2::X, time: 0.0 }
    }
}

#[derive(Component)]
pub struct ConeBeam {
    pub owner_id: PlayerId,
    pub beam_type: ConeBeamType,
    pub damage_timer: Timer,
    pub current_intensity: f32,
    pub hit_distances: [f32; MAX_HITS],
    pub hit_count: u32,
    pub smooth_hit_distances: [f32; MAX_HITS],
    pub smooth_hit_count: f32,
    pub prev_pos: Vec2,
    pub smooth_move_speed: f32,
    // Geglaettete Blickrichtung (vermeidet harte Spruenge bei WASD-Richtungswechsel)
    smooth_facing: Vec2,
    // Emissionsgeschichte: Ring-Buffer mit (tip, facing, time) pro Frame
    history: [EmissionFrame; HISTORY_SIZE],
    history_head: usize,
    history_initialized: bool,
    // Berechnete Strahlpositionen (fuer Mesh + Damage)
    pub chain_pos: [Vec2; CHAIN_POINTS],
    pub chain_initialized: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ConeBeamType {
    Flame,
    Freeze,
}

// ===================== Mesh Helpers =====================

fn create_beam_strip_mesh() -> Mesh {
    let vert_count = (MESH_SEGMENTS + 1) * 2;
    let positions: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0]; vert_count];
    let normals: Vec<[f32; 3]> = vec![[0.0, 0.0, 1.0]; vert_count];

    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(vert_count);
    for i in 0..=MESH_SEGMENTS {
        let u = i as f32 / MESH_SEGMENTS as f32;
        uvs.push([u, 0.0]);
        uvs.push([u, 1.0]);
    }

    let mut indices: Vec<u32> = Vec::new();
    for i in 0..MESH_SEGMENTS as u32 {
        let bl = i * 2;
        let br = i * 2 + 1;
        let tl = (i + 1) * 2;
        let tr = (i + 1) * 2 + 1;
        indices.extend_from_slice(&[bl, br, tr, bl, tr, tl]);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Interpoliert eine Position auf der Chain (t=0..1)
fn chain_lerp(chain: &[Vec2; CHAIN_POINTS], t: f32) -> Vec2 {
    let scaled = t * (CHAIN_POINTS - 1) as f32;
    let idx = (scaled as usize).min(CHAIN_POINTS - 2);
    let frac = scaled - idx as f32;
    chain[idx] * (1.0 - frac) + chain[idx + 1] * frac
}

/// Berechnet die Richtung auf der Chain an Position t (0..1)
fn chain_dir(chain: &[Vec2; CHAIN_POINTS], t: f32, fallback: Vec2) -> Vec2 {
    let dt = 0.01;
    let a = chain_lerp(chain, (t - dt).max(0.0));
    let b = chain_lerp(chain, (t + dt).min(1.0));
    (b - a).normalize_or(fallback)
}

/// Berechnet Vertex-Positionen fuer das Beam-Strip-Mesh
fn compute_beam_vertices(
    chain: &[Vec2; CHAIN_POINTS],
    tip: Vec2,
    half_width: f32,
    facing_dir: Vec2,
) -> Vec<[f32; 3]> {
    let mut positions = Vec::with_capacity((MESH_SEGMENTS + 1) * 2);
    // Senkrechte von Vertex zu Vertex propagieren (statt globale Referenz)
    // So dreht sie sich sanft mit der Kurve mit, kein Flip bei >90 Grad
    let mut prev_perp = Vec2::new(-facing_dir.y, facing_dir.x);

    for i in 0..=MESH_SEGMENTS {
        let t = i as f32 / MESH_SEGMENTS as f32;
        let center = chain_lerp(chain, t) - tip;
        let dir = chain_dir(chain, t, facing_dir);
        let raw_perp = Vec2::new(-dir.y, dir.x);
        // Orientierung am Vorgaenger ausrichten (nie ploetzlich flippen)
        let perp = if raw_perp.dot(prev_perp) >= 0.0 { raw_perp } else { -raw_perp };
        prev_perp = perp;

        let left = center + perp * half_width;
        let right = center - perp * half_width;
        positions.push([left.x, left.y, 0.0]);
        positions.push([right.x, right.y, 0.0]);
    }
    positions
}

// ===================== History Lookup =====================

/// Schaut in der Emissionsgeschichte nach: wo war tip/facing vor t_ago Sekunden?
/// Interpoliert linear zwischen den zwei naechsten Frames.
fn lookup_history(
    history: &[EmissionFrame; HISTORY_SIZE],
    head: usize,
    current_time: f32,
    t_ago: f32,
) -> (Vec2, Vec2) {
    let target_time = current_time - t_ago;

    // Letzten Frame als Fallback
    let latest_idx = (head + HISTORY_SIZE - 1) % HISTORY_SIZE;
    let mut prev_idx = latest_idx;
    let mut next_idx = latest_idx;

    // Rueckwaerts durch History laufen, zwei Frames um target_time finden
    for i in 0..HISTORY_SIZE - 1 {
        let idx = (head + HISTORY_SIZE - 1 - i) % HISTORY_SIZE;
        let prev = (head + HISTORY_SIZE - 2 - i) % HISTORY_SIZE;
        if history[idx].time >= target_time && history[prev].time <= target_time {
            next_idx = idx;
            prev_idx = prev;
            break;
        }
    }

    let f0 = &history[prev_idx];
    let f1 = &history[next_idx];
    let dt = f1.time - f0.time;

    if dt < 0.0001 {
        return (f1.tip, f1.facing);
    }

    let t = ((target_time - f0.time) / dt).clamp(0.0, 1.0);
    let tip = f0.tip * (1.0 - t) + f1.tip * t;
    // Facing interpolieren (Winkel-Lerp fuer korrekte Drehung)
    let a0 = f0.facing.y.atan2(f0.facing.x);
    let a1 = f1.facing.y.atan2(f1.facing.x);
    let mut angle_diff = a1 - a0;
    if angle_diff > std::f32::consts::PI { angle_diff -= 2.0 * std::f32::consts::PI; }
    if angle_diff < -std::f32::consts::PI { angle_diff += 2.0 * std::f32::consts::PI; }
    let angle = a0 + angle_diff * t;
    let facing = Vec2::new(angle.cos(), angle.sin());

    (tip, facing)
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

        let has_beam = beam_query.iter().any(|b| b.owner_id == player.id && b.beam_type == beam_type);
        if has_beam { continue; }

        let (color_inner, color_outer, cone_angle, bt) = match beam_type {
            ConeBeamType::Flame => (
                LinearRgba::new(1.0, 0.95, 0.4, 1.0),
                LinearRgba::new(1.0, 0.15, 0.0, 0.8),
                0.15, 0.0,
            ),
            ConeBeamType::Freeze => (
                LinearRgba::new(0.85, 0.97, 1.0, 1.0),
                LinearRgba::new(0.15, 0.4, 0.95, 0.7),
                0.25, 1.0,
            ),
        };

        let material = materials.add(ConeBeamMaterial {
            params: ConeBeamParams {
                color_inner, color_outer,
                time: 0.0, intensity: 0.0, cone_angle, beam_type: bt,
                hit_distances: [Vec4::ZERO; 2], hit_count: 0.0,
                move_speed: 0.0, _pad1: 0.0, _pad2: 0.0,
            },
        });

        let mesh = meshes.add(create_beam_strip_mesh());

        commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_xyz(0.0, 0.0, 15.0),
            ConeBeam {
                owner_id: player.id, beam_type,
                damage_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                current_intensity: 0.0,
                hit_distances: [0.0; MAX_HITS], hit_count: 0,
                smooth_hit_distances: [0.0; MAX_HITS], smooth_hit_count: 0.0,
                prev_pos: Vec2::ZERO, smooth_move_speed: 0.0, smooth_facing: Vec2::X,
                history: [EmissionFrame::default(); HISTORY_SIZE],
                history_head: 0, history_initialized: false,
                chain_pos: [Vec2::ZERO; CHAIN_POINTS],
                chain_initialized: false,
            },
        ));
    }
}

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
    settings: Res<GameSettings>,
    score: Res<Score>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ConeBeamMaterial>>,
    mut beam_query: Query<(
        &mut ConeBeam,
        &mut Transform,
        &MeshMaterial2d<ConeBeamMaterial>,
        &Mesh2d,
    )>,
    player_query: Query<(&Player, &Transform, &Children), Without<ConeBeam>>,
    arm_query: Query<(&PlayerArm, &Transform), (Without<Player>, Without<ConeBeam>)>,
) {
    for (mut beam, mut transform, material_handle, mesh2d) in beam_query.iter_mut() {
        let Some((player, pt, children)) = player_query.iter().find(|(p, _, _)| p.id == beam.owner_id) else {
            continue;
        };

        let wants_shoot = match player.id {
            PlayerId::P1 => keyboard.pressed(KeyCode::Space),
            PlayerId::P2 => keyboard.pressed(KeyCode::Enter),
        };
        let is_active = wants_shoot && !player.reloading && player.ammo > 0;

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
        let current_time = time.elapsed_secs();
        let dt = time.delta_secs();

        // Facing glaetten: Winkel-Lerp ueber kuerzesten Weg (vermeidet harte WASD-Spruenge)
        let target_angle = facing.y.atan2(facing.x);
        let current_angle = beam.smooth_facing.y.atan2(beam.smooth_facing.x);
        let mut angle_diff = target_angle - current_angle;
        if angle_diff > std::f32::consts::PI { angle_diff -= 2.0 * std::f32::consts::PI; }
        if angle_diff < -std::f32::consts::PI { angle_diff += 2.0 * std::f32::consts::PI; }
        let smooth_angle = current_angle + angle_diff * (15.0 * dt).min(1.0);
        beam.smooth_facing = Vec2::new(smooth_angle.cos(), smooth_angle.sin());
        let facing_dir = beam.smooth_facing;

        let lvl = settings.weapon_level(player.weapon, score.points);
        let ws = settings.weapon_at_level(player.weapon, lvl);
        let actual_range = match beam.beam_type {
            ConeBeamType::Flame => ws.range.max(100.0),
            ConeBeamType::Freeze => ws.range.max(120.0),
        };
        let range_scale = actual_range / match beam.beam_type {
            ConeBeamType::Flame => 130.0,
            ConeBeamType::Freeze => 150.0,
        };

        // Wie lange braucht der Strahl von Muendung bis max Range (bestimmt Nachzieh-Staerke)
        let travel_time = match beam.beam_type {
            ConeBeamType::Flame => 0.15,  // Feuer fliegt schnell
            ConeBeamType::Freeze => 0.20, // Eis etwas langsamer
        };

        // History initialisieren
        if !beam.history_initialized {
            beam.smooth_facing = facing;
            let facing_dir_init = facing;
            for frame in beam.history.iter_mut() {
                frame.tip = tip;
                frame.facing = facing_dir_init;
                frame.time = current_time;
            }
            beam.history_initialized = true;
        }

        // Aktuellen Frame in History schreiben
        let head = beam.history_head;
        beam.history[head] = EmissionFrame {
            tip, facing: facing_dir, time: current_time,
        };
        beam.history_head = (head + 1) % HISTORY_SIZE;

        // Chain-Positionen aus History berechnen:
        // Punkt i bei Distanz d nutzt die Emissionsdaten von vor (d/speed) Sekunden
        for i in 0..CHAIN_POINTS {
            let t = i as f32 / (CHAIN_POINTS - 1) as f32;
            let d = t * actual_range;
            let t_ago = t * travel_time;
            let (hist_tip, hist_facing) = lookup_history(
                &beam.history, beam.history_head, current_time, t_ago,
            );
            beam.chain_pos[i] = hist_tip + hist_facing * d;
        }
        beam.chain_initialized = true;

        // Mesh-Vertices aus Chain berechnen
        let half_width = match beam.beam_type {
            ConeBeamType::Flame => 60.0 * range_scale,
            ConeBeamType::Freeze => 85.0 * range_scale,
        };
        let positions = compute_beam_vertices(&beam.chain_pos, tip, half_width, facing_dir);
        if let Some(mesh) = meshes.get_mut(&mesh2d.0) {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        }

        // Transform: nur Position, keine Rotation/Skalierung
        transform.translation = Vec3::new(tip.x, tip.y, 15.0);
        transform.rotation = Quat::IDENTITY;
        transform.scale = Vec3::ONE;

        // Bewegungsgeschwindigkeit tracken
        let move_delta = (player_pos - beam.prev_pos).length();
        let move_vel = if dt > 0.0001 { move_delta / dt } else { 0.0 };
        beam.prev_pos = player_pos;
        let raw_move = (move_vel / 200.0).clamp(0.0, 1.0);
        let move_lerp = (6.0 * dt).min(1.0);
        beam.smooth_move_speed += (raw_move - beam.smooth_move_speed) * move_lerp;

        // Smooth intensity ramp
        let target = if is_active { 1.0_f32 } else { 0.0_f32 };
        let ramp_speed = if is_active { 7.0 } else { 4.0 };
        beam.current_intensity = beam.current_intensity + (target - beam.current_intensity) * (ramp_speed * dt).min(1.0);

        // Smooth hit distances
        let hit_lerp = (12.0 * dt).min(1.0);
        let fade_lerp = (6.0 * dt).min(1.0);
        let target_count = if is_active { beam.hit_count as f32 } else { 0.0 };
        beam.smooth_hit_count += (target_count - beam.smooth_hit_count) * hit_lerp;
        for i in 0..MAX_HITS {
            if (i as u32) < beam.hit_count && is_active {
                beam.smooth_hit_distances[i] += (beam.hit_distances[i] - beam.smooth_hit_distances[i]) * hit_lerp;
            } else {
                beam.smooth_hit_distances[i] += (1.0 - beam.smooth_hit_distances[i]) * fade_lerp;
            }
        }

        // Material updaten
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.params.time = time.elapsed_secs();
            material.params.intensity = beam.current_intensity;
            material.params.hit_count = beam.smooth_hit_count;
            material.params.move_speed = beam.smooth_move_speed;
            material.params.hit_distances = [
                Vec4::new(
                    beam.smooth_hit_distances[0], beam.smooth_hit_distances[1],
                    beam.smooth_hit_distances[2], beam.smooth_hit_distances[3],
                ),
                Vec4::new(
                    beam.smooth_hit_distances[4], beam.smooth_hit_distances[5],
                    beam.smooth_hit_distances[6], beam.smooth_hit_distances[7],
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
        // Damage nutzt die Richtung zum Chain-Mittelpunkt
        let visual_facing = if beam.chain_initialized {
            (beam.chain_pos[2] - tip).normalize_or(player.facing)
        } else {
            player.facing
        };

        let (range, half_angle) = match beam.beam_type {
            ConeBeamType::Flame => (ws.range.max(100.0), 0.35_f32),
            ConeBeamType::Freeze => (ws.range.max(120.0), 0.55_f32),
        };

        let mut hit_dists: Vec<f32> = Vec::new();

        if !wants_shoot || player.reloading || player.ammo == 0 {
            beam.hit_distances = [0.0; MAX_HITS];
            beam.hit_count = 0;
            continue;
        }

        for (zombie_entity, zt, mut health, mut zombie) in zombie_query.iter_mut() {
            let zombie_pos = zt.translation.truncate();
            let to_zombie = zombie_pos - tip;
            let dist = to_zombie.length();

            if dist > range || dist < 1.0 { continue; }

            let angle_to_zombie = to_zombie.normalize().dot(visual_facing).acos();
            if angle_to_zombie > half_angle { continue; }

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

        hit_dists.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        beam.hit_count = hit_dists.len().min(MAX_HITS) as u32;
        beam.hit_distances = [0.0; MAX_HITS];
        for (i, d) in hit_dists.iter().take(MAX_HITS).enumerate() {
            beam.hit_distances[i] = *d;
        }
    }
}

// ===================== Debug Gizmos =====================

pub fn cone_beam_debug_gizmos(
    mut gizmos: Gizmos,
    beam_query: Query<&ConeBeam>,
    player_query: Query<(&Player, &Transform, &Children), Without<ConeBeam>>,
    arm_query: Query<(&PlayerArm, &Transform), (Without<Player>, Without<ConeBeam>, Without<Zombie>)>,
    settings: Res<GameSettings>,
    score: Res<Score>,
) {
    for beam in beam_query.iter() {
        let Some((player, pt, children)) = player_query.iter().find(|(p, _, _)| p.id == beam.owner_id) else {
            continue;
        };

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

        let lvl = settings.weapon_level(player.weapon, score.points);
        let ws = settings.weapon_at_level(player.weapon, lvl);

        let visual_facing = if beam.chain_initialized {
            (beam.chain_pos[2] - tip).normalize_or(player.facing)
        } else {
            player.facing
        };

        let (range, half_angle) = match beam.beam_type {
            ConeBeamType::Flame => (ws.range.max(100.0), 0.35_f32),
            ConeBeamType::Freeze => (ws.range.max(120.0), 0.55_f32),
        };

        let tip3 = Vec3::new(tip.x, tip.y, 20.0);

        // Damage-Cone Kanten
        let angle = visual_facing.y.atan2(visual_facing.x);
        let left_dir = Vec2::new((angle + half_angle).cos(), (angle + half_angle).sin());
        let right_dir = Vec2::new((angle - half_angle).cos(), (angle - half_angle).sin());
        let left_end = Vec3::new(tip.x + left_dir.x * range, tip.y + left_dir.y * range, 20.0);
        let right_end = Vec3::new(tip.x + right_dir.x * range, tip.y + right_dir.y * range, 20.0);
        let center_end = Vec3::new(tip.x + visual_facing.x * range, tip.y + visual_facing.y * range, 20.0);

        gizmos.line(tip3, left_end, Color::srgba(0.0, 1.0, 0.0, 0.6));
        gizmos.line(tip3, right_end, Color::srgba(0.0, 1.0, 0.0, 0.6));
        gizmos.line(tip3, center_end, Color::srgba(1.0, 1.0, 0.0, 0.4));

        // Chain-Punkte
        for i in 0..CHAIN_POINTS {
            let p = Vec3::new(beam.chain_pos[i].x, beam.chain_pos[i].y, 20.0);
            gizmos.circle_2d(
                Isometry2d::from_translation(beam.chain_pos[i]),
                3.0,
                Color::srgba(1.0, 0.0, 0.0, 0.8),
            );
            if i > 0 {
                let prev = Vec3::new(beam.chain_pos[i - 1].x, beam.chain_pos[i - 1].y, 20.0);
                gizmos.line(prev, p, Color::srgba(1.0, 0.0, 0.0, 0.5));
            }
        }
    }
}
