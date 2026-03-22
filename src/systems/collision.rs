use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::systems::blood::spawn_blood;
use crate::systems::crates::spawn_random_crate;
use rand::Rng;

pub fn apply_knockback(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Knockback, &mut Transform)>,
) {
    let half_w = crate::constants::WINDOW_WIDTH / 2.0 - crate::constants::WALL_THICKNESS;
    let half_h = crate::constants::WINDOW_HEIGHT / 2.0 - crate::constants::WALL_THICKNESS;

    for (entity, mut kb, mut transform) in query.iter_mut() {
        kb.duration.tick(time.delta());
        let frac = 1.0 - kb.duration.fraction();
        transform.translation.x += kb.velocity.x * frac * time.delta_secs();
        transform.translation.y += kb.velocity.y * frac * time.delta_secs();
        // Boundary clamp
        transform.translation.x = transform.translation.x.clamp(-half_w, half_w);
        transform.translation.y = transform.translation.y.clamp(-half_h, half_h);
        if kb.duration.is_finished() {
            commands.entity(entity).remove::<Knockback>();
        }
    }
}

fn spawn_lightning_arc(commands: &mut Commands, from: Vec2, to: Vec2) {
    let mut rng = rand::rng();
    let segments = 5;
    let dir = to - from;
    let step = dir / segments as f32;
    let perp = Vec2::new(-dir.y, dir.x).normalize_or_zero();

    for i in 0..segments {
        let base = from + step * (i as f32 + 0.5);
        let offset = perp * rng.random_range(-8.0..8.0);
        let pos = base + offset;
        let seg_len = step.length().max(2.0);

        commands.spawn((
            Sprite {
                color: Color::srgba(0.6, 0.7, 1.0, 0.9),
                custom_size: Some(Vec2::new(seg_len, 2.0)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 15.0)
                .with_rotation(Quat::from_rotation_z(dir.y.atan2(dir.x))),
            LightningArc {
                lifetime: Timer::from_seconds(0.3, TimerMode::Once),
            },
        ));
    }
}

pub fn register_kill(score: &mut Score, combo: &mut ComboMeter, settings: &GameSettings) {
    score.kills += 1;
    combo.position += settings.combo_kill_boost;

    // Score FIRST at current multiplier, THEN advance
    let base_score = 10_i32;
    score.points += base_score * combo.current_multiplier() as i32;

    // Multiplier streak (advances for NEXT kill)
    combo.kill_streak += 1;
    combo.streak_timer = Timer::from_seconds(settings.multiplier_kill_window, TimerMode::Once);

    // Advance multiplier tier - only one tier per kill max
    let kills_for_next = match combo.multiplier_index {
        0 => 5,    // 5 kills -> x2
        1 => 10,   // 10 more -> x5
        2 => 20,   // 20 more -> x10
        3 => 30,   // 30 more -> x25
        4 => 50,   // 50 more -> x50
        5 => 75,   // 75 more -> x100
        6 => 100,
        7 => 150,
        8 => 200,
        9 => 300,
        _ => 999,
    };
    if combo.kill_streak >= kills_for_next && combo.multiplier_index < ComboMeter::MULTIPLIER_TIERS.len() - 1 {
        combo.multiplier_index += 1;
        combo.kill_streak = 0;
    }
}

fn aabb_overlap(pos_a: Vec2, size_a: Vec2, pos_b: Vec2, size_b: Vec2) -> bool {
    let half_a = size_a / 2.0;
    let half_b = size_b / 2.0;
    (pos_a.x - half_a.x < pos_b.x + half_b.x)
        && (pos_a.x + half_a.x > pos_b.x - half_b.x)
        && (pos_a.y - half_a.y < pos_b.y + half_b.y)
        && (pos_a.y + half_a.y > pos_b.y - half_b.y)
}

pub fn bullet_zombie_collision(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut wave: ResMut<WaveState>,
    mut combo: ResMut<ComboMeter>,
    settings: Res<GameSettings>,
    mut bullet_query: Query<(Entity, &Transform, &mut Bullet, Option<&TeslaBullet>, Option<&FreezeBullet>, Option<&FlameBullet>)>,
    mut zombie_query: Query<(Entity, &Transform, &mut Health, &mut Zombie, Option<&Children>)>,
    zombie_arm_query: Query<(Entity, &ZombieArm, &Sprite, &Transform), Without<ZombieLeg>>,
    zombie_leg_query: Query<(Entity, &ZombieLeg, &Sprite, &Transform), Without<ZombieArm>>,
    sprite_query: Query<(&Sprite, &Transform), (Without<Zombie>, Without<Player>)>,
    mut sound_events: ResMut<super::audio::SoundQueue>,
) {
    // Sammle Zombie-Positionen fuer Tesla-Chain
    let zombie_positions: Vec<(Entity, Vec2)> = zombie_query.iter()
        .map(|(e, t, _, _, _)| (e, t.translation.truncate()))
        .collect();

    for (bullet_entity, bullet_transform, mut bullet, tesla, freeze, flame) in bullet_query.iter_mut() {
        let bullet_pos = bullet_transform.translation.truncate();

        for (zombie_entity, zombie_transform, mut health, mut zombie, children) in zombie_query.iter_mut() {
            let zombie_pos = zombie_transform.translation.truncate();

            if aabb_overlap(bullet_pos, Vec2::new(8.0, 8.0), zombie_pos, crate::constants::ZOMBIE_SIZE) {
                health.current -= bullet.damage;
                spawn_blood(&mut commands, bullet_pos);

                // Dismemberment bei Treffer (nicht-toedlich)
                if health.current > 0.0 {
                    if let Some(ch) = children {
                        let bullet_dir = (zombie_pos - bullet_pos).normalize_or_zero();
                        crate::systems::blood::try_dismember(
                            &mut commands, zombie_entity, zombie_pos, bullet_dir,
                            ch, &zombie_arm_query, &zombie_leg_query,
                            settings.dismember_chance, settings.gib_decay_time,
                        );
                    }
                }

                // Freeze-Effekt
                if let Some(fb) = freeze {
                    zombie.speed_modifier = fb.slow_factor;
                    zombie.freeze_timer = Timer::from_seconds(fb.slow_duration, TimerMode::Once);
                    // Add freeze stacks for full-freeze mechanic
                    commands.entity(zombie_entity).try_insert(FreezeStacks {
                        hits: 0, // Will be incremented below
                        frozen: false,
                        frozen_timer: Timer::from_seconds(fb.slow_duration * 2.0, TimerMode::Once),
                    });
                }

                // Flame-Effekt: set zombie on fire
                if flame.is_some() {
                    commands.entity(zombie_entity).try_insert(Burning {
                        damage_per_second: bullet.damage * 0.5,
                        timer: Timer::from_seconds(3.0, TimerMode::Once),
                        tick_timer: Timer::from_seconds(0.25, TimerMode::Repeating),
                    });
                }

                // Tesla stun on chain targets
                if tesla.is_some() {
                    commands.entity(zombie_entity).try_insert(Stunned {
                        timer: Timer::from_seconds(0.5, TimerMode::Once),
                    });
                }

                // Knockback on zombie
                if settings.knockback_strength_zombie > 0.0 && health.current > 0.0 {
                    let kb_dir = (zombie_pos - bullet_pos).normalize_or_zero();
                    commands.entity(zombie_entity).insert(Knockback {
                        velocity: kb_dir * settings.knockback_strength_zombie,
                        duration: Timer::from_seconds(settings.knockback_duration, TimerMode::Once),
                    });
                }

                if health.current <= 0.0 {
                    // Tesla-Chain: sammle Targets, Damage wird spaeter angewendet
                    if let Some(tb) = tesla {
                        let mut last_pos = zombie_pos;
                        let mut used: Vec<Entity> = vec![zombie_entity];

                        for _ in 0..tb.chain_count {
                            if let Some((next_e, next_pos)) = zombie_positions.iter()
                                .filter(|(e, _): &&(Entity, Vec2)| !used.contains(e))
                                .min_by(|(_, a): &&(Entity, Vec2), (_, b): &&(Entity, Vec2)| {
                                    a.distance(last_pos).partial_cmp(&b.distance(last_pos)).unwrap()
                                })
                            {
                                if next_pos.distance(last_pos) <= tb.chain_range {
                                    // Zigzag lightning arc visual
                                    spawn_lightning_arc(&mut commands, last_pos, *next_pos);

                                    // Stun chained zombies
                                    commands.entity(*next_e).try_insert(Stunned {
                                        timer: Timer::from_seconds(0.5, TimerMode::Once),
                                    });

                                    used.push(*next_e);
                                    last_pos = *next_pos;
                                }
                            }
                        }
                        // Chain-Damage via Explosion (simpel, kein zweites borrow)
                        for chain_e in used.iter().skip(1) {
                            if let Some((_, cpos)) = zombie_positions.iter().find(|(e, _)| e == chain_e) {
                                commands.spawn((
                                    Sprite { color: Color::srgba(0.3, 0.3, 1.0, 0.5), custom_size: Some(Vec2::splat(20.0)), ..default() },
                                    Transform::from_xyz(cpos.x, cpos.y, 12.0),
                                    Explosion {
                                        lifetime: Timer::from_seconds(0.1, TimerMode::Once),
                                        damage: tb.chain_damage,
                                        radius: 15.0,
                                        damaged: false,
                                        level: 0,
                                    },
                                ));
                            }
                        }
                    }

                    // Zombie explodiert: alle Teile als Gibs
                    if let Some(ch) = children {
                        crate::systems::blood::zombie_explode(
                            &mut commands, zombie_pos, ch, &sprite_query,
                            settings.gib_decay_time,
                        );
                    }
                    commands.entity(zombie_entity).try_despawn();
                    wave.zombies_alive = wave.zombies_alive.saturating_sub(1);
                    register_kill(&mut score, &mut combo, &settings);
                    sound_events.0.push(super::audio::SoundEvent::ZombieDeath);

                    // Red crate drop chance
                    if rand::rng().random::<f32>() < settings.crate_spawn_chance {
                        spawn_random_crate(&mut commands, zombie_pos, settings.crate_despawn_time);
                    }
                }

                bullet.pierce_remaining = bullet.pierce_remaining.saturating_sub(1);
                if bullet.pierce_remaining == 0 {
                    commands.entity(bullet_entity).despawn();
                    break;
                }
            }
        }
    }
}

pub fn explosion_zombie_collision(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut wave: ResMut<WaveState>,
    mut combo: ResMut<ComboMeter>,
    settings: Res<GameSettings>,
    mut explosion_query: Query<(&Transform, &mut Explosion)>,
    mut shader_explosion_query: Query<(&Transform, &mut ShaderExplosion), Without<Explosion>>,
    mut zombie_query: Query<(Entity, &Transform, &mut Health, Option<&Children>), With<Zombie>>,
    sprite_query: Query<(&Sprite, &Transform), (Without<Zombie>, Without<Player>)>,
    mut sound_events: ResMut<super::audio::SoundQueue>,
) {
    // Sammle alle Explosions-Daten (alte Sprite + neue Shader)
    let mut explosions: Vec<(Vec2, f32, f32)> = Vec::new();

    for (expl_transform, mut explosion) in explosion_query.iter_mut() {
        if explosion.damaged { continue; }
        explosion.damaged = true;
        explosions.push((expl_transform.translation.truncate(), explosion.radius, explosion.damage));
    }
    for (expl_transform, mut explosion) in shader_explosion_query.iter_mut() {
        if explosion.damaged { continue; }
        explosion.damaged = true;
        explosions.push((expl_transform.translation.truncate(), explosion.radius, explosion.damage));
    }

    for (expl_pos, radius, damage) in explosions {
        for (zombie_entity, zombie_transform, mut health, children) in zombie_query.iter_mut() {
            let zombie_pos = zombie_transform.translation.truncate();
            let dist = expl_pos.distance(zombie_pos);
            if dist < radius {
                let t = dist / radius;
                let falloff = (1.0 - t * t).max(0.0);
                health.current -= damage * falloff;
                spawn_blood(&mut commands, zombie_pos);
                if health.current <= 0.0 {
                    if let Some(ch) = children {
                        crate::systems::blood::zombie_explode(
                            &mut commands, zombie_pos, ch, &sprite_query,
                            settings.gib_decay_time,
                        );
                    }
                    commands.entity(zombie_entity).try_despawn();
                    wave.zombies_alive = wave.zombies_alive.saturating_sub(1);
                    register_kill(&mut score, &mut combo, &settings);
                    sound_events.0.push(super::audio::SoundEvent::ZombieDeath);
                    if rand::rng().random::<f32>() < settings.crate_spawn_chance {
                        spawn_random_crate(&mut commands, zombie_pos, settings.crate_despawn_time);
                    }
                }
            }
        }
    }
}

pub fn explosion_player_collision(
    mut commands: Commands,
    settings: Res<GameSettings>,
    mut wave: ResMut<WaveState>,
    mut next_state: ResMut<NextState<GameState>>,
    explosion_query: Query<(&Transform, &Explosion)>,
    shader_explosion_query: Query<(&Transform, &ShaderExplosion), Without<Explosion>>,
    mut player_query: Query<(Entity, &Player, &mut Health, &Transform, Option<&mut RegenCooldown>), (Without<Explosion>, Without<ShaderExplosion>)>,
) {
    if !settings.explosion_friendly_fire { return; }

    // Sammle alle Explosions-Daten die bereits Zombie-Damage gemacht haben
    let mut explosions: Vec<(Vec2, f32, f32)> = Vec::new();
    for (t, e) in explosion_query.iter() {
        if e.damaged {
            explosions.push((t.translation.truncate(), e.radius, e.damage));
        }
    }
    for (t, e) in shader_explosion_query.iter() {
        if e.damaged {
            explosions.push((t.translation.truncate(), e.radius, e.damage));
        }
    }

    for (expl_pos, radius, damage) in explosions {
        for (entity, player, mut health, player_transform, regen) in player_query.iter_mut() {
            let player_pos = player_transform.translation.truncate();
            let dist = expl_pos.distance(player_pos);
            if dist < radius {
                let t = dist / radius;
                let falloff = (1.0 - t * t).max(0.0);
                health.current -= damage * falloff;
                spawn_blood(&mut commands, player_pos);
                if let Some(mut regen) = regen {
                    regen.timer = Timer::from_seconds(settings.player_regen_delay.max(0.1), TimerMode::Once);
                }
                if health.current <= 0.0 {
                    if !wave.dead_players.contains(&player.id) {
                        wave.dead_players.push(player.id);
                    }
                    commands.entity(entity).try_despawn();
                }
            }
        }
    }
    if player_query.iter().count() == 0 {
        next_state.set(GameState::GameOver);
    }
}

pub fn bullet_player_collision(
    mut commands: Commands,
    settings: Res<GameSettings>,
    mut wave: ResMut<WaveState>,
    mut next_state: ResMut<NextState<GameState>>,
    bullet_query: Query<(Entity, &Transform, &Bullet, &BulletOwner)>,
    mut player_query: Query<(Entity, &Player, &mut Health, &Transform, Option<&mut RegenCooldown>), Without<Bullet>>,
) {
    if !settings.friendly_fire { return; }

    for (bullet_entity, bullet_transform, bullet, owner) in bullet_query.iter() {
        let bullet_pos = bullet_transform.translation.truncate();

        for (player_entity, player, mut health, player_transform, regen) in player_query.iter_mut() {
            // Eigene Bullets ignorieren
            if player.id == owner.0 { continue; }

            let player_pos = player_transform.translation.truncate();
            if aabb_overlap(bullet_pos, Vec2::new(8.0, 8.0), player_pos, crate::constants::PLAYER_SIZE) {
                health.current -= bullet.damage;
                spawn_blood(&mut commands, bullet_pos);
                commands.entity(bullet_entity).try_despawn();
                // Reset regen cooldown on damage
                if let Some(mut regen) = regen {
                    regen.timer = Timer::from_seconds(settings.player_regen_delay.max(0.1), TimerMode::Once);
                }

                if health.current <= 0.0 {
                    if !wave.dead_players.contains(&player.id) {
                        wave.dead_players.push(player.id);
                    }
                    commands.entity(player_entity).try_despawn();
                }
                break;
            }
        }
    }
    if player_query.iter().count() == 0 {
        next_state.set(GameState::GameOver);
    }
}

pub fn zombie_player_collision(
    time: Res<Time>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    settings: Res<GameSettings>,
    mut wave: ResMut<WaveState>,
    mut zombie_query: Query<(&mut Zombie, &Transform)>,
    mut player_query: Query<(Entity, &Player, &mut Health, &mut Transform, Option<&mut RegenCooldown>), Without<Zombie>>,
    mut sound_events: ResMut<super::audio::SoundQueue>,
) {
    use crate::constants::*;
    for (mut zombie, zombie_transform) in zombie_query.iter_mut() {
        let zombie_pos = zombie_transform.translation.truncate();
        zombie.damage_cooldown.tick(time.delta());
        for (entity, player, mut health, mut player_transform, regen) in player_query.iter_mut() {
            let player_pos = player_transform.translation.truncate();
            if aabb_overlap(player_pos, PLAYER_SIZE, zombie_pos, ZOMBIE_SIZE) {
                if zombie.damage_cooldown.is_finished() {
                    health.current -= settings.zombie_damage;
                    zombie.damage_cooldown.reset();
                    sound_events.0.push(super::audio::SoundEvent::PlayerDamage);
                    // Reset regen cooldown on damage
                    if let Some(mut regen) = regen {
                        regen.timer = Timer::from_seconds(settings.player_regen_delay.max(0.1), TimerMode::Once);
                    }
                    // Knockback on player
                    let diff = player_pos - zombie_pos;
                    if diff.length() > 0.1 {
                        commands.entity(entity).insert(Knockback {
                            velocity: diff.normalize() * settings.knockback_strength_player,
                            duration: Timer::from_seconds(settings.knockback_duration, TimerMode::Once),
                        });
                    }
                    if health.current <= 0.0 {
                        if !wave.dead_players.contains(&player.id) {
                            wave.dead_players.push(player.id);
                        }
                        commands.entity(entity).try_despawn();
                    }
                    break;
                }
            }
        }
    }
    if player_query.iter().count() == 0 {
        next_state.set(GameState::GameOver);
    }
}
