use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::{GameSettings, GameField};
use super::ground_decals::{DecalStamp, GroundDecalMap};
use super::explosion_fx::{self, ExplosionMaterial};
use rand::RngExt;

pub fn bullet_movement(
    mut commands: Commands,
    time: Res<Time>,
    field: Res<GameField>,
    mut query: Query<(Entity, &mut Transform, &Velocity, &mut Bullet)>,
) {
    let half_w = field.width / 2.0 - WALL_THICKNESS;
    let half_h = field.height / 2.0 - WALL_THICKNESS;

    for (entity, mut transform, velocity, mut bullet) in query.iter_mut() {
        let delta = time.delta_secs();
        let move_dist = velocity.0.length() * delta;
        transform.translation.x += velocity.0.x * delta;
        transform.translation.y += velocity.0.y * delta;

        bullet.range_remaining -= move_dist;

        let hit_wall = transform.translation.x.abs() > half_w
            || transform.translation.y.abs() > half_h;

        if bullet.range_remaining <= 0.0 || hit_wall {
            if hit_wall {
                let pos = transform.translation.truncate();
                spawn_wall_impact(&mut commands, pos, bullet.damage);
            }
            commands.entity(entity).try_despawn();
        }
    }
}

pub fn grenade_movement(
    mut commands: Commands,
    time: Res<Time>,
    _settings: Res<GameSettings>,
    field: Res<GameField>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut explosion_materials: ResMut<Assets<ExplosionMaterial>>,
    mut query: Query<(Entity, &mut Transform, &mut Velocity, &mut GrenadeProjectile)>,
    zombie_query: Query<&Transform, (With<Zombie>, Without<GrenadeProjectile>)>,
    mut sound_events: ResMut<super::audio::SoundQueue>,
) {
    let wall_min_x = -field.width / 2.0 + WALL_THICKNESS;
    let wall_max_x = field.width / 2.0 - WALL_THICKNESS;
    let wall_min_y = -field.height / 2.0 + WALL_THICKNESS;
    let wall_max_y = field.height / 2.0 - WALL_THICKNESS;

    let mut rng = rand::rng();

    for (entity, mut transform, mut velocity, mut grenade) in query.iter_mut() {
        let delta = time.delta_secs();
        transform.translation.x += velocity.0.x * delta;
        transform.translation.y += velocity.0.y * delta;

        grenade.fuse.tick(time.delta());

        // Wand-Abprall (immer)
        if transform.translation.x <= wall_min_x {
            transform.translation.x = wall_min_x;
            velocity.0.x = velocity.0.x.abs();
        } else if transform.translation.x >= wall_max_x {
            transform.translation.x = wall_max_x;
            velocity.0.x = -velocity.0.x.abs();
        }
        if transform.translation.y <= wall_min_y {
            transform.translation.y = wall_min_y;
            velocity.0.y = velocity.0.y.abs();
        } else if transform.translation.y >= wall_max_y {
            transform.translation.y = wall_max_y;
            velocity.0.y = -velocity.0.y.abs();
        }

        // Zombie-Abprall (Wahrscheinlichkeit steigt je naeher)
        let grenade_pos = transform.translation.truncate();
        let speed = velocity.0.length();
        if speed > 10.0 {
            for zombie_transform in zombie_query.iter() {
                let zombie_pos = zombie_transform.translation.truncate();
                let dist = grenade_pos.distance(zombie_pos);
                let bounce_radius = 30.0;
                if dist < bounce_radius {
                    // Chance: 100% bei dist=0, 0% bei dist=bounce_radius
                    let chance = 1.0 - (dist / bounce_radius);
                    if rng.random::<f32>() < chance * delta * 30.0 {
                        // Abprallen: Velocity weg vom Zombie reflektieren
                        let away = (grenade_pos - zombie_pos).normalize_or_zero();
                        velocity.0 = away * speed * 0.6;
                        break;
                    }
                }
            }
        }

        // Granate abbremsen und liegen bleiben
        let fuse_frac = grenade.fuse.fraction();
        velocity.0 *= 1.0 - (2.0 * delta).min(1.0) * fuse_frac;

        if grenade.fuse.is_finished() {
            let pos = transform.translation;
            let radius = grenade.explosion_radius;
            let level = grenade.level;
            spawn_explosion(&mut commands, &mut meshes, &mut explosion_materials, pos, radius, grenade.damage, level);
            spawn_shrapnel(&mut commands, pos.truncate(), grenade.damage, level);
            sound_events.0.push(super::audio::SoundEvent::Explosion(super::audio::ExplosionType::Grenade));
            commands.entity(entity).try_despawn();
        }
    }
}

