use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::GameSettings;

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
    settings: Res<GameSettings>,
    mut query: Query<(Entity, &mut Transform, &Velocity, &mut GrenadeProjectile)>,
    zombie_query: Query<&Transform, (With<Zombie>, Without<GrenadeProjectile>)>,
) {
    for (entity, mut transform, velocity, mut grenade) in query.iter_mut() {
        let delta = time.delta_secs();
        transform.translation.x += velocity.0.x * delta;
        transform.translation.y += velocity.0.y * delta;

        grenade.fuse.tick(time.delta());

        // Explodieren bei Zombie-Treffer oder Fuse abgelaufen
        let grenade_pos = transform.translation.truncate();
        let mut hit_zombie = false;
        for zombie_transform in zombie_query.iter() {
            let zombie_pos = zombie_transform.translation.truncate();
            if grenade_pos.distance(zombie_pos) < 20.0 {
                hit_zombie = true;
                break;
            }
        }

        if grenade.fuse.finished() || hit_zombie {
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

        if explosion.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}
