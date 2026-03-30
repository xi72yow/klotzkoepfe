use bevy::prelude::*;
use rand::{Rng, RngExt};

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
                legs_remaining: 2,
                arms_remaining: 2,
                crawl_transition: 0.0,
                fire_visual: 0.0,
                freeze_visual: 0.0,
                permanently_frozen: false,
            },
            Health { current: hp, max: hp },
            ZombieVariant(variant),
        ));

    if is_big {
        entity_cmds.insert(BigZombie);
    }

    let eye_color = Color::srgb(0.9, 0.1, 0.1);
    entity_cmds
        .with_children(|parent| {
            // Kopf
            parent.spawn((
                Sprite { color: head_color, custom_size: Some(head_size), ..default() },
                Transform::from_xyz(0.0, 8.0, 2.0),
                ZombieHead,
                OriginalColor(head_color),
            ));
            // Augen (rote Punkte)
            parent.spawn((
                Sprite { color: eye_color, custom_size: Some(Vec2::new(3.5, 3.5)), ..default() },
                Transform::from_xyz(-4.0, 10.0, 3.0),
                ZombieEye { side: -1.0 },
                OriginalColor(eye_color),
            ));
            parent.spawn((
                Sprite { color: eye_color, custom_size: Some(Vec2::new(3.5, 3.5)), ..default() },
                Transform::from_xyz(4.0, 10.0, 3.0),
                ZombieEye { side: 1.0 },
                OriginalColor(eye_color),
            ));
            // Koerper
            parent.spawn((
                Sprite { color: body_color, custom_size: Some(body_size), ..default() },
                Transform::from_xyz(0.0, -4.0, 1.0),
                ZombieBody,
                OriginalColor(body_color),
            ));
            // Linkes Bein
            parent.spawn((
                Sprite { color: leg_color, custom_size: Some(leg_size), ..default() },
                Transform::from_xyz(-5.0, -14.0, 0.5),
                ZombieLeg { side: -1.0 },
                OriginalColor(leg_color),
            ));
            // Rechtes Bein
            parent.spawn((
                Sprite { color: leg_color, custom_size: Some(leg_size), ..default() },
                Transform::from_xyz(5.0, -14.0, 0.5),
                ZombieLeg { side: 1.0 },
                OriginalColor(leg_color),
            ));
            // Linker Arm (ausgestreckt)
            parent.spawn((
                Sprite { color: arm_color, custom_size: Some(arm_size), ..default() },
                Transform::from_xyz(-9.5, -2.0, 0.5),
                ZombieArm { side: -1.0 },
                OriginalColor(arm_color),
            ));
            // Rechter Arm (ausgestreckt)
            parent.spawn((
                Sprite { color: arm_color, custom_size: Some(arm_size), ..default() },
                Transform::from_xyz(9.5, -2.0, 0.5),
                ZombieArm { side: 1.0 },
                OriginalColor(arm_color),
            ));
        });
}

pub fn zombie_ai(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut zombie_query: Query<(&Zombie, &mut Transform), (Without<Player>, Without<Knockback>)>,
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
            // Mobility: 2 Beine = 100%, 1 Bein = 50%, 0 Beine = kriechen mit Armen (15%)
            let mobility = match zombie.legs_remaining {
                2 => 1.0,
                1 => 0.5,
                _ => if zombie.arms_remaining > 0 { 0.15 } else { 0.0 },
            };
            let effective_speed = zombie.speed * zombie.speed_modifier * mobility;
            transform.translation.x += direction.x * effective_speed * time.delta_secs();
            transform.translation.y += direction.y * effective_speed * time.delta_secs();
        }
    }
}

