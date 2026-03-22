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
        let mut rng = rand::rng();

        let half_w = WINDOW_WIDTH / 2.0 - WALL_THICKNESS - ZOMBIE_SIZE.x;
        let half_h = WINDOW_HEIGHT / 2.0 - WALL_THICKNESS - ZOMBIE_SIZE.y;

        let side = rng.random_range(0..4);
        let pos = match side {
            0 => Vec2::new(rng.random_range(-half_w..half_w), half_h),
            1 => Vec2::new(rng.random_range(-half_w..half_w), -half_h),
            2 => Vec2::new(-half_w, rng.random_range(-half_h..half_h)),
            _ => Vec2::new(half_w, rng.random_range(-half_h..half_h)),
        };

        let variant = rng.random_range(0u8..3);
        let is_big = wave.current_wave >= settings.big_zombie_start_wave
            && rng.random::<f32>() < settings.big_zombie_spawn_chance;
        spawn_zombie(&mut commands, pos, &settings, variant, is_big, &mut rng);

        wave.zombies_to_spawn -= 1;
        wave.zombies_alive += 1;
    }
}

fn spawn_zombie(commands: &mut Commands, pos: Vec2, settings: &GameSettings, variant: u8, is_big: bool, rng: &mut impl Rng) {
    // 3 Zombie-Designs
    let d = if is_big { 0.7 } else { 1.0 };
    let (head_color, body_color, arm_color, leg_color) = match variant {
        // Typ 0: Klassisch gruen (frischer Zombie)
        0 => (
            Color::srgb(0.4 * d, 0.6 * d, 0.3 * d),
            Color::srgb(0.3 * d, 0.5 * d, 0.25 * d),
            Color::srgb(0.45 * d, 0.55 * d, 0.3 * d),
            Color::srgb(0.3 * d, 0.4 * d, 0.2 * d),
        ),
        // Typ 1: Grau/blau (verwester Zombie)
        1 => (
            Color::srgb(0.5 * d, 0.5 * d, 0.6 * d),
            Color::srgb(0.35 * d, 0.35 * d, 0.45 * d),
            Color::srgb(0.45 * d, 0.45 * d, 0.55 * d),
            Color::srgb(0.3 * d, 0.3 * d, 0.4 * d),
        ),
        // Typ 2: Rot/braun (blutiger Zombie)
        _ => (
            Color::srgb(0.6 * d, 0.3 * d, 0.25 * d),
            Color::srgb(0.5 * d, 0.2 * d, 0.15 * d),
            Color::srgb(0.55 * d, 0.25 * d, 0.2 * d),
            Color::srgb(0.4 * d, 0.2 * d, 0.15 * d),
        ),
    };

    let scale = if is_big { settings.big_zombie_scale } else { 1.0 };
    let (hp, speed, damage) = if is_big {
        (settings.big_zombie_hp, settings.big_zombie_speed, settings.big_zombie_damage)
    } else {
        (settings.zombie_hp, settings.zombie_speed, settings.zombie_damage)
    };

    // Gleiche Proportionen wie Spieler
    let head_size = Vec2::new(18.0, 18.0);
    let body_size = Vec2::new(14.0, 12.0);
    let leg_size = Vec2::new(5.0, 8.0);
    let arm_size = Vec2::new(5.0, 12.0);
    let collision_size = PLAYER_SIZE * scale;

    let mut entity_cmds = commands
        .spawn((
            // Unsichtbarer Root
            Sprite { color: Color::NONE, custom_size: Some(collision_size), ..default() },
            Transform::from_xyz(pos.x, pos.y, 9.0).with_scale(Vec3::splat(scale)),
            Zombie {
                speed,
                damage_cooldown: Timer::from_seconds(settings.zombie_damage_cooldown, TimerMode::Once),
                speed_modifier: 1.0,
                freeze_timer: Timer::from_seconds(0.0, TimerMode::Once),
                groan_timer: Timer::from_seconds(
                    rng.random_range(5.0..20.0),
                    TimerMode::Once,
                ),
            },
            Health { current: hp, max: hp },
            ZombieVariant(variant),
        ));

    if is_big {
        entity_cmds.insert(BigZombie);
    }

    entity_cmds
        .with_children(|parent| {
            // Kopf
            parent.spawn((
                Sprite { color: head_color, custom_size: Some(head_size), ..default() },
                Transform::from_xyz(0.0, 8.0, 2.0),
            ));
            // Augen (rote Punkte)
            parent.spawn((
                Sprite { color: Color::srgb(0.9, 0.1, 0.1), custom_size: Some(Vec2::new(3.5, 3.5)), ..default() },
                Transform::from_xyz(-4.0, 10.0, 3.0),
            ));
            parent.spawn((
                Sprite { color: Color::srgb(0.9, 0.1, 0.1), custom_size: Some(Vec2::new(3.5, 3.5)), ..default() },
                Transform::from_xyz(4.0, 10.0, 3.0),
            ));
            // Koerper
            parent.spawn((
                Sprite { color: body_color, custom_size: Some(body_size), ..default() },
                Transform::from_xyz(0.0, -4.0, 1.0),
            ));
            // Linkes Bein
            parent.spawn((
                Sprite { color: leg_color, custom_size: Some(leg_size), ..default() },
                Transform::from_xyz(-5.0, -14.0, 0.5),
                ZombieLeg { side: -1.0 },
            ));
            // Rechtes Bein
            parent.spawn((
                Sprite { color: leg_color, custom_size: Some(leg_size), ..default() },
                Transform::from_xyz(5.0, -14.0, 0.5),
                ZombieLeg { side: 1.0 },
            ));
            // Linker Arm (ausgestreckt)
            parent.spawn((
                Sprite { color: arm_color, custom_size: Some(arm_size), ..default() },
                Transform::from_xyz(-9.5, -2.0, 0.5),
                ZombieArm { side: -1.0 },
            ));
            // Rechter Arm (ausgestreckt)
            parent.spawn((
                Sprite { color: arm_color, custom_size: Some(arm_size), ..default() },
                Transform::from_xyz(9.5, -2.0, 0.5),
                ZombieArm { side: 1.0 },
            ));
        });
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
        // Skip if speed is 0 (stunned/frozen)
        if zombie.speed_modifier <= 0.0 { continue; }
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
        }
    }
}

