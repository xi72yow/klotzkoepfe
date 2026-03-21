use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;

pub fn zombie_spawn(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<GameSettings>,
    mut wave: ResMut<WaveState>,
) {
    if !wave.active || wave.zombies_to_spawn == 0 {
        return;
    }

    wave.spawn_timer.tick(time.delta());

    if wave.spawn_timer.just_finished() {
        let mut rng = rand::thread_rng();

        let half_w = WINDOW_WIDTH / 2.0 - WALL_THICKNESS - ZOMBIE_SIZE.x;
        let half_h = WINDOW_HEIGHT / 2.0 - WALL_THICKNESS - ZOMBIE_SIZE.y;

        let side = rng.gen_range(0..4);
        let pos = match side {
            0 => Vec2::new(rng.gen_range(-half_w..half_w), half_h),
            1 => Vec2::new(rng.gen_range(-half_w..half_w), -half_h),
            2 => Vec2::new(-half_w, rng.gen_range(-half_h..half_h)),
            _ => Vec2::new(half_w, rng.gen_range(-half_h..half_h)),
        };

        commands.spawn((
            Sprite {
                color: ZOMBIE_COLOR,
                custom_size: Some(ZOMBIE_SIZE),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 10.0),
            Zombie {
                speed: settings.zombie_speed,
                damage_cooldown: Timer::from_seconds(settings.zombie_damage_cooldown, TimerMode::Once),
                speed_modifier: 1.0,
                freeze_timer: Timer::from_seconds(0.0, TimerMode::Once),
            },
            Health {
                current: settings.zombie_hp,
                max: settings.zombie_hp,
            },
        ));

        wave.zombies_to_spawn -= 1;
        wave.zombies_alive += 1;
    }
}

pub fn zombie_ai(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut zombie_query: Query<(&Zombie, &mut Transform), Without<Player>>,
) {
    let player_positions: Vec<Vec2> = player_query
        .iter()
        .map(|t| t.translation.truncate())
        .collect();

    if player_positions.is_empty() {
        return;
    }

    for (zombie, mut transform) in zombie_query.iter_mut() {
        let zombie_pos = transform.translation.truncate();

        // Naechsten Spieler finden
        let nearest = player_positions
            .iter()
            .min_by(|a, b| {
                a.distance(zombie_pos)
                    .partial_cmp(&b.distance(zombie_pos))
                    .unwrap()
            })
            .unwrap();

        let diff = *nearest - zombie_pos;

        if diff.length() > 1.0 {
            let direction = diff.normalize();
            let effective_speed = zombie.speed * zombie.speed_modifier;
            transform.translation.x += direction.x * effective_speed * time.delta_secs();
            transform.translation.y += direction.y * effective_speed * time.delta_secs();

            let angle = direction.y.atan2(direction.x);
            transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}

pub fn zombie_separation(
    mut query: Query<(Entity, &mut Transform), With<Zombie>>,
) {
    let positions: Vec<(Entity, Vec2)> = query
        .iter()
        .map(|(e, t)| (e, t.translation.truncate()))
        .collect();

    for (entity, mut transform) in query.iter_mut() {
        let pos = transform.translation.truncate();
        let mut push = Vec2::ZERO;

        for (other_entity, other_pos) in &positions {
            if entity == *other_entity {
                continue;
            }
            let diff = pos - *other_pos;
            let dist = diff.length();
            if dist < 30.0 && dist > 0.01 {
                push += diff.normalize() * (30.0 - dist) * 0.5;
            }
        }

        transform.translation.x += push.x;
        transform.translation.y += push.y;
    }
}