pub fn zombie_animation(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut zombie_query: Query<(&mut Zombie, &mut Transform, &Children), Without<Player>>,
    mut leg_query: Query<(&ZombieLeg, &mut Transform), (Without<Zombie>, Without<ZombieArm>, Without<ZombieEye>, Without<ZombieHead>, Without<ZombieBody>, Without<Player>)>,
    mut arm_query: Query<(&ZombieArm, &mut Transform), (Without<Zombie>, Without<ZombieLeg>, Without<ZombieEye>, Without<ZombieHead>, Without<ZombieBody>, Without<Player>)>,
    mut eye_query: Query<(&ZombieEye, &mut Transform, &mut Sprite), (Without<Zombie>, Without<ZombieLeg>, Without<ZombieArm>, Without<ZombieHead>, Without<ZombieBody>, Without<Player>)>,
    mut head_query: Query<&mut Transform, (With<ZombieHead>, Without<Zombie>, Without<ZombieLeg>, Without<ZombieArm>, Without<ZombieEye>, Without<ZombieBody>, Without<Player>)>,
    mut body_query: Query<&mut Transform, (With<ZombieBody>, Without<Zombie>, Without<ZombieLeg>, Without<ZombieArm>, Without<ZombieEye>, Without<ZombieHead>, Without<Player>)>,
) {
    let player_positions: Vec<Vec2> = player_query
        .iter()
        .map(|t| t.translation.truncate())
        .collect();

    for (mut zombie, mut ztransform, children) in zombie_query.iter_mut() {
        let zombie_pos = ztransform.translation.truncate();
        let crawling = zombie.legs_remaining < 2;

        // Richtung zum naechsten Spieler
        let facing = if let Some(nearest) = player_positions.iter()
            .min_by(|a, b| a.distance(zombie_pos).partial_cmp(&b.distance(zombie_pos)).unwrap())
        {
            (*nearest - zombie_pos).normalize_or_zero()
        } else {
            Vec2::NEG_Y
        };

        // Umfall-Transition: smooth von 0 (stehend) zu 1 (liegend)
        let target_crawl = if crawling { 1.0 } else { 0.0 };
        let transition_speed = 4.0; // ~0.25s zum Umfallen
        if zombie.crawl_transition < target_crawl {
            zombie.crawl_transition = (zombie.crawl_transition + transition_speed * time.delta_secs()).min(1.0);
        } else if zombie.crawl_transition > target_crawl {
            zombie.crawl_transition = (zombie.crawl_transition - transition_speed * time.delta_secs()).max(0.0);
        }
        let ct = zombie.crawl_transition;

        // Root-Rotation: interpoliert zwischen aufrecht und liegend
        let facing_angle = facing.y.atan2(facing.x) - std::f32::consts::FRAC_PI_2;
        let crawl_rot = Quat::from_rotation_z(facing_angle);
        ztransform.rotation = Quat::IDENTITY.slerp(crawl_rot, ct);

        let t = time.elapsed_secs();
        let is_frozen = zombie.speed_modifier < 0.5;
        let is_solid_ice = zombie.speed_modifier <= 0.0;
        let anim_speed = if is_solid_ice { 0.0 } else if is_frozen { 0.3 } else { 1.0 };

        for child in children.iter() {
            // Bein-Animation
            if let Ok((leg, mut transform)) = leg_query.get_mut(child) {
                if !is_solid_ice {
                    if crawling {
                        let wiggle = (t * 6.0 + leg.side).sin() * 2.0;
                        transform.translation.x = 4.0 * leg.side + wiggle;
                        transform.translation.y = -13.0 + (t * 4.0).sin().abs() * 1.5;
                    } else {
                        let swing = (t * 4.0 * anim_speed + leg.side * std::f32::consts::PI).sin() * 2.5;
                        transform.translation.x = 4.0 * leg.side + facing.x * swing;
                        transform.translation.y = -13.0 + facing.y * swing;
                    }
                }
            }

            // Arm-Animation
            if let Ok((arm, mut transform)) = arm_query.get_mut(child) {
                if !is_solid_ice {
                    if crawling {
                        let phase = arm.side;
                        let crawl_cycle = (t * 3.0 + phase * std::f32::consts::PI).sin();
                        let reach = 8.0 + crawl_cycle * 5.0;
                        transform.translation.x = arm.side * 7.0;
                        transform.translation.y = reach;
                        transform.rotation = Quat::IDENTITY;
                    } else {
                        let wobble = (t * 3.0 + arm.side * 2.0).sin() * 0.1;
                        let base_x = arm.side * 9.5;
                        let base_y = -2.0;
                        let arm_reach = 6.0;
                        transform.translation.x = base_x + facing.x * arm_reach;
                        transform.translation.y = base_y + facing.y * arm_reach;
                        let angle = facing.y.atan2(facing.x);
                        transform.rotation = Quat::from_rotation_z(angle - std::f32::consts::FRAC_PI_2 + wobble);
                    }
                }
            }

            // Augen-Animation
            if let Ok((eye, mut transform, mut sprite)) = eye_query.get_mut(child) {
                if crawling {
                    sprite.custom_size = Some(Vec2::new(2.5, 2.5));
                    transform.translation.x = eye.side * 4.0;
                    transform.translation.y = 17.0;
                    transform.translation.z = 3.0;
                } else {
                    sprite.custom_size = Some(Vec2::new(3.5, 3.5));
                    transform.translation.x = eye.side * 4.0;
                    transform.translation.y = 10.0;
                    transform.translation.z = 3.0;
                }
            }

            // Kopf-Animation: leichtes Wippen und Drehen
            if let Ok(mut transform) = head_query.get_mut(child) {
                let bob = (t * 2.5 * anim_speed).sin() * 0.8;
                let sway = (t * 1.8 * anim_speed + 0.5).sin() * 0.6;
                let tilt = (t * 2.0 * anim_speed + 1.0).sin() * 0.04;
                transform.translation.x = sway;
                transform.translation.y = 8.0 + bob;
                transform.rotation = Quat::from_rotation_z(tilt);
            }

            // Koerper-Animation: leichtes Schwanken
            if let Ok(mut transform) = body_query.get_mut(child) {
                let sway = (t * 2.5 * anim_speed + 2.0).sin() * 0.4;
                let bob = (t * 2.5 * anim_speed).sin() * 0.3;
                let tilt = (t * 2.0 * anim_speed + 0.7).sin() * 0.025;
                transform.translation.x = sway;
                transform.translation.y = -4.0 + bob;
                transform.rotation = Quat::from_rotation_z(tilt);
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
    mut query: Query<(Entity, &Transform, &mut Health, &mut Burning, &mut Zombie)>,
    zombie_positions: Query<(Entity, &Transform), With<Zombie>>,
) {
    // Fire jumping
    let all_pos: Vec<(Entity, Vec2)> = zombie_positions.iter()
        .map(|(e, t)| (e, t.translation.truncate()))
        .collect();

    let mut new_burns: Vec<(Entity, f32)> = Vec::new();

    for (entity, transform, mut health, mut burning, mut zombie) in query.iter_mut() {
        burning.timer.tick(time.delta());
        burning.tick_timer.tick(time.delta());

        if burning.tick_timer.just_finished() {
            let tick_dmg = burning.damage_per_second * 0.25;
            health.current -= tick_dmg;
            zombie.fire_visual += tick_dmg * 3.0;
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
            commands.entity(entity).try_remove::<Burning>();
        }
    }

    // Apply fire jumping (deferred)
    for (e, dmg) in new_burns {
        if let Ok(mut ec) = commands.get_entity(e) {
            ec.try_insert(Burning {
                damage_per_second: dmg,
                timer: Timer::from_seconds(2.0, TimerMode::Once),
                tick_timer: Timer::from_seconds(0.25, TimerMode::Repeating),
            });
        }
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
            commands.entity(entity).try_remove::<Stunned>();
        }
    }
}

pub fn freeze_stack_system(
    time: Res<Time>,
    mut query: Query<(&mut FreezeStacks, &mut Zombie, &mut Sprite, &Health)>,
) {
    for (mut stacks, mut zombie, mut sprite, health) in query.iter_mut() {
        if stacks.frozen {
            stacks.frozen_timer.tick(time.delta());
            zombie.speed_modifier = 0.0;
            sprite.color = Color::NONE;
            if stacks.frozen_timer.is_finished() {
                stacks.frozen = false;
                stacks.hits = 0;
                // Nur auftauen wenn nicht permanent vereist
                let freeze_factor = (zombie.freeze_visual / (health.max * 0.5)).clamp(0.0, 1.0);
                if freeze_factor < 0.95 {
                    zombie.speed_modifier = 1.0;
                }
                sprite.color = Color::NONE;
            }
        }
    }
}

/// Frost taut ueber Zeit auf (freeze_visual zerfaellt), voll vereist = permanent
pub fn zombie_freeze_thaw(
    time: Res<Time>,
    mut query: Query<(&mut Zombie, &Health)>,
    mut sound_events: ResMut<super::audio::SoundQueue>,
) {
    let dt = time.delta_secs();
    for (mut zombie, health) in query.iter_mut() {
        if zombie.freeze_visual > 0.0 {
            let freeze_factor = (zombie.freeze_visual / (health.max * 0.5)).clamp(0.0, 1.0);

            // Voll vereist (>= 95%): permanenter Eisblock, kein Auftauen
            if freeze_factor >= 0.95 {
                if !zombie.permanently_frozen {
                    zombie.permanently_frozen = true;
                    sound_events.0.push(super::audio::SoundEvent::IceFreeze);
                }
                continue;
            }

            // Langsames Auftauen: 8% pro Sekunde
            zombie.freeze_visual = (zombie.freeze_visual - zombie.freeze_visual.max(0.5) * 0.08 * dt).max(0.0);
        }
    }
}

/// Visuelles Damage-System: Flammenwerfer verkohlt + schrumpft leicht, Eis faerbt blau + friert ein
pub fn zombie_elemental_visuals(
    mut zombie_query: Query<(
        &mut Zombie,
        &Health,
        &Children,
    )>,
    mut sprite_query: Query<(&mut Sprite, &mut Transform, &OriginalColor), (
        Without<Zombie>,
        Or<(
            With<ZombieHead>, With<ZombieBody>,
            With<ZombieArm>, With<ZombieLeg>,
            With<ZombieEye>,
        )>,
    )>,
) {
    for (mut zombie, health, children) in zombie_query.iter_mut() {
        // Feuer: burn_factor 0..1, voller Effekt bei 50% max HP Schaden
        let burn_factor = (zombie.fire_visual / (health.max * 0.5)).clamp(0.0, 1.0);

        // Eis: freeze_factor 0..1, voller Effekt bei 50% max HP Schaden
        let freeze_factor = (zombie.freeze_visual / (health.max * 0.5)).clamp(0.0, 1.0);

        // Speed direkt an freeze_factor koppeln: 0%=volle Speed, 80%+=eingefroren
        if freeze_factor > 0.8 {
            zombie.speed_modifier = 0.0;
        } else if freeze_factor > 0.01 {
            let slow = 1.0 - freeze_factor / 0.8; // linear: 100% bei 0, 0% bei 0.8
            zombie.speed_modifier = zombie.speed_modifier.min(slow);
        }

        // Verkohlt: Zombie wird langsamer (ab 30% bis 80% runter auf 20%)
        if burn_factor > 0.3 {
            let fire_slow = 1.0 - (burn_factor - 0.3).min(0.5) / 0.5 * 0.8; // 100% -> 20%
            if zombie.speed_modifier > fire_slow {
                zombie.speed_modifier = fire_slow;
            }
        }

        // Keine Effekte aktiv -> Farbe/Scale zuruecksetzen
        if burn_factor < 0.01 && freeze_factor < 0.01 {
            for child in children.iter() {
                if let Ok((mut sprite, mut transform, original)) = sprite_query.get_mut(child) {
                    sprite.color = original.0;
                    transform.scale.x = 1.0;
                    transform.scale.y = 1.0;
                }
            }
            zombie.speed_modifier = 1.0;
            continue;
        }

        // Scale-Faktoren berechnen (absolut, nicht kumulativ)
        let mut scale_x = 1.0f32;
        let mut scale_y = 1.0f32;

        if burn_factor > 0.0 {
            // Schrumpft bis 85%, dann Stopp
            let shrink_t = (burn_factor / 0.69).min(1.0);
            scale_x *= 1.0 - shrink_t * 0.15;
        }

        if freeze_factor > 0.0 {
            // Waechst bis 125% - deutlich sichtbar
            let grow = 1.0 + freeze_factor * 0.25;
            scale_x *= grow;
            scale_y *= grow;
        }

        for child in children.iter() {
            if let Ok((mut sprite, mut transform, original)) = sprite_query.get_mut(child) {
                let orig = original.0.to_srgba();

                let mut r = orig.red;
                let mut g = orig.green;
                let mut b = orig.blue;

                // Feuer: Erst nur leicht abdunkeln (Gamma), dann staerker abdunkeln
                // KEIN Rot-Shift - Originalfarbe bleibt erkennbar
                if burn_factor > 0.0 {
                    // Phase 1 (0-0.5): leichte Gamma-Abdunklung
                    // Phase 2 (0.5-1.0): staerkere Abdunklung
                    let gamma_boost = 1.0 + burn_factor * 0.8; // Gamma 1.0 -> 1.8
                    r = r.powf(gamma_boost);
                    g = g.powf(gamma_boost);
                    b = b.powf(gamma_boost);

                    // Ab 50% nochmal gleichmaessig abdunkeln
                    if burn_factor > 0.5 {
                        let extra_dark = 1.0 - (burn_factor - 0.5) * 0.6; // 1.0 -> 0.7
                        r *= extra_dark;
                        g *= extra_dark;
                        b *= extra_dark;
                    }
                }

                // Eis: Originalfarbe wird blauer/heller, am Ende fast weiss-blau
                if freeze_factor > 0.0 {
                    let t = freeze_factor;
                    // Stark nach weiss-blau verschieben
                    r = r * (1.0 - t * 0.7) + t * 0.6;
                    g = g * (1.0 - t * 0.5) + t * 0.75;
                    b = b * (1.0 - t * 0.2) + t * 0.95;
                }

                sprite.color = Color::srgb(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0));
                transform.scale.x = scale_x;
                transform.scale.y = scale_y;
            }
        }
    }
}

/// Markiert voll verkohlte Zombies zum Zerbroeseln
pub fn zombie_ash_death(
    mut commands: Commands,
    settings: Res<GameSettings>,
    mut score: ResMut<Score>,
    mut wave: ResMut<WaveState>,
    mut combo: ResMut<ComboMeter>,
    query: Query<(Entity, &Zombie, &Health), Without<AshCrumble>>,
    mut sound_events: ResMut<super::audio::SoundQueue>,
) {
    for (entity, zombie, health) in query.iter() {
        let burn_factor = (zombie.fire_visual / (health.max * 0.5)).clamp(0.0, 1.0);
        if burn_factor < 0.95 { continue; }

        wave.zombies_alive = wave.zombies_alive.saturating_sub(1);
        super::collision::register_kill(&mut score, &mut combo, &settings);
        sound_events.0.push(super::audio::SoundEvent::ZombieDeath);

        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.try_insert(AshCrumble {
                // Phase 1: 0.3s still stehen, Phase 2: 0.1s burst -> Partikel
                timer: Timer::from_seconds(0.4, TimerMode::Once),
                particle_timer: Timer::from_seconds(0.0, TimerMode::Once), // einmalig
                killed: true,
            });
            ec.try_remove::<Health>();
        }
    }
}

