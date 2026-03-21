use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use crate::systems::blood::spawn_blood;

// --- Mine System ---
pub fn mine_system(
    mut commands: Commands,
    time: Res<Time>,
    mut score: ResMut<Score>,
    mut wave: ResMut<WaveState>,
    mut combo: ResMut<ComboMeter>,
    settings: Res<GameSettings>,
    mut mine_query: Query<(Entity, &Transform, &mut MineEntity)>,
    mut zombie_query: Query<(Entity, &Transform, &mut Health), With<Zombie>>,
) {
    for (mine_entity, mine_transform, mut mine) in mine_query.iter_mut() {
        mine.arm_timer.tick(time.delta());
        if !mine.arm_timer.is_finished() {
            continue;
        }

        let mine_pos = mine_transform.translation.truncate();
        let mut triggered = false;

        for (_, zombie_transform, _) in zombie_query.iter() {
            let dist = mine_pos.distance(zombie_transform.translation.truncate());
            if dist < mine.trigger_radius {
                triggered = true;
                break;
            }
        }

        if triggered {
            // Explosion spawnen
            commands.spawn((
                Sprite {
                    color: EXPLOSION_COLOR,
                    custom_size: Some(Vec2::splat(mine.radius * 2.0)),
                    ..default()
                },
                Transform::from_translation(mine_transform.translation),
                Explosion {
                    lifetime: Timer::from_seconds(EXPLOSION_LIFETIME, TimerMode::Once),
                    damage: mine.damage,
                    radius: mine.radius,
                    damaged: false,
                },
            ));
            commands.entity(mine_entity).despawn();
        }
    }
}

// --- Boomerang System ---
pub fn boomerang_system(
    mut commands: Commands,
    time: Res<Time>,
    mut combo: ResMut<ComboMeter>,
    mut score: ResMut<Score>,
    mut wave: ResMut<WaveState>,
    settings: Res<GameSettings>,
    mut player_query: Query<(&mut Player, &Transform)>,
    mut boom_query: Query<(Entity, &mut Transform, &mut BoomerangProjectile), (Without<Player>, Without<Zombie>)>,
    mut zombie_query: Query<(Entity, &Transform, &mut Health), (With<Zombie>, Without<Player>, Without<BoomerangProjectile>)>,
) {
    let ws = settings.weapon(WeaponType::Boomerang);

    for (entity, mut transform, mut boom) in boom_query.iter_mut() {
        let speed = ws.bullet_speed;
        let dt = time.delta_secs();

        if !boom.returning {
            // Vorwaerts fliegen
            transform.translation.x += boom.direction.x * speed * dt;
            transform.translation.y += boom.direction.y * speed * dt;
            boom.traveled += speed * dt;

            if boom.traveled >= boom.max_dist {
                boom.returning = true;
            }
        } else {
            // Zurueck zum Spieler
            let owner_pos = player_query.iter()
                .find(|(p, _)| p.id == boom.owner_id)
                .map(|(_, t)| t.translation.truncate());


            if let Some(target) = owner_pos {
                let pos = transform.translation.truncate();
                let diff = target - pos;
                let dist = diff.length();

                if dist < 15.0 {
                    // Boomerang gefangen: Ammo zurueckgeben
                    for (mut player, _) in player_query.iter_mut() {
                        if player.id == boom.owner_id && player.weapon == WeaponType::Boomerang {
                            let lvl = settings.weapon_level(WeaponType::Boomerang, score.points);
                            let max_ammo = settings.weapon_at_level(WeaponType::Boomerang, lvl).magazine;
                            if player.ammo < max_ammo {
                                player.ammo += 1;
                            }
                            break;
                        }
                    }
                    commands.entity(entity).despawn();
                    continue;
                }

                let dir = diff.normalize();
                transform.translation.x += dir.x * speed * 1.5 * dt;
                transform.translation.y += dir.y * speed * 1.5 * dt;
            } else {
                commands.entity(entity).despawn();
                continue;
            }
        }

        // Collision mit Zombies
        let boom_pos = transform.translation.truncate();
        for (zombie_entity, zombie_transform, mut health) in zombie_query.iter_mut() {
            let zombie_pos = zombie_transform.translation.truncate();
            if boom_pos.distance(zombie_pos) < 20.0 {
                health.current -= boom.damage;
                spawn_blood(&mut commands, zombie_pos);

                if health.current <= 0.0 {
                    commands.entity(zombie_entity).try_despawn();
                    wave.zombies_alive = wave.zombies_alive.saturating_sub(1);
                    crate::systems::collision::register_kill(&mut score, &mut combo, &settings);
                    spawn_blood(&mut commands, zombie_pos);
                    if rand::rng().random::<f32>() < settings.crate_spawn_chance {
                        crate::systems::crates::spawn_random_crate(&mut commands, zombie_pos, settings.crate_despawn_time);
                    }
                }
            }
        }
    }
}

