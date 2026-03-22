use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;

const AMMO_COLOR_EMPTY: Color = Color::srgb(0.15, 0.15, 0.15);
const AMMO_COLOR_RELOAD: Color = Color::srgb(0.8, 0.3, 0.0);
const AMMO_AREA_WIDTH: f32 = 220.0;

/// Gibt (Breite, Hoehe, Farbe) fuer die Ammo-Anzeige einer Waffe zurueck
fn ammo_style(weapon: WeaponType) -> (f32, f32, Color) {
    match weapon {
        WeaponType::Pistol     => (5.0, 12.0, Color::srgb(1.0, 0.9, 0.2)),
        WeaponType::Uzi        => (3.0, 10.0, Color::srgb(1.0, 0.7, 0.1)),
        WeaponType::Shotgun    => (7.0, 14.0, Color::srgb(0.9, 0.5, 0.2)),
        WeaponType::Grenade    => (8.0, 8.0,  Color::srgb(0.3, 0.6, 0.2)),
        WeaponType::Rocket     => (10.0, 5.0, Color::srgb(0.8, 0.3, 0.1)),
        WeaponType::Railgun    => (12.0, 3.0, Color::srgb(0.3, 0.8, 1.0)),
        WeaponType::Flamethrower => (3.0, 6.0, Color::srgb(1.0, 0.4, 0.0)),
        WeaponType::Laser      => (2.0, 8.0,  Color::srgb(1.0, 0.1, 0.1)),
        WeaponType::Mine       => (8.0, 8.0,  Color::srgb(0.6, 0.6, 0.1)),
        WeaponType::Boomerang  => (8.0, 4.0,  Color::srgb(0.8, 0.4, 0.0)),
        WeaponType::Tesla      => (5.0, 10.0, Color::srgb(0.5, 0.5, 1.0)),
        WeaponType::Buzzsaw    => (8.0, 8.0,  Color::srgb(0.7, 0.7, 0.7)),
        WeaponType::FreezeGun  => (4.0, 10.0, Color::srgb(0.4, 0.9, 1.0)),
    }
}

pub fn setup_hud(mut commands: Commands) {
    let track_y = WINDOW_HEIGHT / 2.0 - 40.0;

    // Combo-Track Hintergrund
    commands.spawn((
        Sprite {
            color: Color::srgb(0.15, 0.15, 0.2),
            custom_size: Some(Vec2::new(COMBO_TRACK_WIDTH, COMBO_TRACK_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, track_y, 20.0),
        ComboTrack,
    ));

    // Combo-Block
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 0.8, 0.0),
            custom_size: Some(Vec2::new(COMBO_BLOCK_SIZE, COMBO_BLOCK_SIZE)),
            ..default()
        },
        Transform::from_xyz(0.0, track_y, 21.0),
        ComboBlock,
    ));

    // Score Text
    commands.spawn((
        Text2d::new("Score: 0"),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, track_y + 22.0, 20.0),
        ScoreText,
    ));

    // Wave Text
    commands.spawn((
        Text2d::new("Wave: 1"),
        TextFont { font_size: 20.0, ..default() },
        TextColor(Color::srgb(0.7, 0.7, 0.7)),
        Transform::from_xyz(0.0, track_y - 22.0, 20.0),
        WaveText,
    ));

    // Waffen-Name P1 (links unten)
    let ammo_y = -WINDOW_HEIGHT / 2.0 + 30.0;

    commands.spawn((
        Text2d::new("Pistole"),
        TextFont { font_size: 16.0, ..default() },
        TextColor(PLAYER_COLOR_P1),
        Transform::from_xyz(-WINDOW_WIDTH / 2.0 + 20.0 + 50.0, ammo_y + 18.0, 20.0),
        WeaponNameText(PlayerId::P1),
    ));

    // Waffen-Name P2 (rechts unten)
    commands.spawn((
        Text2d::new("Pistole"),
        TextFont { font_size: 16.0, ..default() },
        TextColor(PLAYER_COLOR_P2),
        Transform::from_xyz(WINDOW_WIDTH / 2.0 - 20.0 - 50.0, ammo_y + 18.0, 20.0),
        WeaponNameText(PlayerId::P2),
    ));
}