pub fn rocket_movement(
    mut commands: Commands,
    time: Res<Time>,
    field: Res<GameField>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut explosion_materials: ResMut<Assets<ExplosionMaterial>>,
    mut query: Query<(Entity, &mut Transform, &Velocity, &mut RocketProjectile)>,
    zombie_query: Query<&Transform, (With<Zombie>, Without<RocketProjectile>)>,
    mut sound_events: ResMut<super::audio::SoundQueue>,
) {
    let half_w = field.width / 2.0 - WALL_THICKNESS;
    let half_h = field.height / 2.0 - WALL_THICKNESS;

    for (entity, mut transform, velocity, mut rocket) in query.iter_mut() {
        let delta = time.delta_secs();
        let move_dist = velocity.0.length() * delta;
        transform.translation.x += velocity.0.x * delta;
        transform.translation.y += velocity.0.y * delta;
        rocket.range_remaining -= move_dist;

        let rocket_pos = transform.translation.truncate();
        let mut explode = false;

        // Wand-Treffer
        if rocket_pos.x.abs() >= half_w || rocket_pos.y.abs() >= half_h {
            explode = true;
        }

        // Reichweite aufgebraucht
        if rocket.range_remaining <= 0.0 {
            explode = true;
        }

        // Zombie-Kontakt
        if !explode {
            for zombie_transform in zombie_query.iter() {
                let zombie_pos = zombie_transform.translation.truncate();
                if rocket_pos.distance(zombie_pos) < 20.0 {
                    explode = true;
                    break;
                }
            }
        }

        if explode {
            let pos = transform.translation;
            let radius = rocket.explosion_radius;
            let level = rocket.level;
            spawn_explosion(&mut commands, &mut meshes, &mut explosion_materials, pos, radius, rocket.damage, level);
            sound_events.0.push(super::audio::SoundEvent::Explosion(super::audio::ExplosionType::Rocket));
            commands.entity(entity).try_despawn();
        }
    }
}

/// Spawn an explosion - Shader fuer echte Explosionen (level > 0), Sprite fuer Tesla etc.
pub fn spawn_explosion(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ExplosionMaterial>,
    pos: Vec3,
    radius: f32,
    damage: f32,
    level: u32,
) {
    if level > 0 {
        // Shader-Explosion fuer Granaten, Raketen, Minen
        explosion_fx::spawn_shader_explosion(commands, meshes, materials, pos, radius, damage, level);
    } else {
        // Einfache Sprite-Explosion fuer Tesla-Chain etc.
        let lifetime = EXPLOSION_LIFETIME;
        commands.spawn((
            Sprite {
                color: EXPLOSION_COLOR,
                custom_size: Some(Vec2::splat(0.1)),
                ..default()
            },
            Transform::from_translation(pos.truncate().extend(15.0)),
            Explosion {
                lifetime: Timer::from_seconds(lifetime, TimerMode::Once),
                damage,
                radius,
                damaged: false,
                level,
            },
        ));
    }
}

pub fn explosion_update(
    mut commands: Commands,
    time: Res<Time>,
    mut decal_map: ResMut<GroundDecalMap>,
    mut query: Query<(Entity, &mut Explosion, &mut Sprite, &mut Transform, Option<&Children>)>,
    mut ring_query: Query<(&mut Sprite, &mut Transform), (With<ShockwaveRing>, Without<Explosion>)>,
) {
    for (entity, mut explosion, mut sprite, mut transform, children) in query.iter_mut() {
        explosion.lifetime.tick(time.delta());
        let frac = explosion.lifetime.fraction();
        let level_f = explosion.level.max(1) as f32;
        let radius = explosion.radius;

        // Phase 1: Rapid expansion (0..0.3)
        // Phase 2: Hold + color shift (0.3..0.6)
        // Phase 3: Fade out + shrink (0.6..1.0)
        let (size_mult, alpha, r, g, b) = if frac < 0.3 {
            // Expand quickly with bright white-yellow core
            let t = frac / 0.3;
            let ease = 1.0 - (1.0 - t) * (1.0 - t); // ease-out quad
            (ease, 1.0, 1.0, 0.8 + 0.2 * (1.0 - t), 0.3 + 0.4 * (1.0 - t))
        } else if frac < 0.6 {
            // Hold size, shift from yellow to orange
            let t = (frac - 0.3) / 0.3;
            (1.0, 1.0, 1.0, 0.5 + 0.3 * (1.0 - t), 0.1 * (1.0 - t))
        } else {
            // Fade out and shrink slightly
            let t = (frac - 0.6) / 0.4;
            let alpha = (1.0 - t * t).max(0.0);
            let shrink = 1.0 - 0.2 * t;
            (shrink, alpha, 1.0, 0.3 * (1.0 - t), 0.0)
        };

        let display_size = radius * 2.0 * size_mult;
        sprite.custom_size = Some(Vec2::splat(display_size));
        sprite.color = Color::srgba(r, g, b, alpha);

        // Slight screen-shake feel via scale pulse at higher levels
        let pulse = 1.0 + 0.05 * level_f * (frac * 30.0).sin() * (1.0 - frac);
        transform.scale = Vec3::splat(pulse);

        // Shockwave ring: expands outward, thins and fades
        if let Some(children) = children {
            for child in children.iter() {
                if let Ok((mut ring_sprite, mut ring_transform)) = ring_query.get_mut(child) {
                    let ring_expand = 1.0 + 0.8 * level_f;
                    let ring_size = radius * 2.0 * frac * ring_expand;
                    let ring_alpha = (1.0 - frac * frac) * 0.5;
                    let thickness = (4.0 + 2.0 * level_f) * (1.0 - frac);

                    ring_sprite.custom_size = Some(Vec2::new(ring_size, thickness.max(1.0)));
                    ring_sprite.color = Color::srgba(1.0, 0.6, 0.1, ring_alpha);
                    ring_transform.scale = Vec3::new(1.0, ring_size / thickness.max(1.0), 1.0);
                    ring_transform.rotation = Quat::from_rotation_z(frac * std::f32::consts::PI * 2.0);
                }
            }
        }

        if explosion.lifetime.is_finished() {
            // Brandfleck auf Boden stempeln
            if explosion.level > 0 {
                let pos = transform.translation.truncate();
                decal_map.pending_stamps.push(DecalStamp::Burn {
                    position: pos,
                    radius: radius,
                });
            }
            commands.entity(entity).try_despawn();
        }
    }
}