pub fn zombie_animation(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    zombie_query: Query<(&Zombie, &Transform, &Children), Without<Player>>,
    mut leg_query: Query<(&ZombieLeg, &mut Transform), (Without<Zombie>, Without<ZombieArm>, Without<Player>)>,
    mut arm_query: Query<(&ZombieArm, &mut Transform), (Without<Zombie>, Without<ZombieLeg>, Without<Player>)>,
) {
    let player_positions: Vec<Vec2> = player_query
        .iter()
        .map(|t| t.translation.truncate())
        .collect();

    for (zombie, ztransform, children) in zombie_query.iter() {
        let zombie_pos = ztransform.translation.truncate();

        // Richtung zum naechsten Spieler
        let facing = if let Some(nearest) = player_positions.iter()
            .min_by(|a, b| a.distance(zombie_pos).partial_cmp(&b.distance(zombie_pos)).unwrap())
        {
            (*nearest - zombie_pos).normalize_or_zero()
        } else {
            Vec2::NEG_Y
        };

        let t = time.elapsed_secs();
        let is_frozen = zombie.speed_modifier < 0.5;

        for child in children.iter() {
            // Bein-Animation: langsames Schlurfen in Laufrichtung
            if let Ok((leg, mut transform)) = leg_query.get_mut(child) {
                let speed = if is_frozen { 0.2 } else { 1.0 };
                let swing = (t * 4.0 * speed + leg.side * std::f32::consts::PI).sin() * 2.5;
                transform.translation.x = 4.0 * leg.side + facing.x * swing;
                transform.translation.y = -13.0 + facing.y * swing;
            }

            // Arm-Animation: ausgestreckt zum Spieler, leichtes Zittern
            if let Ok((arm, mut transform)) = arm_query.get_mut(child) {
                let wobble = (t * 3.0 + arm.side * 2.0).sin() * 0.1;
                // Arm-Basis am Koerper
                let base_x = arm.side * 9.5;
                let base_y = -2.0;
                // Arm zeigt in Richtung Spieler (Offset vom Koerper weg)
                let arm_reach = 6.0;
                transform.translation.x = base_x + facing.x * arm_reach;
                transform.translation.y = base_y + facing.y * arm_reach;
                // Arm rotieren zum Spieler
                let angle = facing.y.atan2(facing.x);
                transform.rotation = Quat::from_rotation_z(angle + wobble);
            }
        }
    }
}

