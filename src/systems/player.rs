use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use super::explosion_fx::MuzzleFlashMaterial;

pub fn spawn_players(mut commands: Commands, settings: Res<GameSettings>) {
    do_spawn_players(&mut commands, &settings);
}

pub fn do_spawn_players(commands: &mut Commands, settings: &GameSettings) {
    spawn_one_player(commands, settings, PlayerId::P1, -80.0, PLAYER_COLOR_P1, Vec2::X);
    if settings.player_count >= 2 {
        spawn_one_player(commands, settings, PlayerId::P2, 80.0, PLAYER_COLOR_P2, Vec2::NEG_X);
    }
}

// Spieler-Proportionen (Boxhead-Stil)
const HEAD_SIZE: Vec2 = Vec2::new(18.0, 18.0);
const BODY_SIZE: Vec2 = Vec2::new(14.0, 12.0);
const LEG_SIZE: Vec2 = Vec2::new(5.0, 8.0);
const ARM_SIZE: Vec2 = Vec2::new(5.0, 10.0);
const LEG_SPACING: f32 = 5.0;
const ARM_OFFSET_X: f32 = 9.5;

fn skin_color() -> Color {
    Color::srgb(0.9, 0.75, 0.6)
}

pub fn spawn_one_player_pub(commands: &mut Commands, settings: &GameSettings, id: PlayerId, x: f32, color: Color, facing: Vec2) {
    spawn_one_player(commands, settings, id, x, color, facing);
}

fn darken(c: Color, factor: f32) -> Color {
    let s = c.to_srgba();
    Color::srgb(s.red * factor, s.green * factor, s.blue * factor)
}

fn spawn_one_player(commands: &mut Commands, settings: &GameSettings, id: PlayerId, x: f32, color: Color, facing: Vec2) {
    let ws = settings.weapon(WeaponType::Pistol);
    let weapon = WeaponType::Pistol;
    let weapon_arm_side: f32 = if rand::Rng::random_bool(&mut rand::rng(), 0.5) { 1.0 } else { -1.0 };
    let body_color = darken(color, 0.7);

    // Initialize magazines for all weapons
    let mut magazines = std::collections::HashMap::new();
    for w in WeaponType::all() {
        let wset = settings.weapon(*w);
        let max_mags = if wset.max_magazines > 0 { wset.max_magazines } else { 999 };
        magazines.insert(*w, max_mags);
    }

    commands
        .spawn((
            // Unsichtbarer Root fuer Collision
            Sprite { color: Color::NONE, custom_size: Some(PLAYER_SIZE), ..default() },
            Transform::from_xyz(x, 0.0, 10.0),
            Player {
                id, facing, weapon,
                ammo: ws.magazine,
                shoot_cooldown: Timer::from_seconds(ws.cooldown, TimerMode::Once),
                reload_timer: Timer::from_seconds(ws.reload_time, TimerMode::Once),
                reloading: false, reload_elapsed: 0.0,
                magazines,
            },
            Health { current: settings.player_hp, max: settings.player_hp },
            RegenCooldown { timer: Timer::from_seconds(settings.player_regen_delay.max(0.1), TimerMode::Once) },
        ))
        .with_children(|parent| {
            // Kopf (grosser Block, oben)
            parent.spawn((
                Sprite { color, custom_size: Some(HEAD_SIZE), ..default() },
                Transform::from_xyz(0.0, 8.0, 2.0),
                PlayerHead,
            ));
            // Linkes Auge
            parent.spawn((
                Sprite { color: Color::WHITE, custom_size: Some(Vec2::new(4.0, 4.0)), ..default() },
                Transform::from_xyz(-4.0, 10.0, 3.0),
                PlayerEye { side: -1.0 },
            ));
            // Rechtes Auge
            parent.spawn((
                Sprite { color: Color::WHITE, custom_size: Some(Vec2::new(4.0, 4.0)), ..default() },
                Transform::from_xyz(4.0, 10.0, 3.0),
                PlayerEye { side: 1.0 },
            ));
            // Koerper (dunkler als Kopf)
            parent.spawn((
                Sprite { color: body_color, custom_size: Some(BODY_SIZE), ..default() },
                Transform::from_xyz(0.0, -4.0, 1.0),
                PlayerBody,
            ));
            // Linkes Bein
            parent.spawn((
                Sprite { color: skin_color(), custom_size: Some(LEG_SIZE), ..default() },
                Transform::from_xyz(-LEG_SPACING, -14.0, 0.5),
                PlayerLeg { side: -1.0 },
            ));
            // Rechtes Bein
            parent.spawn((
                Sprite { color: skin_color(), custom_size: Some(LEG_SIZE), ..default() },
                Transform::from_xyz(LEG_SPACING, -14.0, 0.5),
                PlayerLeg { side: 1.0 },
            ));
            // Linker Arm (Waffen-Arm = Hautfarbe/nackt, freier Arm = Aermel)
            parent.spawn((
                Sprite {
                    color: if weapon_arm_side < 0.0 { skin_color() } else { body_color },
                    custom_size: Some(ARM_SIZE),
                    ..default()
                },
                Transform::from_xyz(-ARM_OFFSET_X, -2.0, 0.5),
                PlayerArm { side: -1.0, has_weapon: weapon_arm_side < 0.0 },
            ));
            // Rechter Arm
            parent.spawn((
                Sprite {
                    color: if weapon_arm_side > 0.0 { skin_color() } else { body_color },
                    custom_size: Some(ARM_SIZE),
                    ..default()
                },
                Transform::from_xyz(ARM_OFFSET_X, -2.0, 0.5),
                PlayerArm { side: 1.0, has_weapon: weapon_arm_side > 0.0 },
            ));
            // Waffen-Sprite Container (am Waffen-Arm)
            parent.spawn((
                Sprite { color: Color::NONE, custom_size: Some(weapon.sprite_size()), ..default() },
                Transform::from_xyz(ARM_OFFSET_X * weapon_arm_side + weapon.sprite_size().x / 2.0 * weapon_arm_side, -2.0, 3.0),
                WeaponSprite,
            )).with_children(|wp| {
                spawn_weapon_parts(wp, weapon);
            });
            // HP-Balken
            parent.spawn((
                Sprite { color: Color::srgb(0.3, 0.0, 0.0), custom_size: Some(Vec2::new(HP_BAR_WIDTH, HP_BAR_HEIGHT)), ..default() },
                Transform::from_xyz(0.0, HP_BAR_OFFSET_Y, 4.0),
                PlayerHpBarBg,
            ));
            parent.spawn((
                Sprite { color: Color::srgb(0.0, 0.8, 0.0), custom_size: Some(Vec2::new(HP_BAR_WIDTH, HP_BAR_HEIGHT)), ..default() },
                Transform::from_xyz(0.0, HP_BAR_OFFSET_Y, 5.0),
                PlayerHpBar,
            ));
        });
}

