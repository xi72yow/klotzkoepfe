use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::systems::blood::spawn_blood;
use crate::systems::weapons::spawn_drop;
use rand::Rng;

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
    mut bullet_query: Query<(Entity, &Transform, &mut Bullet, Option<&TeslaBullet>, Option<&FreezeBullet>)>,
    mut zombie_query: Query<(Entity, &Transform, &mut Health, &mut Zombie)>,
) {
    // Sammle Zombie-Positionen fuer Tesla-Chain
    let zombie_positions: Vec<(Entity, Vec2)> = zombie_query.iter()
        .map(|(e, t, _, _)| (e, t.translation.truncate()))
        .collect();

    for (bullet_entity, bullet_transform, mut bullet, tesla, freeze) in bullet_query.iter_mut() {
        let bullet_pos = bullet_transform.translation.truncate();

        for (zombie_entity, zombie_transform, mut health, mut zombie) in zombie_query.iter_mut() {
            let zombie_pos = zombie_transform.translation.truncate();

            if aabb_overlap(bullet_pos, Vec2::new(8.0, 8.0), zombie_pos, crate::constants::ZOMBIE_SIZE) {
                health.current -= bullet.damage;
                spawn_blood(&mut commands, bullet_pos);

                // Freeze-Effekt
                if let Some(fb) = freeze {
                    zombie.speed_modifier = fb.slow_factor;
                    zombie.freeze_timer = Timer::from_seconds(fb.slow_duration, TimerMode::Once);
                }

                if health.current <= 0.0 {
                    // Tesla-Chain: sammle Targets, Damage wird spaeter angewendet
                    if let Some(tb) = tesla {
                        let mut last_pos = zombie_pos;
                        let mut used: Vec<Entity> = vec![zombie_entity];

                        for _ in 0..tb.chain_count {
                            if let Some((next_e, next_pos)) = zombie_positions.iter()
                                .filter(|(e, _)| !used.contains(e))
                                .min_by(|(_, a), (_, b)| {
                                    a.distance(last_pos).partial_cmp(&b.distance(last_pos)).unwrap()
                                })
                            {
                                if next_pos.distance(last_pos) <= tb.chain_range {
                                    used.push(*next_e);
                                    last_pos = *next_pos;
                                    // Blitz-Visualisierung
                                    commands.spawn((
                                        Sprite { color: Color::srgb(0.5, 0.5, 1.0), custom_size: Some(Vec2::new(6.0, 6.0)), ..default() },
                                        Transform::from_xyz(next_pos.x, next_pos.y, 15.0),
                                        BloodParticle { lifetime: Timer::from_seconds(0.2, TimerMode::Once), on_ground: false },
                                        Velocity(Vec2::ZERO),
                                    ));
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
                                    },
                                ));
                            }
                        }
                    }

                    commands.entity(zombie_entity).try_despawn();
                    score.kills += 1;
                    wave.zombies_alive = wave.zombies_alive.saturating_sub(1);
                    combo.position += settings.combo_kill_boost;
                    spawn_blood(&mut commands, zombie_pos);

                    // Drop-Chance (~8%)
                    if rand::thread_rng().gen_ratio(1, 12) {
                        spawn_drop(&mut commands, zombie_pos);
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
    mut zombie_query: Query<(Entity, &Transform, &mut Health), With<Zombie>>,
) {
    for (expl_transform, mut explosion) in explosion_query.iter_mut() {
        if explosion.damaged { continue; }
        explosion.damaged = true;
        let expl_pos = expl_transform.translation.truncate();

        for (zombie_entity, zombie_transform, mut health) in zombie_query.iter_mut() {
            let zombie_pos = zombie_transform.translation.truncate();
            let dist = expl_pos.distance(zombie_pos);
            if dist < explosion.radius {
                let falloff = 1.0 - (dist / explosion.radius);
                health.current -= explosion.damage * falloff;
                spawn_blood(&mut commands, zombie_pos);
                if health.current <= 0.0 {
                    commands.entity(zombie_entity).try_despawn();
                    score.kills += 1;
                    wave.zombies_alive = wave.zombies_alive.saturating_sub(1);
                    combo.position += settings.combo_kill_boost;
                    spawn_blood(&mut commands, zombie_pos);
                    if rand::thread_rng().gen_ratio(1, 12) {
                        spawn_drop(&mut commands, zombie_pos);
                    }
                }
            }
        }
    }
}

pub fn zombie_player_collision(
    time: Res<Time>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    settings: Res<GameSettings>,
    mut wave: ResMut<WaveState>,
    mut zombie_query: Query<(&mut Zombie, &Transform)>,
    mut player_query: Query<(Entity, &Player, &mut Health, &mut Transform), Without<Zombie>>,
) {
    use crate::constants::*;
    for (mut zombie, zombie_transform) in zombie_query.iter_mut() {
        let zombie_pos = zombie_transform.translation.truncate();
        zombie.damage_cooldown.tick(time.delta());
        for (entity, player, mut health, mut player_transform) in player_query.iter_mut() {
            let player_pos = player_transform.translation.truncate();
            if aabb_overlap(player_pos, PLAYER_SIZE, zombie_pos, ZOMBIE_SIZE) {
                if zombie.damage_cooldown.finished() {
                    health.current -= settings.zombie_damage;
                    zombie.damage_cooldown.reset();
                    let diff = player_pos - zombie_pos;
                    if diff.length() > 0.1 {
                        let kb = diff.normalize() * 20.0;
                        player_transform.translation.x += kb.x;
                        player_transform.translation.y += kb.y;
                    }
                    if health.current <= 0.0 {
                        wave.dead_players.push(player.id);
                        commands.entity(entity).try_despawn_recursive();
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
