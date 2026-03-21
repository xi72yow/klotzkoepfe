use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;

pub fn spawn_players(mut commands: Commands, settings: Res<GameSettings>) {
    spawn_one_player(&mut commands, &settings, PlayerId::P1, -80.0, PLAYER_COLOR_P1, Vec2::X);
    spawn_one_player(&mut commands, &settings, PlayerId::P2, 80.0, PLAYER_COLOR_P2, Vec2::NEG_X);
}

fn spawn_one_player(commands: &mut Commands, settings: &GameSettings, id: PlayerId, x: f32, color: Color, facing: Vec2) {
    let ws = settings.weapon(WeaponType::Pistol);
    let weapon = WeaponType::Pistol;
    commands
        .spawn((
            Sprite { color, custom_size: Some(PLAYER_SIZE), ..default() },
            Transform::from_xyz(x, 0.0, 10.0),
            Player {
                id, facing, weapon,
                ammo: ws.magazine,
                shoot_cooldown: Timer::from_seconds(ws.cooldown, TimerMode::Once),
                reload_timer: Timer::from_seconds(ws.reload_time, TimerMode::Once),
                reloading: false, reload_elapsed: 0.0,
            },
            Health { current: settings.player_hp, max: settings.player_hp },
        ))
        .with_children(|parent| {
            parent.spawn((
                Sprite { color: Color::srgb(0.3, 0.0, 0.0), custom_size: Some(Vec2::new(HP_BAR_WIDTH, HP_BAR_HEIGHT)), ..default() },
                Transform::from_xyz(0.0, HP_BAR_OFFSET_Y, 1.0),
                PlayerHpBarBg,
            ));
            parent.spawn((
                Sprite { color: Color::srgb(0.0, 0.8, 0.0), custom_size: Some(Vec2::new(HP_BAR_WIDTH, HP_BAR_HEIGHT)), ..default() },
                Transform::from_xyz(0.0, HP_BAR_OFFSET_Y, 2.0),
                PlayerHpBar,
            ));
            parent.spawn((
                Sprite { color: weapon.sprite_color(), custom_size: Some(weapon.sprite_size()), ..default() },
                Transform::from_xyz(PLAYER_SIZE.x / 2.0 + weapon.sprite_size().x / 2.0, 0.0, 0.5),
                WeaponSprite,
            ));
        });
}

pub fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    settings: Res<GameSettings>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    let half_w = WINDOW_WIDTH / 2.0 - WALL_THICKNESS - PLAYER_SIZE.x / 2.0;
    let half_h = WINDOW_HEIGHT / 2.0 - WALL_THICKNESS - PLAYER_SIZE.y / 2.0;

    for (mut player, mut transform) in query.iter_mut() {
        let mut direction = Vec2::ZERO;
        match player.id {
            PlayerId::P1 => {
                if keyboard.pressed(KeyCode::KeyW) { direction.y += 1.0; }
                if keyboard.pressed(KeyCode::KeyS) { direction.y -= 1.0; }
                if keyboard.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
                if keyboard.pressed(KeyCode::KeyD) { direction.x += 1.0; }
            }
            PlayerId::P2 => {
                if keyboard.pressed(KeyCode::ArrowUp) { direction.y += 1.0; }
                if keyboard.pressed(KeyCode::ArrowDown) { direction.y -= 1.0; }
                if keyboard.pressed(KeyCode::ArrowLeft) { direction.x -= 1.0; }
                if keyboard.pressed(KeyCode::ArrowRight) { direction.x += 1.0; }
            }
        }
        if direction != Vec2::ZERO {
            direction = direction.normalize();
            player.facing = direction;
            transform.rotation = Quat::from_rotation_z(direction.y.atan2(direction.x));
        }
        transform.translation.x += direction.x * settings.player_speed * time.delta_secs();
        transform.translation.y += direction.y * settings.player_speed * time.delta_secs();
        transform.translation.x = transform.translation.x.clamp(-half_w, half_w);
        transform.translation.y = transform.translation.y.clamp(-half_h, half_h);
    }
}