// --- Spinning Visual ---
pub fn spinning_system(
    time: Res<Time>,
    mut query: Query<(&Spinning, &mut Transform)>,
) {
    for (spinning, mut transform) in query.iter_mut() {
        transform.rotate_z(spinning.speed * time.delta_secs());
    }
}

// --- Zombie Freeze Timer ---
pub fn zombie_freeze_update(
    time: Res<Time>,
    mut query: Query<&mut Zombie>,
) {
    for mut zombie in query.iter_mut() {
        if zombie.speed_modifier < 1.0 {
            zombie.freeze_timer.tick(time.delta());
            if zombie.freeze_timer.is_finished() {
                zombie.speed_modifier = 1.0;
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct UnlockedWeapons {
    pub shown: Vec<WeaponType>,
}

// --- Weapon Unlock ---
pub fn weapon_unlock_check(
    mut commands: Commands,
    score: Res<Score>,
    settings: Res<GameSettings>,
    mut unlocked: ResMut<UnlockedWeapons>,
    player_query: Query<(&Player, &Transform)>,
) {
    for weapon in WeaponType::all() {
        let req = settings.weapon(*weapon).score_required;
        if req > 0 && score.points >= req && !unlocked.shown.contains(weapon) {
            unlocked.shown.push(*weapon);

            // Oben am Bildschirm: Waffenname
            commands.spawn((
                Text2d::new(format!("{} freigeschaltet!", weapon.name())),
                TextFont { font_size: 22.0, ..default() },
                TextColor(Color::srgb(1.0, 1.0, 0.3)),
                Transform::from_xyz(0.0, crate::constants::WINDOW_HEIGHT / 2.0 - 80.0, 40.0),
                WeaponUnlockText {
                    lifetime: Timer::from_seconds(3.0, TimerMode::Once),
                },
            ));

            // Ueber jedem Spieler: Waffen-Sprite (Block)
            for (player, transform) in player_query.iter() {
                commands.spawn((
                    Sprite {
                        color: weapon.sprite_color(),
                        custom_size: Some(weapon.sprite_size() * 1.5),
                        ..default()
                    },
                    Transform::from_xyz(
                        transform.translation.x,
                        transform.translation.y + 45.0,
                        30.0,
                    ),
                    WeaponUnlockIcon {
                        lifetime: Timer::from_seconds(2.5, TimerMode::Once),
                        player_id: player.id,
                    },
                ));
            }
        }
    }
}

pub fn weapon_unlock_fade(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<(&Player, &Transform)>,
    mut text_query: Query<(Entity, &mut WeaponUnlockText, &mut TextColor), Without<Player>>,
    mut icon_query: Query<(Entity, &mut WeaponUnlockIcon, &mut Transform, &mut Sprite), Without<Player>>,
) {
    // Text oben ausfaden
    for (entity, mut unlock, mut color) in text_query.iter_mut() {
        unlock.lifetime.tick(time.delta());
        let alpha = 1.0 - unlock.lifetime.fraction();
        *color = TextColor(Color::srgba(1.0, 1.0, 0.3, alpha));
        if unlock.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }

    // Icons ueber Spielern: folgen + nach oben schweben + ausfaden
    for (entity, mut icon, mut transform, mut sprite) in icon_query.iter_mut() {
        icon.lifetime.tick(time.delta());

        if let Some((_, pt)) = player_query.iter().find(|(p, _)| p.id == icon.player_id) {
            transform.translation.x = pt.translation.x;
        }

        transform.translation.y += 15.0 * time.delta_secs();

        let alpha = 1.0 - icon.lifetime.fraction();
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);

        if icon.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