/// Baut Composite-Waffen-Sprites aus mehreren Teilen
/// Koordinaten relativ zum WeaponSprite-Container, X = vorwaerts (Lauf), Y = seitlich
fn spawn_weapon_parts(parent: &mut bevy::prelude::ChildSpawnerCommands, weapon: WeaponType) {
    let metal = Color::srgb(0.55, 0.55, 0.58);
    let dark_metal = Color::srgb(0.35, 0.35, 0.38);
    let wood = Color::srgb(0.45, 0.3, 0.15);
    let dark_wood = Color::srgb(0.35, 0.2, 0.1);

    match weapon {
        WeaponType::Pistol => {
            // Lauf
            parent.spawn((Sprite { color: metal, custom_size: Some(Vec2::new(8.0, 3.0)), ..default() },
                Transform::from_xyz(3.0, 0.0, 0.0), WeaponPart));
            // Griff
            parent.spawn((Sprite { color: dark_metal, custom_size: Some(Vec2::new(3.0, 5.0)), ..default() },
                Transform::from_xyz(-3.0, -1.5, -0.1), WeaponPart));
        }
        WeaponType::Uzi => {
            // Lauf
            parent.spawn((Sprite { color: metal, custom_size: Some(Vec2::new(10.0, 3.0)), ..default() },
                Transform::from_xyz(4.0, 0.0, 0.0), WeaponPart));
            // Body
            parent.spawn((Sprite { color: dark_metal, custom_size: Some(Vec2::new(6.0, 5.0)), ..default() },
                Transform::from_xyz(-2.0, 0.0, -0.1), WeaponPart));
            // Magazin
            parent.spawn((Sprite { color: Color::srgb(0.3, 0.3, 0.3), custom_size: Some(Vec2::new(2.0, 5.0)), ..default() },
                Transform::from_xyz(-1.0, -3.0, -0.2), WeaponPart));
        }
        WeaponType::Shotgun => {
            // Langer Lauf
            parent.spawn((Sprite { color: metal, custom_size: Some(Vec2::new(14.0, 3.0)), ..default() },
                Transform::from_xyz(2.0, 0.0, 0.0), WeaponPart));
            // Zweiter Lauf (Doppellauf-Look)
            parent.spawn((Sprite { color: dark_metal, custom_size: Some(Vec2::new(12.0, 2.0)), ..default() },
                Transform::from_xyz(3.0, 2.0, -0.1), WeaponPart));
            // Holzschaft
            parent.spawn((Sprite { color: wood, custom_size: Some(Vec2::new(6.0, 4.0)), ..default() },
                Transform::from_xyz(-7.0, 0.5, -0.1), WeaponPart));
        }
        WeaponType::Flamethrower => {
            // Tank (hinten)
            parent.spawn((Sprite { color: Color::srgb(0.6, 0.25, 0.1), custom_size: Some(Vec2::new(6.0, 6.0)), ..default() },
                Transform::from_xyz(-5.0, 0.0, -0.1), WeaponPart));
            // Rohr
            parent.spawn((Sprite { color: metal, custom_size: Some(Vec2::new(10.0, 3.0)), ..default() },
                Transform::from_xyz(3.0, 0.0, 0.0), WeaponPart));
            // Duese (vorne, breiter)
            parent.spawn((Sprite { color: Color::srgb(0.7, 0.3, 0.0), custom_size: Some(Vec2::new(3.0, 5.0)), ..default() },
                Transform::from_xyz(9.0, 0.0, 0.1), WeaponPart));
        }
        WeaponType::Grenade => {
            // Koerper (rund-eckig)
            parent.spawn((Sprite { color: Color::srgb(0.3, 0.4, 0.2), custom_size: Some(Vec2::new(6.0, 6.0)), ..default() },
                Transform::from_xyz(0.0, 0.0, 0.0), WeaponPart));
            // Zuender oben
            parent.spawn((Sprite { color: metal, custom_size: Some(Vec2::new(2.0, 3.0)), ..default() },
                Transform::from_xyz(0.0, 4.0, 0.1), WeaponPart));
        }
        WeaponType::Railgun => {
            // Langer Lauf
            parent.spawn((Sprite { color: Color::srgb(0.2, 0.5, 0.6), custom_size: Some(Vec2::new(18.0, 2.0)), ..default() },
                Transform::from_xyz(4.0, 0.0, 0.0), WeaponPart));
            // Energiekern (leuchtend)
            parent.spawn((Sprite { color: Color::srgb(0.3, 0.8, 1.0), custom_size: Some(Vec2::new(4.0, 4.0)), ..default() },
                Transform::from_xyz(-2.0, 0.0, 0.1), WeaponPart));
            // Griff
            parent.spawn((Sprite { color: dark_metal, custom_size: Some(Vec2::new(3.0, 4.0)), ..default() },
                Transform::from_xyz(-6.0, -1.0, -0.1), WeaponPart));
        }
        WeaponType::FreezeGun => {
            // Lauf
            parent.spawn((Sprite { color: Color::srgb(0.5, 0.7, 0.8), custom_size: Some(Vec2::new(10.0, 3.0)), ..default() },
                Transform::from_xyz(4.0, 0.0, 0.0), WeaponPart));
            // Eiskristall-Muendung
            parent.spawn((Sprite { color: Color::srgb(0.6, 0.9, 1.0), custom_size: Some(Vec2::new(3.0, 5.0)), ..default() },
                Transform::from_xyz(10.0, 0.0, 0.1), WeaponPart));
            // Body
            parent.spawn((Sprite { color: Color::srgb(0.2, 0.4, 0.5), custom_size: Some(Vec2::new(5.0, 5.0)), ..default() },
                Transform::from_xyz(-2.0, 0.0, -0.1), WeaponPart));
        }
        WeaponType::Tesla => {
            // Spule (hinten)
            parent.spawn((Sprite { color: Color::srgb(0.4, 0.4, 0.6), custom_size: Some(Vec2::new(5.0, 6.0)), ..default() },
                Transform::from_xyz(-3.0, 0.0, -0.1), WeaponPart));
            // Rohr
            parent.spawn((Sprite { color: metal, custom_size: Some(Vec2::new(8.0, 3.0)), ..default() },
                Transform::from_xyz(3.0, 0.0, 0.0), WeaponPart));
            // Blitz-Spitze
            parent.spawn((Sprite { color: Color::srgb(0.5, 0.5, 1.0), custom_size: Some(Vec2::new(3.0, 3.0)), ..default() },
                Transform::from_xyz(8.0, 0.0, 0.1), WeaponPart));
        }
        WeaponType::Laser => {
            // Langer duenner Lauf
            parent.spawn((Sprite { color: Color::srgb(0.6, 0.2, 0.2), custom_size: Some(Vec2::new(16.0, 2.0)), ..default() },
                Transform::from_xyz(4.0, 0.0, 0.0), WeaponPart));
            // Linse vorne
            parent.spawn((Sprite { color: Color::srgb(1.0, 0.3, 0.3), custom_size: Some(Vec2::new(2.0, 4.0)), ..default() },
                Transform::from_xyz(13.0, 0.0, 0.1), WeaponPart));
            // Energiezelle
            parent.spawn((Sprite { color: Color::srgb(0.8, 0.1, 0.1), custom_size: Some(Vec2::new(4.0, 4.0)), ..default() },
                Transform::from_xyz(-4.0, 0.0, -0.1), WeaponPart));
        }
        WeaponType::Rocket => {
            // Rohr
            parent.spawn((Sprite { color: Color::srgb(0.4, 0.3, 0.2), custom_size: Some(Vec2::new(14.0, 4.0)), ..default() },
                Transform::from_xyz(2.0, 0.0, 0.0), WeaponPart));
            // Muendung (breiter)
            parent.spawn((Sprite { color: dark_metal, custom_size: Some(Vec2::new(3.0, 6.0)), ..default() },
                Transform::from_xyz(10.0, 0.0, 0.1), WeaponPart));
            // Griff/Schulterstuetze
            parent.spawn((Sprite { color: dark_wood, custom_size: Some(Vec2::new(5.0, 3.0)), ..default() },
                Transform::from_xyz(-6.0, -2.0, -0.1), WeaponPart));
        }
        WeaponType::Mine => {
            // Koerper (flach)
            parent.spawn((Sprite { color: Color::srgb(0.4, 0.4, 0.15), custom_size: Some(Vec2::new(7.0, 7.0)), ..default() },
                Transform::from_xyz(0.0, 0.0, 0.0), WeaponPart));
            // Druckplatte oben
            parent.spawn((Sprite { color: Color::srgb(0.6, 0.15, 0.1), custom_size: Some(Vec2::new(4.0, 4.0)), ..default() },
                Transform::from_xyz(0.0, 0.0, 0.1), WeaponPart));
        }
        WeaponType::Boomerang => {
            // Fluegel links
            parent.spawn((Sprite { color: wood, custom_size: Some(Vec2::new(5.0, 3.0)), ..default() },
                Transform::from_xyz(-2.0, 1.5, 0.0), WeaponPart));
            // Fluegel rechts
            parent.spawn((Sprite { color: wood, custom_size: Some(Vec2::new(5.0, 3.0)), ..default() },
                Transform::from_xyz(2.0, -1.5, 0.0), WeaponPart));
            // Mitte
            parent.spawn((Sprite { color: dark_wood, custom_size: Some(Vec2::new(3.0, 3.0)), ..default() },
                Transform::from_xyz(0.0, 0.0, 0.1), WeaponPart));
        }
        WeaponType::Buzzsaw => {
            // Scheibe
            parent.spawn((Sprite { color: metal, custom_size: Some(Vec2::new(9.0, 9.0)), ..default() },
                Transform::from_xyz(0.0, 0.0, 0.0), WeaponPart));
            // Inneres (dunkler)
            parent.spawn((Sprite { color: dark_metal, custom_size: Some(Vec2::new(4.0, 4.0)), ..default() },
                Transform::from_xyz(0.0, 0.0, 0.1), WeaponPart));
            // Zaehne (markiert durch helle Punkte an den Seiten)
            for i in 0..4 {
                let a = i as f32 * std::f32::consts::FRAC_PI_2;
                parent.spawn((Sprite { color: Color::srgb(0.8, 0.8, 0.8), custom_size: Some(Vec2::new(2.0, 2.0)), ..default() },
                    Transform::from_xyz(a.cos() * 4.5, a.sin() * 4.5, 0.2), WeaponPart));
            }
        }
    }
}