pub fn player_weapon_switch(
    keyboard: Res<ButtonInput<KeyCode>>,
    score: Res<Score>,
    settings: Res<GameSettings>,
    mut query: Query<&mut Player>,
) {
    for mut player in query.iter_mut() {
        let switch = match player.id {
            PlayerId::P1 => keyboard.just_pressed(KeyCode::KeyQ),
            PlayerId::P2 => keyboard.just_pressed(KeyCode::ShiftRight),
        };
        if switch {
            let available: Vec<WeaponType> = WeaponType::all().iter().copied()
                .filter(|w| settings.weapon(*w).score_required <= score.points)
                .collect();
            if available.len() <= 1 { continue; }
            let idx = available.iter().position(|w| *w == player.weapon).unwrap_or(0);
            let new_weapon = available[(idx + 1) % available.len()];
            let ws = settings.weapon(new_weapon);
            player.weapon = new_weapon;
            player.ammo = ws.magazine;
            player.reloading = false;
            player.reload_elapsed = 0.0;
            player.shoot_cooldown = Timer::from_seconds(ws.cooldown, TimerMode::Once);
            player.shoot_cooldown.tick(std::time::Duration::from_secs(10));
            player.reload_timer = Timer::from_seconds(ws.reload_time, TimerMode::Once);
        }
    }
}

