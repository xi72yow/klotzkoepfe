use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::constants::*;

pub fn spawn_blood(commands: &mut Commands, position: Vec2) {
    let mut rng = rand::rng();

    for _ in 0..BLOOD_PARTICLES_PER_HIT {
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = rng.random_range(BLOOD_SPREAD_SPEED * 0.5..BLOOD_SPREAD_SPEED);

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

pub fn blood_update(
    time: Res<Time>,
    mut query: Query<(&mut BloodParticle, &mut Transform, &mut Velocity)>,
) {
    for (mut particle, mut transform, mut velocity) in query.iter_mut() {
        if !particle.on_ground {
            particle.lifetime.tick(time.delta());
            transform.translation.x += velocity.0.x * time.delta_secs();
            transform.translation.y += velocity.0.y * time.delta_secs();

            if particle.lifetime.is_finished() {
                particle.on_ground = true;
                velocity.0 = Vec2::ZERO;
            }
        }
    }
}