pub fn combo_system(
    time: Res<Time>,
    settings: Res<GameSettings>,
    mut combo: ResMut<ComboMeter>,
    mut score: ResMut<Score>,
) {
    // Combo bar visual (no score changes, multiplier handles scoring now)
    if combo.position >= 1.0 {
        combo.position = 0.5;
    }

    combo.position -= settings.combo_drain_speed * time.delta_secs();

    if combo.position <= 0.0 {
        combo.position = 0.5;
    }

    // Multiplier decay
    combo.streak_timer.tick(time.delta());
    if combo.streak_timer.is_finished() && combo.multiplier_index > 0 {
        // Higher tiers decay faster
        let decay_speed = settings.multiplier_decay_rate * (1.0 + combo.multiplier_index as f32 * 0.3);
        if time.elapsed_secs() % (1.0 / decay_speed.max(0.01)) < time.delta_secs() {
            combo.multiplier_index = combo.multiplier_index.saturating_sub(1);
            combo.kill_streak = 0;
        }
    }
}

pub fn update_hud(
    mut commands: Commands,
    score: Res<Score>,
    wave: Res<WaveState>,
    combo: Res<ComboMeter>,
    settings: Res<GameSettings>,
    player_query: Query<&Player>,
    mut block_query: Query<&mut Transform, With<ComboBlock>>,
    mut score_text: Query<&mut Text2d, (With<ScoreText>, Without<WaveText>, Without<WeaponNameText>)>,
    mut wave_text: Query<&mut Text2d, (With<WaveText>, Without<ScoreText>, Without<WeaponNameText>)>,
    mut weapon_name: Query<(&mut Text2d, &WeaponNameText), (Without<ScoreText>, Without<WaveText>)>,
    ammo_query: Query<Entity, With<AmmoIndicator>>,
) {
    // Combo-Block
    if let Ok(mut transform) = block_query.single_mut() {
        let half_track = COMBO_TRACK_WIDTH / 2.0;
        let x = -half_track + combo.position.clamp(0.0, 1.0) * COMBO_TRACK_WIDTH;
        transform.translation.x = x;
        let bounce = (combo.position * std::f32::consts::PI * 4.0).sin().abs() * 4.0;
        let track_y = WINDOW_HEIGHT / 2.0 - 40.0;
        transform.translation.y = track_y + bounce;
    }

    if let Ok(mut text) = score_text.single_mut() {
        let mult = combo.current_multiplier();
        if mult > 1 {
            **text = format!("Score: {} | x{}", score.points, mult);
        } else {
            **text = format!("Score: {}", score.points);
        }
    }
    if let Ok(mut text) = wave_text.single_mut() {
        **text = format!("Wave: {}", wave.current_wave);
    }

    // Waffen-Name aktualisieren
    for (mut text, wnt) in weapon_name.iter_mut() {
        if let Some(player) = player_query.iter().find(|p| p.id == wnt.0) {
            let reload_str = if player.reloading { " [R]" } else { "" };
            let prefix = match wnt.0 {
                PlayerId::P1 => "P1",
                PlayerId::P2 => "P2",
            };
            let lvl = settings.weapon_level(player.weapon, score.points);
            let ws = settings.weapon_at_level(player.weapon, lvl);
            let max_mags = if ws.max_magazines > 0 && ws.max_magazines < 999 { ws.max_magazines } else { 0 };
            let mag_str = if max_mags > 0 {
                let remaining = player.magazines.get(&player.weapon).copied().unwrap_or(max_mags);
                format!(" {}x", remaining)
            } else {
                String::new()
            };
            **text = format!("{}: {}{}{}", prefix, player.weapon.name_at_level(lvl), mag_str, reload_str);
        } else {
            **text = String::new();
        }
    }

    // Alte Ammo-Indikatoren entfernen
    for entity in ammo_query.iter() {
        commands.entity(entity).despawn();
    }

    // Munitions-Anzeige dynamisch pro Spieler aufbauen
    let ammo_y = -WINDOW_HEIGHT / 2.0 + 30.0;

    for player in player_query.iter() {
        let weapon = player.weapon;
        let lvl = settings.weapon_level(weapon, score.points);
        let magazine = settings.weapon_at_level(weapon, lvl).magazine;
        let (rect_w, rect_h, full_color) = ammo_style(weapon);

        // Berechne Layout: max Spalten pro Reihe passend zur AMMO_AREA_WIDTH
        let spacing_x = rect_w + 2.0;
        let spacing_y = rect_h + 2.0;
        let cols_per_row = ((AMMO_AREA_WIDTH / spacing_x) as u32).max(1);
        let rows = (magazine + cols_per_row - 1) / cols_per_row;

        let base_x = match player.id {
            PlayerId::P1 => -WINDOW_WIDTH / 2.0 + 20.0,
            PlayerId::P2 => WINDOW_WIDTH / 2.0 - 20.0 - (cols_per_row.min(magazine) as f32 - 1.0) * spacing_x,
        };

        for i in 0..magazine {
            let col = i % cols_per_row;
            let row = i / cols_per_row;
            let x = base_x + col as f32 * spacing_x;
            let y = ammo_y - row as f32 * spacing_y;

            let color = if player.reloading {
                AMMO_COLOR_RELOAD
            } else if i < player.ammo {
                full_color
            } else {
                AMMO_COLOR_EMPTY
            };

            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(rect_w, rect_h)),
                    ..default()
                },
                Transform::from_xyz(x, y, 20.0),
                AmmoIndicator { player_id: player.id, index: i },
            ));
        }
    }
}