/// Zombie steht kurz still, dann zerplatzt er in viele Asche-Partikel die runterfallen
pub fn zombie_ash_crumble(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<GameSettings>,
    mut decal_map: ResMut<super::ground_decals::GroundDecalMap>,
    mut query: Query<(Entity, &mut AshCrumble, &mut Transform, &Children)>,
    sprite_query: Query<&Sprite, Without<AshParticle>>,
) {
    let mut rng = rand::rng();

    for (entity, mut crumble, transform, children) in query.iter_mut() {
        crumble.timer.tick(time.delta());
        let progress = crumble.timer.fraction(); // 0 -> 1

        // Phase 1 (0-0.75): still stehen, nichts passiert (Zombie ist schon geschrumpft/verkohlt)
        if progress < 0.75 {
            continue;
        }

        // Phase 2 (0.75): Burst! Zombie verschwindet, Partikel spawnen
        if !crumble.particle_timer.is_finished() {
            crumble.particle_timer.tick(time.delta() + std::time::Duration::from_secs(1)); // sofort fertig

            let pos = transform.translation.truncate();
            let scale = transform.scale.x.max(0.3);

            // Viele Partikel auf einmal - jedes Koerperteil wird zu Asche
            let particle_count = 20 + (scale * 10.0) as u32;
            for _ in 0..particle_count {
                let offset = Vec2::new(
                    rng.random_range(-12.0 * scale..12.0 * scale),
                    rng.random_range(-12.0 * scale..12.0 * scale),
                );
                let shade = rng.random_range(0.06..0.2);
                let size = rng.random_range(1.5..4.0);

                // Partikel fliegen auseinander und fallen dann runter (negatives Y = runter im Screen)
                let spread = Vec2::new(
                    rng.random_range(-30.0..30.0),
                    rng.random_range(-25.0..10.0), // meistens runter
                );

                commands.spawn((
                    Sprite {
                        color: Color::srgba(shade, shade * 0.85, shade * 0.7, 0.95),
                        custom_size: Some(Vec2::splat(size)),
                        ..default()
                    },
                    Transform::from_xyz(pos.x + offset.x, pos.y + offset.y, 14.0),
                    Velocity(spread),
                    AshParticle {
                        lifetime: Timer::from_seconds(rng.random_range(0.5..1.2), TimerMode::Once),
                    },
                ));
            }

            // Zombie sofort unsichtbar machen (Children Sprites)
            for child in children.iter() {
                if let Ok(sprite) = sprite_query.get(child) {
                    let _ = sprite; // nur um den Query zu nutzen
                    if let Ok(mut ec) = commands.get_entity(child) {
                        ec.try_insert(Visibility::Hidden);
                    }
                }
            }
        }

        // Nach Timer-Ende: Stamp + Despawn
        if crumble.timer.is_finished() {
            let pos = transform.translation.truncate();
            decal_map.pending_stamps.push(super::ground_decals::DecalStamp::Ash {
                position: pos,
                radius: 16.0,
            });
            commands.entity(entity).try_despawn();

            if rng.random::<f32>() < settings.crate_spawn_chance {
                super::crates::spawn_random_crate(&mut commands, pos, settings.crate_despawn_time);
            }
        }
    }
}

/// Asche-Partikel fallen runter, bremsen ab und verblassen
pub fn ash_particle_update(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut AshParticle, &mut Transform, &mut Velocity, &mut Sprite)>,
) {
    for (entity, mut particle, mut transform, mut velocity, mut sprite) in query.iter_mut() {
        particle.lifetime.tick(time.delta());
        let dt = time.delta_secs();

        // Schwerkraft: Partikel fallen nach unten
        velocity.0.y -= 40.0 * dt;
        // Luftwiderstand
        velocity.0 *= 1.0 - 2.0 * dt;

        transform.translation.x += velocity.0.x * dt;
        transform.translation.y += velocity.0.y * dt;

        let frac = particle.lifetime.fraction();

        // Partikel werden dunkler am Boden
        let c = sprite.color.to_srgba();
        let alpha = (1.0 - frac * 0.7) * 0.95;
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);

        if particle.lifetime.is_finished() {
            commands.entity(entity).try_despawn();
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
            commands.entity(entity).try_despawn();
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
