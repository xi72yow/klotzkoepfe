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
            commands.spawn((
                Sprite {
                    color: EXPLOSION_COLOR,
                    custom_size: Some(Vec2::new(radius * 2.0, radius * 2.0)),
                    ..default()
                },
                Transform::from_translation(pos),
                Explosion {
                    lifetime: Timer::from_seconds(EXPLOSION_LIFETIME, TimerMode::Once),
                    damage: grenade.damage,
                    radius,
                    damaged: false,
                },
            ));
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
            commands.spawn((
                Sprite {
                    color: EXPLOSION_COLOR,
                    custom_size: Some(Vec2::new(radius * 2.0, radius * 2.0)),
                    ..default()
                },
                Transform::from_translation(pos),
                Explosion {
                    lifetime: Timer::from_seconds(EXPLOSION_LIFETIME, TimerMode::Once),
                    damage: rocket.damage,
                    radius,
                    damaged: false,
                },
            ));
            commands.entity(entity).despawn();
        }
    }
}

pub fn explosion_update(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Explosion, &mut Sprite)>,
) {
    for (entity, mut explosion, mut sprite) in query.iter_mut() {
        explosion.lifetime.tick(time.delta());

        // Ausblenden
        let ratio = 1.0 - explosion.lifetime.fraction();
        let alpha = 1.0 - ratio;
        sprite.color = Color::srgba(1.0, 0.5 * alpha, 0.0, alpha);

        if explosion.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