pub fn player_shoot(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    settings: Res<GameSettings>,
    mut query: Query<(&mut Player, &Transform)>,
) {
    for (mut player, transform) in query.iter_mut() {
        let ws = settings.weapon(player.weapon);

        if player.reloading {
            player.reload_timer.tick(time.delta());
            player.reload_elapsed += time.delta_secs();
            if player.reload_timer.finished() {
                player.reloading = false;
                player.reload_elapsed = 0.0;
                player.ammo = ws.magazine;
            }
            continue;
        }

        player.shoot_cooldown.tick(time.delta());

        let wants_shoot = match player.id {
            PlayerId::P1 => keyboard.pressed(KeyCode::Space),
            PlayerId::P2 => keyboard.pressed(KeyCode::Enter),
        };

        if wants_shoot && player.shoot_cooldown.finished() && player.ammo > 0 {
            player.shoot_cooldown.reset();
            player.ammo -= 1;

            let weapon = player.weapon;
            let dir = player.facing;
            let pos = transform.translation;
            let angle = dir.y.atan2(dir.x);
            let mut rng = rand::thread_rng();

            match weapon {
                WeaponType::Grenade => {
                    commands.spawn((
                        Sprite { color: weapon.bullet_color(), custom_size: Some(weapon.bullet_size()), ..default() },
                        Transform::from_translation(pos).with_rotation(Quat::from_rotation_z(angle)),
                        GrenadeProjectile {
                            damage: ws.damage,
                            fuse: Timer::from_seconds(ws.range / ws.bullet_speed, TimerMode::Once),
                            explosion_radius: settings.explosion_radius,
                        },
                        Velocity(dir * ws.bullet_speed),
                    ));
                }
                WeaponType::Rocket => {
                    let expl_r = if ws.explosion_radius_override > 0.0 { ws.explosion_radius_override } else { settings.explosion_radius };
                    commands.spawn((
                        Sprite { color: weapon.bullet_color(), custom_size: Some(weapon.bullet_size()), ..default() },
                        Transform::from_translation(pos).with_rotation(Quat::from_rotation_z(angle)),
                        GrenadeProjectile {
                            damage: ws.damage,
                            fuse: Timer::from_seconds(ws.range / ws.bullet_speed, TimerMode::Once),
                            explosion_radius: expl_r,
                        },
                        Velocity(dir * ws.bullet_speed),
                    ));
                }
                WeaponType::Flamethrower => {
                    let sa = ws.spread_angle.max(0.01);
                    let spread = rng.gen_range(-sa..sa);
                    let fa = angle + spread;
                    let fd = Vec2::new(fa.cos(), fa.sin());
                    commands.spawn((
                        Sprite {
                            color: Color::srgb(1.0, rng.gen_range(0.2..0.6), 0.0),
                            custom_size: Some(Vec2::splat(rng.gen_range(4.0..8.0))),
                            ..default()
                        },
                        Transform::from_translation(pos).with_rotation(Quat::from_rotation_z(fa)),
                        Bullet { damage: ws.damage, range_remaining: ws.range * rng.gen_range(0.6..1.0), pierce_remaining: 1 },
                        Velocity(fd * ws.bullet_speed * rng.gen_range(0.7..1.3)),
                    ));
                }
                WeaponType::Shotgun => {
                    let count = ws.pellet_count.max(1);
                    let spread = ws.spread_angle.max(0.01);
                    for _ in 0..count {
                        let offset = rng.gen_range(-spread..spread);
                        let pa = angle + offset;
                        let pd = Vec2::new(pa.cos(), pa.sin());
                        commands.spawn((
                            Sprite { color: weapon.bullet_color(), custom_size: Some(weapon.bullet_size()), ..default() },
                            Transform::from_translation(pos).with_rotation(Quat::from_rotation_z(pa)),
                            Bullet { damage: ws.damage, range_remaining: ws.range * rng.gen_range(0.7..1.0), pierce_remaining: 1 },
                            Velocity(pd * ws.bullet_speed * rng.gen_range(0.85..1.15)),
                        ));
                    }
                }
                WeaponType::Mine => {
                    let tr = if ws.trigger_radius > 0.0 { ws.trigger_radius } else { 40.0 };
                    let expl_r = if ws.explosion_radius_override > 0.0 { ws.explosion_radius_override } else { settings.explosion_radius };
                    commands.spawn((
                        Sprite { color: weapon.bullet_color(), custom_size: Some(weapon.bullet_size()), ..default() },
                        Transform::from_translation(pos.truncate().extend(5.0)),
                        MineEntity {
                            damage: ws.damage, radius: expl_r, trigger_radius: tr,
                            arm_timer: Timer::from_seconds(0.5, TimerMode::Once),
                        },
                    ));
                }
                WeaponType::Boomerang => {
                    commands.spawn((
                        Sprite { color: weapon.bullet_color(), custom_size: Some(weapon.bullet_size()), ..default() },
                        Transform::from_translation(pos),
                        BoomerangProjectile {
                            damage: ws.damage, owner_id: player.id,
                            returning: false, max_dist: ws.range,
                            traveled: 0.0, direction: dir,
                        },
                        Spinning { speed: 15.0 },
                    ));
                }
                WeaponType::Tesla => {
                    commands.spawn((
                        Sprite { color: weapon.bullet_color(), custom_size: Some(weapon.bullet_size()), ..default() },
                        Transform::from_translation(pos).with_rotation(Quat::from_rotation_z(angle)),
                        Bullet { damage: ws.damage, range_remaining: ws.range, pierce_remaining: 1 },
                        TeslaBullet {
                            chain_count: ws.chain_count.max(1),
                            chain_range: if ws.chain_range > 0.0 { ws.chain_range } else { 80.0 },
                            chain_damage: ws.damage * 0.7,
                        },
                        Velocity(dir * ws.bullet_speed),
                    ));
                }
                WeaponType::Buzzsaw => {
                    commands.spawn((
                        Sprite { color: weapon.bullet_color(), custom_size: Some(weapon.bullet_size()), ..default() },
                        Transform::from_translation(pos),
                        Bullet { damage: ws.damage, range_remaining: ws.range, pierce_remaining: 999 },
                        Velocity(dir * ws.bullet_speed),
                        Spinning { speed: 12.0 },
                    ));
                }
                WeaponType::FreezeGun => {
                    commands.spawn((
                        Sprite { color: weapon.bullet_color(), custom_size: Some(weapon.bullet_size()), ..default() },
                        Transform::from_translation(pos).with_rotation(Quat::from_rotation_z(angle)),
                        Bullet { damage: ws.damage, range_remaining: ws.range, pierce_remaining: 1 },
                        FreezeBullet {
                            slow_factor: if ws.slow_factor > 0.0 { ws.slow_factor } else { 0.25 },
                            slow_duration: if ws.slow_duration > 0.0 { ws.slow_duration } else { 3.0 },
                        },
                        Velocity(dir * ws.bullet_speed),
                    ));
                }
                // Laser, Railgun, Pistol, Uzi - standard bullets
                _ => {
                    commands.spawn((
                        Sprite { color: weapon.bullet_color(), custom_size: Some(weapon.bullet_size()), ..default() },
                        Transform::from_translation(pos).with_rotation(Quat::from_rotation_z(angle)),
                        Bullet { damage: ws.damage, range_remaining: ws.range, pierce_remaining: weapon.piercing() },
                        Velocity(dir * ws.bullet_speed),
                    ));
                }
            }

            if player.ammo == 0 {
                player.reloading = true;
                player.reload_elapsed = 0.0;
                player.reload_timer = Timer::from_seconds(ws.reload_time, TimerMode::Once);
            }
        }
    }
}

