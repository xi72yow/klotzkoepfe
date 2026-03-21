use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;

#[derive(Component)]
pub struct UnlockUi;

#[derive(Resource, Default)]
pub struct UnlockUiState {
    pub tab: usize, // 0 = Waffen, 1 = Steuerung
}

pub fn setup_unlock_screen(
    mut commands: Commands,
    score: Res<Score>,
    settings: Res<GameSettings>,
    ui_state: Res<UnlockUiState>,
) {
    render_unlock_screen(&mut commands, &score, &settings, &ui_state);
}

fn build_weapons_tab(lines: &mut Vec<String>, score: &Score, settings: &GameSettings) {
    lines.push("=== WAFFENARSENAL ===".into());
    lines.push("Tab: Steuerung | M/ESC: Schliessen".into());
    lines.push(format!("Aktueller Score: {}", score.points));
    lines.push(String::new());

    for weapon in WeaponType::all() {
        let ws = settings.weapon(*weapon);
        let unlocked = settings.gamemaster_level > 0 || ws.score_required <= score.points;
        let current_lvl = settings.weapon_level(*weapon, score.points);

        // Weapon header
        let marker = if unlocked { "+" } else { "-" };
        let unlock_str = if ws.score_required == 0 {
            "Start".into()
        } else {
            format!("ab {} Pkt", ws.score_required)
        };
        lines.push(format!(
            "{} {} ({})",
            marker,
            weapon.name_at_level(current_lvl),
            unlock_str
        ));

        // Show all 3 levels with score requirements and stats
        for lvl in 1..=3u32 {
            let ws_lvl = settings.weapon_at_level(*weapon, lvl);
            let active = unlocked && current_lvl == lvl;
            let prefix = if active { ">>" } else { "  " };

            let score_req = match lvl {
                1 => ws.score_required,
                2 => ws.score_level_2,
                3 => ws.score_level_3,
                _ => 0,
            };

            let reached = settings.gamemaster_level > 0 || score.points >= score_req;
            let status = if reached { " " } else { "X" };

            lines.push(format!(
                "  {} Lv{} {:<14} {:>4} Pkt | Dmg:{:.0} CD:{:.2} Mag:{} [{}]",
                prefix,
                lvl,
                weapon.name_at_level(lvl),
                score_req,
                ws_lvl.damage,
                ws_lvl.cooldown,
                ws_lvl.magazine,
                status
            ));
        }
    }
}

fn build_controls_tab(lines: &mut Vec<String>) {
    lines.push("=== STEUERUNG ===".into());
    lines.push("Tab: Waffen | M/ESC: Schliessen".into());
    lines.push(String::new());
    lines.push("--- Spieler 1 ---".into());
    lines.push("  WASD          Bewegen".into());
    lines.push("  Leertaste     Schiessen".into());
    lines.push("  Q             Waffe wechseln".into());
    lines.push(String::new());
    lines.push("--- Spieler 2 ---".into());
    lines.push("  Pfeiltasten   Bewegen".into());
    lines.push("  Enter         Schiessen".into());
    lines.push("  R-Shift       Waffe wechseln".into());
    lines.push(String::new());
    lines.push("--- Allgemein ---".into());
    lines.push("  ESC           Pause / Zurueck".into());
    lines.push("  M             Waffenuebersicht".into());
    lines.push("  F11           Fullscreen".into());
    lines.push("  F5            Settings speichern".into());
    lines.push("  F6            Defaults laden".into());
}

pub fn unlock_screen_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ui_state: ResMut<UnlockUiState>,
    mut commands: Commands,
    query: Query<Entity, With<UnlockUi>>,
    score: Res<Score>,
    settings: Res<GameSettings>,
) {
    if keyboard.just_pressed(KeyCode::Tab) {
        ui_state.tab = (ui_state.tab + 1) % 2;
        // Re-render: despawn old, then rebuild
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
        // Render inline since we can't pass ResMut as Res
        render_unlock_screen(&mut commands, &score, &settings, &ui_state);
    }
}

fn render_unlock_screen(
    commands: &mut Commands,
    score: &Score,
    settings: &GameSettings,
    ui_state: &UnlockUiState,
) {
    // Background
    commands.spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.85),
            custom_size: Some(Vec2::new(900.0, 650.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 49.0),
        UnlockUi,
    ));

    let mut lines: Vec<String> = Vec::new();

    if ui_state.tab == 0 {
        build_weapons_tab(&mut lines, score, settings);
    } else {
        build_controls_tab(&mut lines);
    }

    let text = lines.join("\n");

    commands.spawn((
        Text2d::new(text),
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Transform::from_xyz(0.0, 0.0, 50.0),
        UnlockUi,
    ));
}

pub fn cleanup_unlock_screen(
    mut commands: Commands,
    query: Query<Entity, With<UnlockUi>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
