use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::GameSettings;
use rand::Rng;

pub fn bullet_movement(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &Velocity, &mut Bullet)>,
) {
    let half_w = WINDOW_WIDTH / 2.0;
    let half_h = WINDOW_HEIGHT / 2.0;

    for (entity, mut transform, velocity, mut bullet) in query.iter_mut() {
        let delta = time.delta_secs();
        let move_dist = velocity.0.length() * delta;
        transform.translation.x += velocity.0.x * delta;
        transform.translation.y += velocity.0.y * delta;

        bullet.range_remaining -= move_dist;

        if bullet.range_remaining <= 0.0
            || transform.translation.x.abs() > half_w
            || transform.translation.y.abs() > half_h
        {
            commands.entity(entity).despawn();
        }
    }
}

pub fn grenade_movement(
    mut commands: Commands,
    time: Res<Time>,
    _settings: Res<GameSettings>,
    mut query: Query<(Entity, &mut Transform, &mut Velocity, &mut GrenadeProjectile)>,
    zombie_query: Query<&Transform, (With<Zombie>, Without<GrenadeProjectile>)>,
) {
    let wall_min_x = -WINDOW_WIDTH / 2.0 + WALL_THICKNESS;
    let wall_max_x = WINDOW_WIDTH / 2.0 - WALL_THICKNESS;
    let wall_min_y = -WINDOW_HEIGHT / 2.0 + WALL_THICKNESS;
    let wall_max_y = WINDOW_HEIGHT / 2.0 - WALL_THICKNESS;

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
            spawn_explosion(&mut commands, pos, radius, grenade.damage, level);
            commands.entity(entity).despawn();
        }
    }
}

pub fn rocket_movement(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &Velocity, &mut RocketProjectile)>,
    zombie_query: Query<&Transform, (With<Zombie>, Without<RocketProjectile>)>,
) {
    let half_w = WINDOW_WIDTH / 2.0 - WALL_THICKNESS;
    let half_h = WINDOW_HEIGHT / 2.0 - WALL_THICKNESS;

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
            spawn_explosion(&mut commands, pos, radius, rocket.damage, level);
            commands.entity(entity).despawn();
        }
    }
}

/// Spawn an explosion with visual effects scaled by level
pub fn spawn_explosion(commands: &mut Commands, pos: Vec3, radius: f32, damage: f32, level: u32) {
    let level_f = level.max(1) as f32;
    // Higher levels get longer lifetime for more dramatic effect
    let lifetime = EXPLOSION_LIFETIME + 0.1 * (level_f - 1.0);

    // Core explosion (starts small, expands)
    commands
        .spawn((
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
        ))
        .with_children(|parent| {
            if level > 0 {
                // Shockwave ring
                parent.spawn((
                    Sprite {
                        color: Color::srgba(1.0, 0.8, 0.3, 0.6),
                        custom_size: Some(Vec2::splat(0.1)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 1.0),
                    ShockwaveRing,
                ));
            }
        });
}

pub fn explosion_update(
    mut commands: Commands,
    time: Res<Time>,
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
            commands.entity(entity).try_despawn();
        }
    }
}

