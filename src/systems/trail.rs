use bevy::prelude::*;

use crate::components::*;

/// Haengt TrailEmitter automatisch an neue Projektile (Bullet, Rocket, Grenade, Boomerang)
pub fn trail_attach(
    mut commands: Commands,
    bullets: Query<(Entity, &BulletOwner), (Added<BulletOwner>, Without<TrailEmitter>, Without<MineEntity>)>,
    // WeaponType ermitteln wir ueber die Sprite-Farbe -> einfacher: Player-Query
    player_query: Query<&Player>,
) {
    for (entity, owner) in bullets.iter() {
        // Finde Waffe des Besitzers
        let weapon = player_query.iter()
            .find(|p| p.id == owner.0)
            .map(|p| p.weapon)
            .unwrap_or(WeaponType::Pistol);

        if let Some((size, interval, lifetime)) = weapon.trail_config() {
            // Trail-Farbe: Waffen-Farbe, leicht transparent
            let base = weapon.bullet_color();
            let [r, g, b, _] = base.to_srgba().to_f32_array();
            let color = Color::srgba(r, g, b, 0.6);

            commands.entity(entity).try_insert(TrailEmitter {
                color,
                size,
                spawn_timer: Timer::from_seconds(interval, TimerMode::Repeating),
                lifetime,
            });
        }
    }
}

pub fn trail_emit(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&Transform, &mut TrailEmitter)>,
) {
    for (transform, mut emitter) in query.iter_mut() {
        emitter.spawn_timer.tick(time.delta());
        for _ in 0..emitter.spawn_timer.times_finished_this_tick() {
            commands.spawn((
                Sprite {
                    color: emitter.color,
                    custom_size: Some(Vec2::splat(emitter.size)),
                    ..default()
                },
                Transform::from_translation(transform.translation.truncate().extend(1.0)),
                TrailParticle {
                    lifetime: Timer::from_seconds(emitter.lifetime, TimerMode::Once),
                    initial_size: emitter.size,
                },
            ));
        }
    }
}

pub fn trail_update(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut TrailParticle, &mut Sprite, &mut Transform)>,
) {
    for (entity, mut particle, mut sprite, mut transform) in query.iter_mut() {
        particle.lifetime.tick(time.delta());
        if particle.lifetime.is_finished() {
            commands.entity(entity).try_despawn();
            continue;
        }
        let frac = particle.lifetime.fraction();
        // Schrumpfen
        let size = particle.initial_size * (1.0 - frac);
        sprite.custom_size = Some(Vec2::splat(size));
        // Ausblenden
        let [r, g, b, _] = sprite.color.to_srgba().to_f32_array();
        sprite.color = Color::srgba(r, g, b, 0.6 * (1.0 - frac));
        // Hinter dem Projektil
        transform.translation.z = 1.0;
    }
}