pub fn zombie_groan(
    time: Res<Time>,
    mut query: Query<&mut Zombie>,
    mut sound_events: ResMut<super::audio::SoundQueue>,
) {
    let mut rng = rand::rng();
    for mut zombie in query.iter_mut() {
        zombie.groan_timer.tick(time.delta());
        if zombie.groan_timer.just_finished() {
            // Nur 30% Chance tatsaechlich zu groanen
            if rng.random::<f32>() < 0.3 {
                let variant = rng.random_range(0u8..3);
                sound_events.0.push(super::audio::SoundEvent::ZombieGroan(variant));
            }
            zombie.groan_timer = Timer::from_seconds(
                rng.random_range(10.0..30.0),
                TimerMode::Once,
            );
        }
    }
}

pub fn burning_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<GameSettings>,
    mut score: ResMut<Score>,
    mut wave: ResMut<WaveState>,
    mut combo: ResMut<ComboMeter>,
    mut query: Query<(Entity, &Transform, &mut Health, &mut Burning)>,
    zombie_positions: Query<(Entity, &Transform), With<Zombie>>,
) {
    // Fire jumping
    let all_pos: Vec<(Entity, Vec2)> = zombie_positions.iter()
        .map(|(e, t)| (e, t.translation.truncate()))
        .collect();

    let mut new_burns: Vec<(Entity, f32)> = Vec::new();

    for (entity, transform, mut health, mut burning) in query.iter_mut() {
        burning.timer.tick(time.delta());
        burning.tick_timer.tick(time.delta());

        if burning.tick_timer.just_finished() {
            health.current -= burning.damage_per_second * 0.25; // Tick every 0.25s
            crate::systems::blood::spawn_blood(&mut commands, transform.translation.truncate());

            // Fire jump to nearby zombies
            let pos = transform.translation.truncate();
            for (other_e, other_pos) in &all_pos {
                if *other_e != entity && pos.distance(*other_pos) < 50.0 {
                    new_burns.push((*other_e, burning.damage_per_second * 0.5));
                }
            }
        }

        if health.current <= 0.0 {
            commands.entity(entity).try_despawn();
            wave.zombies_alive = wave.zombies_alive.saturating_sub(1);
            crate::systems::collision::register_kill(&mut score, &mut combo, &settings);
        }

        if burning.timer.is_finished() {
            commands.entity(entity).remove::<Burning>();
        }
    }

    // Apply fire jumping (deferred)
    for (e, dmg) in new_burns {
        commands.entity(e).try_insert(Burning {
            damage_per_second: dmg,
            timer: Timer::from_seconds(2.0, TimerMode::Once),
            tick_timer: Timer::from_seconds(0.25, TimerMode::Repeating),
        });
    }
}

pub fn stun_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Stunned, &mut Zombie)>,
) {
    for (entity, mut stunned, mut zombie) in query.iter_mut() {
        stunned.timer.tick(time.delta());
        zombie.speed_modifier = 0.0;
        if stunned.timer.is_finished() {
            if zombie.freeze_timer.is_finished() {
                zombie.speed_modifier = 1.0;
            }
            commands.entity(entity).remove::<Stunned>();
        }
    }
}

pub fn freeze_stack_system(
    time: Res<Time>,
    mut query: Query<(&mut FreezeStacks, &mut Zombie, &mut Sprite)>,
) {
    for (mut stacks, mut zombie, mut sprite) in query.iter_mut() {
        if stacks.frozen {
            stacks.frozen_timer.tick(time.delta());
            zombie.speed_modifier = 0.0;
            // Tint blue
            sprite.color = Color::srgb(0.5, 0.8, 1.0);
            if stacks.frozen_timer.is_finished() {
                stacks.frozen = false;
                stacks.hits = 0;
                zombie.speed_modifier = 1.0;
                sprite.color = Color::NONE; // Root sprite is invisible
            }
        }
    }
}

pub fn lightning_arc_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut LightningArc, &mut Sprite)>,
) {
    for (entity, mut arc, mut sprite) in query.iter_mut() {
        arc.lifetime.tick(time.delta());
        let alpha = 1.0 - arc.lifetime.fraction();
        sprite.color = Color::srgba(0.6, 0.7, 1.0, alpha);
        if arc.lifetime.is_finished() {
            commands.entity(entity).despawn();
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
