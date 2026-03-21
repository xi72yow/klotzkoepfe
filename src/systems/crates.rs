use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;

const CRATE_SIZE: f32 = 20.0;
const LIGHT_SIZE: f32 = 3.0;

/// Spawn base crate spawners at fixed positions (called at startup/restart)
pub fn setup_base_crates(mut commands: Commands, settings: Res<GameSettings>) {
    let positions = [
        Vec2::new(-200.0, 150.0),
        Vec2::new(200.0, 150.0),
        Vec2::new(-200.0, -150.0),
        Vec2::new(200.0, -150.0),
    ];

    for pos in positions {
        spawn_loot_crate(&mut commands, pos, CrateType::Base, settings.crate_despawn_time);
    }
}

/// Spawn a random red crate at a given position (called from collision on kill)
pub fn spawn_random_crate(commands: &mut Commands, pos: Vec2, despawn_time: f32) {
    spawn_loot_crate(commands, pos, CrateType::Random, despawn_time);
}

fn spawn_loot_crate(commands: &mut Commands, pos: Vec2, crate_type: CrateType, despawn_time: f32) {
    let color = match crate_type {
        CrateType::Random => Color::srgb(0.8, 0.15, 0.1),
        CrateType::Base => Color::srgb(0.6, 0.5, 0.2),
    };
    let light_timer_duration = if crate_type == CrateType::Random {
        despawn_time / 6.0
    } else {
        999.0 // Base crates don't despawn
    };

    commands
        .spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::splat(CRATE_SIZE)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 7.0),
            LootCrate {
                crate_type,
                despawn_timer: Timer::from_seconds(despawn_time, TimerMode::Once),
                lights: 5,
                light_timer: Timer::from_seconds(light_timer_duration, TimerMode::Repeating),
            },
        ))
        .with_children(|parent| {
            // 5 lights on top
            let light_colors = [
                Color::srgb(0.0, 1.0, 0.0),
                Color::srgb(1.0, 1.0, 0.0),
                Color::srgb(1.0, 0.5, 0.0),
                Color::srgb(1.0, 0.0, 0.0),
                Color::srgb(0.5, 0.0, 1.0),
            ];
            for i in 0..5u8 {
                let lx = -6.0 + i as f32 * 3.0;
                parent.spawn((
                    Sprite {
                        color: light_colors[i as usize],
                        custom_size: Some(Vec2::splat(LIGHT_SIZE)),
                        ..default()
                    },
                    Transform::from_xyz(lx, CRATE_SIZE / 2.0 + 2.0, 1.0),
                    CrateLight { index: i },
                ));
            }
        });
}

/// Update crates: despawn animation and pickup
pub fn crate_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<GameSettings>,
    score: Res<Score>,
    mut player_query: Query<(&mut Player, &mut Health, &Transform)>,
    mut crate_query: Query<(Entity, &mut LootCrate, &Transform, &Children)>,
    mut light_query: Query<(&CrateLight, &mut Visibility)>,
) {
    for (crate_entity, mut loot_crate, crate_transform, children) in crate_query.iter_mut() {
        let crate_pos = crate_transform.translation.truncate();

        // Pickup check
        let mut picked_up = false;
        for (mut player, mut health, player_transform) in player_query.iter_mut() {
            let player_pos = player_transform.translation.truncate();
            if crate_pos.distance(player_pos) < 25.0 {
                // Give loot: ammo + healing if not full HP
                let lvl = settings.weapon_level(player.weapon, score.points);
                let ws = settings.weapon_at_level(player.weapon, lvl);
                player.ammo = ws.magazine;
                player.reloading = false;
                player.reload_elapsed = 0.0;

                // Healing: 25 HP if not full
                if health.current < health.max {
                    health.current = (health.current + 25.0).min(health.max);
                }

                match loot_crate.crate_type {
                    CrateType::Random => {
                        // Refill all magazines
                        for w in WeaponType::all() {
                            let wset = settings.weapon(*w);
                            let max_mags = if wset.max_magazines > 0 { wset.max_magazines } else { 999 };
                            player.magazines.insert(*w, max_mags);
                        }
                    }
                    CrateType::Base => {
                        // Refill one magazine for current weapon
                        let current_weapon = player.weapon;
                        let max_mags = if ws.max_magazines > 0 { ws.max_magazines } else { 999 };
                        let mags = player.magazines.entry(current_weapon).or_insert(0);
                        *mags = (*mags + 1).min(max_mags);
                    }
                }
                picked_up = true;
                break;
            }
        }

        if picked_up {
            commands.entity(crate_entity).try_despawn();

            // If base crate, spawn respawn timer entity
            if loot_crate.crate_type == CrateType::Base {
                commands.spawn(BaseCrateSpawner {
                    position: crate_pos,
                    respawn_timer: Timer::from_seconds(settings.base_crate_respawn_time, TimerMode::Once),
                    active: true,
                });
            }
            continue;
        }

        // Despawn timer (only for random crates)
        if loot_crate.crate_type == CrateType::Random {
            loot_crate.despawn_timer.tick(time.delta());
            loot_crate.light_timer.tick(time.delta());

            // Turn off lights one by one
            if loot_crate.light_timer.just_finished() && loot_crate.lights > 0 {
                loot_crate.lights -= 1;
                let off_index = loot_crate.lights;
                for child in children.iter() {
                    if let Ok((light, mut vis)) = light_query.get_mut(child) {
                        if light.index == off_index {
                            *vis = Visibility::Hidden;
                        }
                    }
                }
            }

            if loot_crate.despawn_timer.is_finished() {
                commands.entity(crate_entity).try_despawn();
            }
        }
    }
}

/// Respawn base crates after timer
pub fn base_crate_respawn(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<GameSettings>,
    mut spawner_query: Query<(Entity, &mut BaseCrateSpawner)>,
) {
    for (entity, mut spawner) in spawner_query.iter_mut() {
        spawner.respawn_timer.tick(time.delta());
        if spawner.respawn_timer.is_finished() {
            spawn_loot_crate(&mut commands, spawner.position, CrateType::Base, settings.crate_despawn_time);
            commands.entity(entity).despawn();
        }
    }
}
