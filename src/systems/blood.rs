use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::constants::*;
use super::ground_decals::{DecalStamp, GroundDecalMap};

pub fn spawn_blood(commands: &mut Commands, position: Vec2) {
    spawn_blood_with_settings(commands, position, BLOOD_PARTICLES_PER_HIT, BLOOD_SPREAD_SPEED);
}

pub fn spawn_blood_with_settings(commands: &mut Commands, position: Vec2, particle_count: u32, spread_speed: f32) {
    let mut rng = rand::rng();

    for _ in 0..particle_count {
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = rng.random_range(spread_speed * 0.5..spread_speed);

        // Farbe zwischen min und max interpolieren
        let t = rng.random_range(0.0_f32..1.0);
        let min = BLOOD_COLOR_MIN.to_srgba();
        let max = BLOOD_COLOR_MAX.to_srgba();
        let color = Color::srgb(
            min.red + (max.red - min.red) * t,
            min.green + (max.green - min.green) * t,
            min.blue + (max.blue - min.blue) * t,
        );

        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;

        commands.spawn((
            Sprite {
                color,
                custom_size: Some(BLOOD_PARTICLE_SIZE),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, 1.0),
            BloodParticle {
                lifetime: Timer::from_seconds(BLOOD_LIFETIME, TimerMode::Once),
                on_ground: false,
            },
            Velocity(velocity),
        ));
    }
}

/// Versucht ein zufaelliges Zombie-Teil abzureissen
pub fn try_dismember(
    commands: &mut Commands,
    zombie_entity: Entity,
    zombie: &mut Zombie,
    zombie_pos: Vec2,
    bullet_dir: Vec2,
    children: &Children,
    arm_query: &Query<(Entity, &ZombieArm, &Sprite, &Transform), Without<ZombieLeg>>,
    leg_query: &Query<(Entity, &ZombieLeg, &Sprite, &Transform), Without<ZombieArm>>,
    dismember_chance: f32,
    gib_decay_time: f32,
) {
    let mut rng = rand::rng();
    if !rng.random_bool(dismember_chance as f64) { return; }

    // Sammle abreissbare Teile: (entity, size, offset, color, is_leg)
    let mut candidates: Vec<(Entity, Vec2, Vec2, Color, bool)> = Vec::new();

    for child in children.iter() {
        if let Ok((e, _arm, sprite, t)) = arm_query.get(child) {
            let size = sprite.custom_size.unwrap_or(Vec2::new(5.0, 12.0));
            let color = sprite.color;
            let world_offset = Vec2::new(t.translation.x, t.translation.y);
            candidates.push((e, size, world_offset, color, false));
        }
        if let Ok((e, _leg, sprite, t)) = leg_query.get(child) {
            let size = sprite.custom_size.unwrap_or(Vec2::new(5.0, 8.0));
            let color = sprite.color;
            let world_offset = Vec2::new(t.translation.x, t.translation.y);
            candidates.push((e, size, world_offset, color, true));
        }
    }

    if candidates.is_empty() { return; }

    // Zufaelliges Teil auswaehlen
    let idx = rng.random_range(0..candidates.len());
    let (part_entity, size, offset, color, is_leg) = candidates[idx];

    // Mobility tracken
    if is_leg {
        zombie.legs_remaining = zombie.legs_remaining.saturating_sub(1);
    } else {
        zombie.arms_remaining = zombie.arms_remaining.saturating_sub(1);
    }

    // Teil vom Zombie entfernen
    commands.entity(part_entity).despawn();

    // Als freies Gib spawnen das wegfliegt
    let gib_pos = zombie_pos + offset;
    let fly_dir = if bullet_dir.length() > 0.1 {
        bullet_dir.normalize()
    } else {
        Vec2::new(rng.random_range(-1.0..1.0), rng.random_range(-1.0..1.0)).normalize_or_zero()
    };
    let speed = rng.random_range(80.0..200.0);
    let spin = rng.random_range(-8.0..8.0);

    commands.spawn((
        Sprite { color, custom_size: Some(size), ..default() },
        Transform::from_xyz(gib_pos.x, gib_pos.y, 8.0),
        Gib {
            lifetime: Timer::from_seconds(0.4, TimerMode::Once),
            on_ground: false,
            decay_timer: Timer::from_seconds(gib_decay_time, TimerMode::Once),
            original_size: size,
        },
        Velocity(fly_dir * speed),
        Spinning { speed: spin },
    ));

    // Etwas Blut an der Abriss-Stelle
    spawn_blood(commands, gib_pos);
}

