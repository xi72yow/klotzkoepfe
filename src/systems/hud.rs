use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;

const AMMO_RECT_W: f32 = 5.0;
const AMMO_RECT_H: f32 = 12.0;
const AMMO_SPACING: f32 = 7.0;
const AMMO_COLOR_FULL: Color = Color::srgb(1.0, 0.9, 0.2);
const AMMO_COLOR_EMPTY: Color = Color::srgb(0.2, 0.2, 0.2);
const AMMO_COLOR_RELOAD: Color = Color::srgb(0.8, 0.3, 0.0);
const MAX_MAGAZINE: u32 = 30; // Uzi hat das groesste Magazin

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
    let p1_base_x = -WINDOW_WIDTH / 2.0 + 20.0;
    let ammo_y = -WINDOW_HEIGHT / 2.0 + 30.0;

    commands.spawn((
        Text2d::new("Pistole"),
        TextFont { font_size: 16.0, ..default() },
        TextColor(PLAYER_COLOR_P1),
        Transform::from_xyz(p1_base_x + 50.0, ammo_y + 18.0, 20.0),
        WeaponNameText(PlayerId::P1),
    ));

    // Patronen P1
    for i in 0..MAX_MAGAZINE {
        commands.spawn((
            Sprite {
                color: AMMO_COLOR_FULL,
                custom_size: Some(Vec2::new(AMMO_RECT_W, AMMO_RECT_H)),
                ..default()
            },
            Transform::from_xyz(p1_base_x + i as f32 * AMMO_SPACING, ammo_y, 20.0),
            AmmoIndicator { player_id: PlayerId::P1, index: i },
        ));
    }

    // Waffen-Name P2 (rechts unten)
    let p2_base_x = WINDOW_WIDTH / 2.0 - 20.0 - (MAX_MAGAZINE - 1) as f32 * AMMO_SPACING;

    commands.spawn((
        Text2d::new("Pistole"),
        TextFont { font_size: 16.0, ..default() },
        TextColor(PLAYER_COLOR_P2),
        Transform::from_xyz(p2_base_x + 50.0, ammo_y + 18.0, 20.0),
        WeaponNameText(PlayerId::P2),
    ));

    // Patronen P2
    for i in 0..MAX_MAGAZINE {
        commands.spawn((
            Sprite {
                color: AMMO_COLOR_FULL,
                custom_size: Some(Vec2::new(AMMO_RECT_W, AMMO_RECT_H)),
                ..default()
            },
            Transform::from_xyz(p2_base_x + i as f32 * AMMO_SPACING, ammo_y, 20.0),
            AmmoIndicator { player_id: PlayerId::P2, index: i },
        ));
    }
}

pub fn combo_system(
    time: Res<Time>,
    settings: Res<GameSettings>,
    mut combo: ResMut<ComboMeter>,
    mut score: ResMut<Score>,
) {
    if combo.position >= 1.0 {
        score.points += 1;
        combo.position = 0.5;
    }

    combo.position -= settings.combo_drain_speed * time.delta_secs();

    if combo.position <= 0.0 {
        if score.points > 0 {
            score.points -= 1;
        }
        combo.position = 0.5;
    }
}

pub fn update_hud(
    score: Res<Score>,
    wave: Res<WaveState>,
    combo: Res<ComboMeter>,
    settings: Res<GameSettings>,
    player_query: Query<&Player>,
    mut block_query: Query<&mut Transform, With<ComboBlock>>,
    mut score_text: Query<&mut Text2d, (With<ScoreText>, Without<WaveText>, Without<WeaponNameText>)>,
    mut wave_text: Query<&mut Text2d, (With<WaveText>, Without<ScoreText>, Without<WeaponNameText>)>,
    mut weapon_name: Query<(&mut Text2d, &WeaponNameText), (Without<ScoreText>, Without<WaveText>)>,
    mut ammo_query: Query<(&mut Sprite, &mut Visibility, &AmmoIndicator)>,
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
        **text = format!("Score: {}", score.points);
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
            **text = format!("{}: {}{}", prefix, player.weapon.name(), reload_str);
        } else {
            **text = String::new();
        }
    }

    // Munitions-Rechtecke aktualisieren
    for (mut sprite, mut vis, indicator) in ammo_query.iter_mut() {
        if let Some(player) = player_query.iter().find(|p| p.id == indicator.player_id) {
            let magazine = settings.weapon(player.weapon).magazine;

            if indicator.index >= magazine {
                *vis = Visibility::Hidden;
            } else {
                *vis = Visibility::Visible;
                if player.reloading {
                    sprite.color = AMMO_COLOR_RELOAD;
                } else if indicator.index < player.ammo {
                    sprite.color = AMMO_COLOR_FULL;
                } else {
                    sprite.color = AMMO_COLOR_EMPTY;
                }
            }
        } else {
            *vis = Visibility::Hidden;
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

pub fn restart_game(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<Score>,
    mut wave: ResMut<WaveState>,
    mut combo: ResMut<ComboMeter>,
    mut unlocked: ResMut<crate::systems::weapons::UnlockedWeapons>,
    settings: Res<GameSettings>,
    all_entities: Query<Entity, (Without<Camera>, Without<Window>)>,
) {
    for entity in all_entities.iter() {
        commands.entity(entity).try_despawn();
    }
    *score = Score::default();
    *wave = WaveState::default();
    *combo = ComboMeter::default();
    *unlocked = crate::systems::weapons::UnlockedWeapons::default();

    // Alles neu aufbauen
    crate::systems::room::setup_room(commands.reborrow());
    crate::systems::player::spawn_players(commands.reborrow(), settings);
    setup_hud(commands);

    next_state.set(GameState::Playing);
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
            _ => {}
        }
    }
}