pub fn update_player_hp_bars(
    player_query: Query<(&Health, &Transform, &Children), With<Player>>,
    mut hp_children: Query<
        (Option<&PlayerHpBar>, Option<&PlayerHpBarBg>, &mut Sprite, &mut Transform),
        Without<Player>,
    >,
) {
    for (health, player_transform, children) in player_query.iter() {
        let ratio = (health.current / health.max).max(0.0);
        let inv_rot = player_transform.rotation.inverse();
        let offset = inv_rot * Vec3::new(0.0, HP_BAR_OFFSET_Y, 0.0);

        for child in children.iter() {
            if let Ok((hp_bar, hp_bg, mut sprite, mut transform)) = hp_children.get_mut(*child) {
                if hp_bar.is_some() {
                    sprite.custom_size = Some(Vec2::new(HP_BAR_WIDTH * ratio, HP_BAR_HEIGHT));
                    sprite.color = if ratio > 0.5 { Color::srgb(0.0, 0.8, 0.0) }
                        else if ratio > 0.25 { Color::srgb(0.8, 0.8, 0.0) }
                        else { Color::srgb(0.8, 0.0, 0.0) };
                    transform.translation = offset + Vec3::new(0.0, 0.0, 2.0);
                    transform.rotation = inv_rot;
                } else if hp_bg.is_some() {
                    transform.translation = offset + Vec3::new(0.0, 0.0, 1.0);
                    transform.rotation = inv_rot;
                }
            }
        }
    }
}

pub fn update_weapon_sprites(
    player_query: Query<(&Player, &Children)>,
    mut weapon_query: Query<(&mut Sprite, &mut Transform), (With<WeaponSprite>, Without<Player>)>,
) {
    for (player, children) in player_query.iter() {
        for child in children.iter() {
            if let Ok((mut sprite, mut transform)) = weapon_query.get_mut(*child) {
                let size = player.weapon.sprite_size();
                sprite.custom_size = Some(size);
                sprite.color = player.weapon.sprite_color();
                transform.translation.x = PLAYER_SIZE.x / 2.0 + size.x / 2.0;
                transform.translation.y = 0.0;
                if player.reloading {
                    let wobble = (player.reload_elapsed * 20.0).sin() * 0.3;
                    transform.rotation = Quat::from_rotation_z(wobble);
                    transform.translation.x = PLAYER_SIZE.x / 2.0 + size.x / 2.0 - 4.0;
                } else {
                    transform.rotation = Quat::IDENTITY;
                }
            }
        }
    }
}