/// Zombie explodiert: alle verbleibenden Teile als Gibs spawnen
pub fn zombie_explode(
    commands: &mut Commands,
    zombie_pos: Vec2,
    children: &Children,
    sprite_query: &Query<(&Sprite, &Transform), (Without<Zombie>, Without<Player>)>,
    gib_decay_time: f32,
) {
    let mut rng = rand::rng();

    for child in children.iter() {
        if let Ok((sprite, t)) = sprite_query.get(child) {
            let size = sprite.custom_size.unwrap_or(Vec2::splat(8.0));
            let color = sprite.color;
            // Unsichtbare Sprites (Root) ueberspringen
            if color == Color::NONE { continue; }

            let gib_pos = zombie_pos + Vec2::new(t.translation.x, t.translation.y);
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let speed = rng.random_range(60.0..180.0);
            let spin = rng.random_range(-10.0..10.0);

            commands.spawn((
                Sprite { color, custom_size: Some(size), ..default() },
                Transform::from_xyz(gib_pos.x, gib_pos.y, 8.0),
                Gib {
                    lifetime: Timer::from_seconds(0.5, TimerMode::Once),
                    on_ground: false,
                    decay_timer: Timer::from_seconds(gib_decay_time, TimerMode::Once),
                    original_size: size,
                },
                Velocity(Vec2::new(angle.cos(), angle.sin()) * speed),
                Spinning { speed: spin },
            ));
        }
    }

    spawn_blood(commands, zombie_pos);
}

pub fn gib_update(
    mut commands: Commands,
    time: Res<Time>,
    mut decal_map: ResMut<GroundDecalMap>,
    mut query: Query<(Entity, &mut Gib, &mut Transform, &mut Velocity, &mut Sprite, Option<&mut Spinning>)>,
) {
    for (entity, mut gib, mut transform, mut velocity, mut sprite, spinning) in query.iter_mut() {
        if !gib.on_ground {
            // Flugphase
            gib.lifetime.tick(time.delta());
            transform.translation.x += velocity.0.x * time.delta_secs();
            transform.translation.y += velocity.0.y * time.delta_secs();
            velocity.0 *= 1.0 - 4.0 * time.delta_secs();

            // Blutspur waehrend Flug: je nach Gib-Groesse sporadisch Troepfchen
            let gib_area = gib.original_size.x * gib.original_size.y;
            let speed = velocity.0.length();
            if speed > 30.0 {
                let mut rng = rand::rng();
                // Groessere Gibs tropfen oefter, kleine selten
                let drop_chance = (gib_area / 200.0).clamp(0.02, 0.3);
                if rng.random::<f32>() < drop_chance {
                    let pos = transform.translation.truncate();
                    // Leichter Offset fuer Spray-Effekt
                    let spray = Vec2::new(
                        rng.random_range(-3.0..3.0),
                        rng.random_range(-3.0..3.0),
                    );
                    let drop_radius = (gib_area / 80.0).clamp(0.5, 2.5);
                    decal_map.pending_stamps.push(DecalStamp::Blood {
                        position: pos + spray,
                        color: BLOOD_COLOR_MIN,
                        radius: drop_radius * rng.random_range(0.5..1.0),
                    });
                }
            }

            if gib.lifetime.is_finished() {
                gib.on_ground = true;
                velocity.0 = Vec2::ZERO;
                transform.rotation = Quat::IDENTITY;
                // Letzten Blutfleck stempeln, Groesse basiert auf Gib
                let pos = transform.translation.truncate();
                let drop_radius = (gib_area / 60.0).clamp(1.0, 3.5);
                decal_map.pending_stamps.push(DecalStamp::Blood {
                    position: pos,
                    color: BLOOD_COLOR_MAX,
                    radius: drop_radius,
                });
                // Rotation stoppen
                if let Some(mut spin) = spinning {
                    spin.speed = 0.0;
                }
            }
        } else {
            // Verrottungs-Phase: wird braun/dunkel, schrumpft, verschwindet
            gib.decay_timer.tick(time.delta());
            let decay = gib.decay_timer.fraction(); // 0.0 -> 1.0

            // Farbe: langsam zu dunkelbraun
            let orig = sprite.color.to_srgba();
            let brown_r = 0.2;
            let brown_g = 0.12;
            let brown_b = 0.08;
            let r = orig.red + (brown_r - orig.red) * decay;
            let g = orig.green + (brown_g - orig.green) * decay;
            let b = orig.blue + (brown_b - orig.blue) * decay;
            sprite.color = Color::srgba(r, g, b, 1.0 - decay * 0.5);

            // Schrumpfen
            let scale = 1.0 - decay * 0.8;
            let shrunk = gib.original_size * scale;
            sprite.custom_size = Some(shrunk);

            if gib.decay_timer.is_finished() {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn blood_update(
    mut commands: Commands,
    time: Res<Time>,
    mut decal_map: ResMut<GroundDecalMap>,
    mut query: Query<(Entity, &mut BloodParticle, &mut Transform, &mut Velocity, &Sprite)>,
) {
    for (entity, mut particle, mut transform, mut velocity, sprite) in query.iter_mut() {
        if !particle.on_ground {
            particle.lifetime.tick(time.delta());
            transform.translation.x += velocity.0.x * time.delta_secs();
            transform.translation.y += velocity.0.y * time.delta_secs();

            if particle.lifetime.is_finished() {
                // Auf Boden gelandet: Decal stempeln und Entity despawnen
                let pos = transform.translation.truncate();
                decal_map.pending_stamps.push(DecalStamp::Blood {
                    position: pos,
                    color: sprite.color,
                    radius: 3.0,
                });
                commands.entity(entity).despawn();
            }
        }
    }
}