/// Auto-join P2 when arrow keys are pressed and P2 doesn't exist yet
pub fn player2_join(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut settings: ResMut<GameSettings>,
    player_query: Query<&Player>,
) {
    // Check if P2 already exists
    if player_query.iter().any(|p| p.id == PlayerId::P2) {
        return;
    }

    // Check if any P2 key is pressed
    if keyboard.any_just_pressed([KeyCode::ArrowUp, KeyCode::ArrowDown, KeyCode::ArrowLeft, KeyCode::ArrowRight, KeyCode::Enter]) {
        settings.player_count = 2;
        spawn_one_player(&mut commands, &settings, PlayerId::P2, 80.0, PLAYER_COLOR_P2, Vec2::NEG_X);
    }
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
        }
        transform.translation.x += direction.x * settings.player_speed * time.delta_secs();
        transform.translation.y += direction.y * settings.player_speed * time.delta_secs();
        transform.translation.x = transform.translation.x.clamp(-half_w, half_w);
        transform.translation.y = transform.translation.y.clamp(-half_h, half_h);
    }
}

pub fn player_walk_animation(
    time: Res<Time>,
    player_query: Query<(&Player, &Children)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut leg_query: Query<(&PlayerLeg, &mut Transform), (Without<Player>, Without<PlayerArm>, Without<PlayerEye>, Without<WeaponSprite>, Without<PlayerHead>, Without<PlayerBody>)>,
    mut eye_query: Query<(&PlayerEye, &mut Transform, &mut Visibility), (Without<Player>, Without<PlayerLeg>, Without<PlayerArm>, Without<WeaponSprite>, Without<PlayerHead>, Without<PlayerBody>)>,
    mut arm_query: Query<(&PlayerArm, &mut Transform), (Without<Player>, Without<PlayerLeg>, Without<PlayerEye>, Without<WeaponSprite>, Without<PlayerHead>, Without<PlayerBody>)>,
    mut weapon_query: Query<(&WeaponSprite, &mut Transform), (Without<Player>, Without<PlayerLeg>, Without<PlayerEye>, Without<PlayerArm>, Without<PlayerHead>, Without<PlayerBody>)>,
    mut head_query: Query<&mut Transform, (With<PlayerHead>, Without<Player>, Without<PlayerLeg>, Without<PlayerArm>, Without<PlayerEye>, Without<WeaponSprite>, Without<PlayerBody>)>,
    mut body_query: Query<&mut Transform, (With<PlayerBody>, Without<Player>, Without<PlayerLeg>, Without<PlayerArm>, Without<PlayerEye>, Without<WeaponSprite>, Without<PlayerHead>)>,
) {
    for (player, children) in player_query.iter() {
        let is_moving = match player.id {
            PlayerId::P1 => keyboard.any_pressed([KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD]),
            PlayerId::P2 => keyboard.any_pressed([KeyCode::ArrowUp, KeyCode::ArrowDown, KeyCode::ArrowLeft, KeyCode::ArrowRight]),
        };

        let facing = player.facing;
        let facing_up = facing.y > 0.3;
        let t = time.elapsed_secs();

        // Erst Waffen-Arm-Position bestimmen
        let mut weapon_arm_pos = Vec2::new(ARM_OFFSET_X, -2.0);
        for child in children.iter() {
            if let Ok((arm, _)) = arm_query.get_mut(child) {
                if arm.has_weapon {
                    let base_x = ARM_OFFSET_X * arm.side;
                    let base_y = -2.0;
                    weapon_arm_pos = Vec2::new(base_x + facing.x * 2.0, base_y + facing.y * 1.5);
                }
            }
        }

        for child in children.iter() {
            // Bein-Animation: Beine bewegen sich in Laufrichtung
            if let Ok((leg, mut transform)) = leg_query.get_mut(child) {
                let base_x = LEG_SPACING * leg.side;
                let base_y = -14.0;

                if is_moving {
                    let phase = leg.side;
                    let swing = (t * 12.0 + phase * std::f32::consts::PI).sin() * 3.0;
                    transform.translation.x = base_x + facing.x * swing;
                    transform.translation.y = base_y + facing.y * swing;
                } else {
                    transform.translation.x = base_x;
                    transform.translation.y = base_y;
                }
            }

            // Augen: sichtbar wenn nach unten/seitlich schauend, versteckt wenn nach oben
            if let Ok((eye, mut transform, mut vis)) = eye_query.get_mut(child) {
                if facing_up {
                    *vis = Visibility::Hidden;
                } else {
                    *vis = Visibility::Visible;
                    let eye_base_x = 4.0 * eye.side;
                    transform.translation.x = eye_base_x + facing.x * 2.0;
                    transform.translation.y = 10.0 + facing.y * 1.0;
                }
            }

            // Arme
            if let Ok((arm, mut transform)) = arm_query.get_mut(child) {
                if arm.has_weapon {
                    // Waffen-Arm: zeigt in Facing-Richtung
                    let base_x = ARM_OFFSET_X * arm.side;
                    let base_y = -2.0;
                    transform.translation.x = base_x + facing.x * 2.0;
                    transform.translation.y = base_y + facing.y * 1.5;
                    let angle = facing.y.atan2(facing.x);
                    transform.rotation = Quat::from_rotation_z(angle - std::f32::consts::FRAC_PI_2);
                } else {
                    // Freier Arm: haengt locker runter, wackelt dynamisch
                    let base_x = ARM_OFFSET_X * arm.side;
                    let base_y = -2.0;
                    if is_moving {
                        let swing = (t * 8.0 + arm.side * std::f32::consts::PI).sin();
                        transform.translation.x = base_x + swing * 1.5;
                        transform.translation.y = base_y - 3.0 + swing.abs() * 1.0;
                        transform.rotation = Quat::from_rotation_z(swing * 0.15);
                    } else {
                        // Idle: leichtes Pendeln
                        let sway = (t * 1.2 + arm.side).sin();
                        transform.translation.x = base_x + sway * 0.5;
                        transform.translation.y = base_y - 3.0;
                        transform.rotation = Quat::from_rotation_z(sway * 0.05);
                    }
                }
            }

            // Waffe: auf den Waffen-Arm platzieren
            if let Ok((_ws, mut transform)) = weapon_query.get_mut(child) {
                let weapon = player.weapon;
                let ws_size = weapon.sprite_size();
                let angle = facing.y.atan2(facing.x);
                // Waffe am Arm-Ende in Facing-Richtung
                transform.translation.x = weapon_arm_pos.x + facing.x * (ws_size.x / 2.0 + 2.0);
                transform.translation.y = weapon_arm_pos.y + facing.y * (ws_size.y / 2.0 + 2.0);
                transform.rotation = Quat::from_rotation_z(angle);
            }

            // Kopf: leichtes Wippen beim Laufen
            if let Ok(mut transform) = head_query.get_mut(child) {
                if is_moving {
                    let bob = (t * 12.0).sin() * 0.6;
                    let sway = (t * 6.0).sin() * 0.4;
                    transform.translation.x = sway;
                    transform.translation.y = 8.0 + bob;
                    transform.rotation = Quat::from_rotation_z((t * 6.0).sin() * 0.03);
                } else {
                    // Idle: subtiles Atmen
                    let breathe = (t * 1.5).sin() * 0.3;
                    transform.translation.x = 0.0;
                    transform.translation.y = 8.0 + breathe;
                    transform.rotation = Quat::IDENTITY;
                }
            }

            // Koerper: leichtes Schwanken beim Laufen
            if let Ok(mut transform) = body_query.get_mut(child) {
                if is_moving {
                    let bob = (t * 12.0 + 1.0).sin() * 0.4;
                    let sway = (t * 6.0 + 0.5).sin() * 0.3;
                    transform.translation.x = sway;
                    transform.translation.y = -4.0 + bob;
                    transform.rotation = Quat::from_rotation_z((t * 6.0 + 0.5).sin() * 0.02);
                } else {
                    let breathe = (t * 1.5 + 0.5).sin() * 0.2;
                    transform.translation.x = 0.0;
                    transform.translation.y = -4.0 + breathe;
                    transform.rotation = Quat::IDENTITY;
                }
            }
        }
    }
}