pub fn setup_game_over(mut commands: Commands, score: Res<Score>) {
    commands.spawn((
        Text2d::new("GAME OVER"),
        TextFont { font_size: 64.0, ..default() },
        TextColor(Color::srgb(0.8, 0.0, 0.0)),
        Transform::from_xyz(0.0, 50.0, 30.0),
        GameOverUi,
    ));
    commands.spawn((
        Text2d::new(format!("Score: {} | Kills: {}", score.points, score.kills)),
        TextFont { font_size: 32.0, ..default() },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, -20.0, 30.0),
        GameOverUi,
    ));
    commands.spawn((
        Text2d::new("Press R to restart"),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::srgb(0.7, 0.7, 0.7)),
        Transform::from_xyz(0.0, -60.0, 30.0),
        GameOverUi,
    ));
}

pub fn game_over_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        next_state.set(GameState::Restarting);
    }
}

pub fn restart_game(world: &mut World) {
    // Nur unsere Game-Entities despawnen (Sprites und Text), nicht Bevy-interne Rendering-Entities!
    // GroundDecalLayer bleibt erhalten (Textur wird nur geleert)
    let to_despawn: Vec<Entity> = world
        .query_filtered::<Entity, Or<(With<Sprite>, With<Text2d>, With<BaseCrateSpawner>, With<ShaderExplosion>)>>()
        .iter(world)
        .filter(|e| !world.get::<GroundDecalLayer>(*e).is_some())
        .collect();

    for entity in to_despawn {
        if world.get_entity(entity).is_ok() {
            world.despawn(entity);
        }
    }

    // Decal-Map leeren
    if let Some(mut decal_map) = world.get_resource_mut::<crate::systems::ground_decals::GroundDecalMap>() {
        let handle = decal_map.image_handle.clone();
        decal_map.pending_stamps.clear();
        decal_map.dirty = false;
        if let Some(mut images) = world.get_resource_mut::<Assets<Image>>() {
            if let Some(image) = images.get_mut(&handle) {
                if let Some(data) = image.data.as_mut() {
                    data.fill(0);
                }
            }
        }
    }

    // Resources resetten
    *world.resource_mut::<Score>() = Score::default();
    *world.resource_mut::<WaveState>() = WaveState::default();
    *world.resource_mut::<ComboMeter>() = ComboMeter::default();
    *world.resource_mut::<crate::systems::weapons::UnlockedWeapons>() =
        crate::systems::weapons::UnlockedWeapons::default();

    // Neu spawnen ueber Commands + flush, damit alles sofort existiert
    let settings = world.resource::<GameSettings>().clone();
    {
        let mut commands = world.commands();
        crate::systems::room::do_setup_room(&mut commands);
        crate::systems::crates::do_setup_base_crates(&mut commands, &settings);
        crate::systems::player::do_spawn_players(&mut commands, &settings);
        setup_hud(commands);
    }
    world.flush();

    world.resource_mut::<NextState<GameState>>().set(GameState::Playing);
}


pub fn pause_toggle(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match state.get() {
            GameState::Playing => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Playing),
            GameState::UnlockScreen => next_state.set(GameState::Playing),
            _ => {}
        }
    }
    if keyboard.just_pressed(KeyCode::KeyM) {
        match state.get() {
            GameState::Playing => next_state.set(GameState::UnlockScreen),
            GameState::UnlockScreen => next_state.set(GameState::Playing),
            _ => {}
        }
    }
}