/// Spawnt Schrapnell-Splitter die von der Explosion wegfliegen und Damage machen
fn spawn_shrapnel(commands: &mut Commands, pos: Vec2, base_damage: f32, level: u32) {
    let mut rng = rand::rng();
    let count = 4 + level as i32 * 3 + rng.random_range(0..3);
    let shard_damage = base_damage * 0.3;
    let shard_speed = 250.0 + level as f32 * 50.0;
    let shard_range = 80.0 + level as f32 * 30.0;

    for _ in 0..count {
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = rng.random_range(shard_speed * 0.6..shard_speed);
        let dir = Vec2::new(angle.cos(), angle.sin());
        let size = Vec2::new(
            rng.random_range(2.0..4.0),
            rng.random_range(1.0..2.0),
        );

        commands.spawn((
            Sprite {
                color: Color::srgb(0.6, 0.6, 0.65),
                custom_size: Some(size),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 12.0)
                .with_rotation(Quat::from_rotation_z(angle)),
            Bullet {
                damage: shard_damage,
                range_remaining: rng.random_range(shard_range * 0.5..shard_range),
                pierce_remaining: 0,
            },
            Velocity(dir * speed),
        ));
    }
}

/// Spawnt Wand-Aufprall-Partikel. Anzahl und Groesse abhaengig vom Damage.
fn spawn_wall_impact(commands: &mut Commands, pos: Vec2, damage: f32) {
    let mut rng = rand::rng();
    let count = (damage * 0.4).clamp(3.0, 15.0) as u32;
    let base_size = (damage * 0.3).clamp(3.0, 10.0);
    let speed = (damage * 3.0).clamp(60.0, 300.0);

    for _ in 0..count {
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let spd = rng.random_range(speed * 0.4..speed);
        let dir = Vec2::new(angle.cos(), angle.sin());
        let size = rng.random_range(base_size * 0.5..base_size);
        let brightness = rng.random_range(0.5..0.9);

        commands.spawn((
            Sprite {
                color: Color::srgba(brightness, brightness * 0.9, brightness * 0.7, 0.9),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 12.0),
            Velocity(dir * spd),
            WallImpactParticle {
                lifetime: Timer::from_seconds(rng.random_range(0.1..0.25), TimerMode::Once),
                initial_size: size,
            },
        ));
    }
}

pub fn wall_impact_update(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut WallImpactParticle, &mut Sprite, &Velocity, &mut Transform)>,
) {
    for (entity, mut particle, mut sprite, vel, mut transform) in query.iter_mut() {
        particle.lifetime.tick(time.delta());
        if particle.lifetime.is_finished() {
            commands.entity(entity).try_despawn();
            continue;
        }
        let frac = particle.lifetime.fraction();
        let dt = time.delta_secs();
        transform.translation.x += vel.0.x * dt * (1.0 - frac);
        transform.translation.y += vel.0.y * dt * (1.0 - frac);
        let size = particle.initial_size * (1.0 - frac);
        sprite.custom_size = Some(Vec2::splat(size));
        let [r, g, b, _] = sprite.color.to_srgba().to_f32_array();
        sprite.color = Color::srgba(r, g, b, 0.9 * (1.0 - frac * frac));
    }
}