pub fn player_weapon_switch(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    score: Res<Score>,
    settings: Res<GameSettings>,
    mut query: Query<(&mut Player, &Children)>,
    weapon_sprite_query: Query<(Entity, Option<&Children>), With<WeaponSprite>>,
    part_query: Query<Entity, With<WeaponPart>>,
    mut sound_events: ResMut<super::audio::SoundQueue>,
) {
    for (mut player, player_children) in query.iter_mut() {
        let switch = match player.id {
            PlayerId::P1 => keyboard.just_pressed(KeyCode::KeyQ),
            PlayerId::P2 => keyboard.just_pressed(KeyCode::ShiftRight),
        };
        if switch {
            let available: Vec<WeaponType> = if settings.gamemaster_level > 0 {
                WeaponType::all().to_vec()
            } else {
                WeaponType::all().iter().copied()
                    .filter(|w| settings.weapon(*w).score_required <= score.points)
                    .collect()
            };
            if available.len() <= 1 { continue; }
            let idx = available.iter().position(|w| *w == player.weapon).unwrap_or(0);
            let new_weapon = available[(idx + 1) % available.len()];
            let lvl = settings.weapon_level(new_weapon, score.points);
            let ws = settings.weapon_at_level(new_weapon, lvl);
            let ws = &ws;
            player.weapon = new_weapon;
            player.ammo = ws.magazine;
            player.reloading = false;
            player.reload_elapsed = 0.0;
            player.shoot_cooldown = Timer::from_seconds(ws.cooldown, TimerMode::Once);
            player.shoot_cooldown.tick(std::time::Duration::from_secs(10));
            player.reload_timer = Timer::from_seconds(ws.reload_time, TimerMode::Once);
            sound_events.0.push(super::audio::SoundEvent::WeaponSwitch);

            // Waffen-Parts neu aufbauen
            for pc in player_children.iter() {
                if let Ok((ws_entity, ws_children)) = weapon_sprite_query.get(pc) {
                    // Alte Parts despawnen
                    if let Some(children) = ws_children {
                        for wc in children.iter() {
                            if part_query.get(wc).is_ok() {
                                commands.entity(wc).despawn();
                            }
                        }
                    }
                    // Neue Parts spawnen
                    commands.entity(ws_entity).with_children(|wp| {
                        spawn_weapon_parts(wp, new_weapon);
                    });
                }
            }
        }
    }
}

/// Berechnet die Waffenspitze (Muendung) in Weltkoordinaten
pub fn weapon_tip(player: &Player, player_pos: Vec2, weapon_arm_pos: Vec2) -> Vec2 {
    let facing = player.facing;
    let ws_size = player.weapon.sprite_size();
    // Waffe sitzt am Arm-Ende + halbe Waffenlaenge + offset in Facing-Richtung
    let tip_x = weapon_arm_pos.x + facing.x * (ws_size.x / 2.0 + 2.0 + ws_size.x / 2.0);
    let tip_y = weapon_arm_pos.y + facing.y * (ws_size.y / 2.0 + 2.0 + ws_size.y / 2.0);
    player_pos + Vec2::new(tip_x, tip_y)
}

pub fn player_shoot(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    settings: Res<GameSettings>,
    score: Res<Score>,
    mut query: Query<(&mut Player, &Transform, &Children)>,
    arm_query: Query<(&PlayerArm, &Transform), Without<Player>>,
    mut sound_events: ResMut<super::audio::SoundQueue>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut flash_materials: ResMut<Assets<MuzzleFlashMaterial>>,
) {
    for (mut player, transform, children) in query.iter_mut() {
        let lvl = settings.weapon_level(player.weapon, score.points);
        let ws = settings.weapon_at_level(player.weapon, lvl);
        let ws = &ws;

        if player.reloading {
            player.reload_timer.tick(time.delta());
            player.reload_elapsed += time.delta_secs();
            if player.reload_timer.is_finished() {
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

        if wants_shoot && player.shoot_cooldown.is_finished() && player.ammo > 0 {
            // Cooldown dynamisch aus Settings (fuer Tweaking)
            player.shoot_cooldown = Timer::from_seconds(ws.cooldown, TimerMode::Once);
            player.shoot_cooldown.tick(std::time::Duration::from_secs(0));
            player.ammo -= 1;
            sound_events.0.push(super::audio::SoundEvent::Shoot(player.weapon));

            let weapon = player.weapon;
            let dir = player.facing;
            let player_pos = transform.translation.truncate();
            let mut rng = rand::rng();

            // Waffen-Arm-Position finden
            let mut weapon_arm_pos = Vec2::new(ARM_OFFSET_X, -2.0);
            for child in children.iter() {
                if let Ok((arm, arm_t)) = arm_query.get(child) {
                    if arm.has_weapon {
                        weapon_arm_pos = Vec2::new(arm_t.translation.x, arm_t.translation.y);
                    }
                }
            }

            let tip = weapon_tip(&player, player_pos, weapon_arm_pos);
            let pos = tip.extend(transform.translation.z);
            let angle = dir.y.atan2(dir.x);

            // Muzzle Flash spawnen (nur fuer Waffen die einen haben)
            if let Some((color_inner, color_outer)) = weapon.muzzle_flash_colors() {
                let flash_size = weapon.muzzle_flash_size();
                super::explosion_fx::spawn_muzzle_flash_at(
                    &mut commands,
                    &mut meshes,
                    &mut flash_materials,
                    &player,
                    tip,
                    color_inner,
                    color_outer,
                    flash_size,
                );
            }

            match weapon {
                WeaponType::Grenade => {
                    commands.spawn((
                        Sprite { color: weapon.bullet_color(), custom_size: Some(weapon.bullet_size()), ..default() },
                        Transform::from_translation(pos).with_rotation(Quat::from_rotation_z(angle)),
                        GrenadeProjectile {
                            damage: ws.damage,
                            fuse: Timer::from_seconds(ws.range / ws.bullet_speed, TimerMode::Once),
                            explosion_radius: settings.explosion_radius,
                            level: lvl,
                        },
                        Velocity(dir * ws.bullet_speed),
                        BulletOwner(player.id),
                    ));
                }
                WeaponType::Rocket => {
                    let expl_r = if ws.explosion_radius_override > 0.0 { ws.explosion_radius_override } else { settings.explosion_radius };
                    commands.spawn((
                        Sprite { color: weapon.bullet_color(), custom_size: Some(weapon.bullet_size()), ..default() },
                        Transform::from_translation(pos).with_rotation(Quat::from_rotation_z(angle)),
                        RocketProjectile {
                            damage: ws.damage,
                            explosion_radius: expl_r,
                            range_remaining: ws.range,
                            level: lvl,
                        },
                        Velocity(dir * ws.bullet_speed),
                        BulletOwner(player.id),
                    ));
                }
                WeaponType::Flamethrower => {
                    let sa = ws.spread_angle.max(0.01);
                    let spread = rng.random_range(-sa..sa);
                    let fa = angle + spread;
                    let fd = Vec2::new(fa.cos(), fa.sin());
                    commands.spawn((
                        Sprite {
                            color: Color::srgb(1.0, rng.random_range(0.2..0.6), 0.0),
                            custom_size: Some(Vec2::splat(rng.random_range(4.0..8.0))),
                            ..default()
                        },
                        Transform::from_translation(pos).with_rotation(Quat::from_rotation_z(fa)),
                        Bullet { damage: ws.damage, range_remaining: ws.range * rng.random_range(0.6..1.0), pierce_remaining: 1 },
                        FlameBullet,
                        Velocity(fd * ws.bullet_speed * rng.random_range(0.7..1.3)),
                        BulletOwner(player.id),
                    ));
                }
                WeaponType::Shotgun => {
                    let count = ws.pellet_count.max(1);
                    let spread = ws.spread_angle.max(0.01);
                    for _ in 0..count {
                        let offset = rng.random_range(-spread..spread);
                        let pa = angle + offset;
                        let pd = Vec2::new(pa.cos(), pa.sin());
                        commands.spawn((
                            Sprite { color: weapon.bullet_color(), custom_size: Some(weapon.bullet_size()), ..default() },
                            Transform::from_translation(pos).with_rotation(Quat::from_rotation_z(pa)),
                            Bullet { damage: ws.damage, range_remaining: ws.range * rng.random_range(0.7..1.0), pierce_remaining: 1 },
                            Velocity(pd * ws.bullet_speed * rng.random_range(0.85..1.15)),
                            BulletOwner(player.id),
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
                        BulletOwner(player.id),
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
                        BulletOwner(player.id),
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
                        BulletOwner(player.id),
                    ));
                }
                WeaponType::Buzzsaw => {
                    let pierce = if ws.pierce_count > 0 { ws.pierce_count } else { 999 };
                    commands.spawn((
                        Sprite { color: weapon.bullet_color(), custom_size: Some(weapon.bullet_size()), ..default() },
                        Transform::from_translation(pos),
                        Bullet { damage: ws.damage, range_remaining: ws.range, pierce_remaining: pierce },
                        Velocity(dir * ws.bullet_speed),
                        Spinning { speed: 12.0 },
                        BulletOwner(player.id),
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
                        BulletOwner(player.id),
                    ));
                }
                // Laser, Railgun, Pistol, Uzi - standard bullets
                _ => {
                    let final_angle = if ws.spread_angle > 0.0 {
                        angle + rng.random_range(-ws.spread_angle / 2.0..ws.spread_angle / 2.0)
                    } else {
                        angle
                    };
                    let final_dir = Vec2::new(final_angle.cos(), final_angle.sin());
                    let pierce = if ws.pierce_count > 0 { ws.pierce_count } else { 1 };
                    commands.spawn((
                        Sprite { color: weapon.bullet_color(), custom_size: Some(weapon.bullet_size()), ..default() },
                        Transform::from_translation(pos).with_rotation(Quat::from_rotation_z(final_angle)),
                        Bullet { damage: ws.damage, range_remaining: ws.range, pierce_remaining: pierce },
                        Velocity(final_dir * ws.bullet_speed),
                        BulletOwner(player.id),
                    ));
                }
            }

            if player.ammo == 0 {
                let current_weapon = player.weapon;
                let mags = player.magazines.entry(current_weapon).or_insert(0);
                if *mags > 0 {
                    *mags -= 1;
                    player.reloading = true;
                    player.reload_elapsed = 0.0;
                    player.reload_timer = Timer::from_seconds(ws.reload_time, TimerMode::Once);
                    sound_events.0.push(super::audio::SoundEvent::Reload);
                }
                // If no magazines left, player can't reload (must pick up ammo)
            }
        }
    }
}

pub fn player_regeneration(
    time: Res<Time>,
    settings: Res<GameSettings>,
    mut query: Query<(&mut Health, &mut RegenCooldown), With<Player>>,
) {
    if settings.player_regen_rate <= 0.0 { return; }
    for (mut health, mut regen) in query.iter_mut() {
        regen.timer.tick(time.delta());
        if regen.timer.is_finished() && health.current < health.max {
            health.current = (health.current + settings.player_regen_rate * time.delta_secs()).min(health.max);
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
            if let Ok((hp_bar, hp_bg, mut sprite, mut transform)) = hp_children.get_mut(child) {
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
    arm_query: Query<(&PlayerArm, &Transform), (Without<WeaponSprite>, Without<Player>)>,
    mut weapon_query: Query<&mut Transform, (With<WeaponSprite>, Without<Player>, Without<PlayerArm>)>,
) {
    for (player, children) in player_query.iter() {
        let facing = player.facing;

        // Waffen-Arm-Position finden
        let mut weapon_arm_pos = Vec2::new(ARM_OFFSET_X, -2.0);
        for child in children.iter() {
            if let Ok((arm, arm_transform)) = arm_query.get(child) {
                if arm.has_weapon {
                    weapon_arm_pos = Vec2::new(arm_transform.translation.x, arm_transform.translation.y);
                }
            }
        }

        for child in children.iter() {
            if let Ok(mut transform) = weapon_query.get_mut(child) {
                let size = player.weapon.sprite_size();

                // Waffe am Waffen-Arm platzieren
                let angle = facing.y.atan2(facing.x);
                transform.translation.x = weapon_arm_pos.x + facing.x * (size.x / 2.0 + 2.0);
                transform.translation.y = weapon_arm_pos.y + facing.y * (size.y / 2.0 + 2.0);

                if player.reloading {
                    let wobble = (player.reload_elapsed * 20.0).sin() * 0.3;
                    transform.rotation = Quat::from_rotation_z(angle + wobble);
                } else {
                    transform.rotation = Quat::from_rotation_z(angle);
                }
            }
        }
    }
}
