use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;

#[derive(Component)]
pub struct UnlockUiRoot;

#[derive(Component)]
pub struct UnlockUi; // Legacy-Marker fuer restart_game

#[derive(Component)]
pub struct WeaponsTabButton;

#[derive(Component)]
pub struct ControlsTabButton;

#[derive(Component)]
pub struct UnlockContent;

#[derive(Resource, Default)]
pub struct UnlockUiState {
    pub tab: usize, // 0 = Waffen, 1 = Steuerung
}

const TAB_ACTIVE: Color = Color::srgb(0.15, 0.4, 0.2);
const TAB_NORMAL: Color = Color::srgb(0.2, 0.2, 0.25);
const TAB_HOVER: Color = Color::srgb(0.3, 0.3, 0.35);

pub fn setup_unlock_screen(
    mut commands: Commands,
    score: Res<Score>,
    settings: Res<GameSettings>,
    ui_state: Res<UnlockUiState>,
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
            UnlockUiRoot,
        ))
        .with_children(|root| {
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                width: Val::Px(850.0),
                max_height: Val::Percent(90.0),
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|panel| {
                // Tab Bar
                panel.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|row| {
                    spawn_tab_button(row, "Waffen", WeaponsTabButton, ui_state.tab == 0);
                    spawn_tab_button(row, "Steuerung", ControlsTabButton, ui_state.tab == 1);

                    // Spacer + Hint
                    row.spawn(Node { width: Val::Px(20.0), ..default() });
                    row.spawn((
                        Text::new("Tab: Wechseln | M/ESC: Schliessen"),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                        Node { align_self: AlignSelf::Center, ..default() },
                    ));
                });

                // Content Container
                panel.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        overflow: Overflow::scroll_y(),
                        max_height: Val::Px(550.0),
                        ..default()
                    },
                    UnlockContent,
                ))
                .with_children(|content| {
                    if ui_state.tab == 0 {
                        build_weapons_tab_ui(content, &score, &settings);
                    } else {
                        build_controls_tab_ui(content);
                    }
                });
            });
        });
}

fn spawn_tab_button(parent: &mut ChildSpawnerCommands, label: &str, marker: impl Component, active: bool) {
    let bg = if active { TAB_ACTIVE } else { TAB_NORMAL };
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(bg),
            marker,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
}

// --- Waffen-Tabelle ---

fn build_weapons_tab_ui(parent: &mut ChildSpawnerCommands, score: &Score, settings: &GameSettings) {
    // Score-Anzeige
    parent.spawn((
        Text::new(format!("Aktueller Score: {}", score.points)),
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgb(1.0, 1.0, 0.5)),
    ));

    // Tabellen-Header
    parent.spawn(Node {
        flex_direction: FlexDirection::Row,
        padding: UiRect::axes(Val::Px(4.0), Val::Px(4.0)),
        ..default()
    })
    .with_children(|row| {
        table_cell(row, "Waffe", 140.0, Color::srgb(0.7, 0.7, 0.7));
        table_cell(row, "Lv", 30.0, Color::srgb(0.7, 0.7, 0.7));
        table_cell(row, "Score", 70.0, Color::srgb(0.7, 0.7, 0.7));
        table_cell(row, "Dmg", 60.0, Color::srgb(0.7, 0.7, 0.7));
        table_cell(row, "CD", 60.0, Color::srgb(0.7, 0.7, 0.7));
        table_cell(row, "Mag", 50.0, Color::srgb(0.7, 0.7, 0.7));
        table_cell(row, "Status", 50.0, Color::srgb(0.7, 0.7, 0.7));
    });

    // Trennlinie
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
    ));

    // Waffen-Daten
    for weapon in WeaponType::all() {
        let ws = settings.weapon(*weapon);
        let unlocked = settings.gamemaster_level > 0 || ws.score_required <= score.points;
        let current_lvl = settings.weapon_level(*weapon, score.points);

        for lvl in 1..=3u32 {
            let ws_lvl = settings.weapon_at_level(*weapon, lvl);
            let active = unlocked && current_lvl == lvl;
            let score_req = match lvl {
                1 => ws.score_required,
                2 => ws.score_level_2,
                3 => ws.score_level_3,
                _ => 0,
            };
            let reached = settings.gamemaster_level > 0 || score.points >= score_req;

            let row_bg = if active {
                Color::srgba(0.0, 0.4, 0.0, 0.25)
            } else {
                Color::NONE
            };

            let name_color = if active {
                Color::srgb(0.2, 1.0, 0.2)
            } else if reached {
                Color::srgb(0.8, 0.8, 0.8)
            } else {
                Color::srgb(0.4, 0.4, 0.4)
            };

            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(row_bg),
            ))
            .with_children(|row| {
                table_cell(row, weapon.name_at_level(lvl), 140.0, name_color);
                table_cell(row, &format!("{}", lvl), 30.0, name_color);
                table_cell(row, &format!("{}", score_req), 70.0, name_color);
                table_cell(row, &format!("{:.0}", ws_lvl.damage), 60.0, name_color);
                table_cell(row, &format!("{:.2}", ws_lvl.cooldown), 60.0, name_color);
                table_cell(row, &format!("{}", ws_lvl.magazine), 50.0, name_color);

                let (status, status_color) = if reached {
                    ("OK", Color::srgb(0.2, 0.8, 0.2))
                } else {
                    ("--", Color::srgb(0.6, 0.2, 0.2))
                };
                table_cell(row, status, 50.0, status_color);
            });
        }

        // Trenner zwischen Waffen
        parent.spawn(Node { height: Val::Px(2.0), ..default() });
    }
}

fn table_cell(parent: &mut ChildSpawnerCommands, text: &str, width: f32, color: Color) {
    parent.spawn((
        Text::new(text),
        TextFont { font_size: 12.0, ..default() },
        TextColor(color),
        Node { width: Val::Px(width), ..default() },
    ));
}

// --- Steuerung / Tasten-Atlas ---

fn build_controls_tab_ui(parent: &mut ChildSpawnerCommands) {
    // Spieler 1
    section_header(parent, "Spieler 1", PLAYER_COLOR_P1);
    key_row(parent, &["W", "A", "S", "D"], "Bewegen");
    key_row(parent, &["Leertaste"], "Schiessen");
    key_row(parent, &["Q"], "Waffe wechseln");
    key_row(parent, &["E"], "Reload");

    parent.spawn(Node { height: Val::Px(12.0), ..default() });

    // Spieler 2
    section_header(parent, "Spieler 2", PLAYER_COLOR_P2);
    key_row(parent, &["\u{2191}", "\u{2190}", "\u{2193}", "\u{2192}"], "Bewegen");
    key_row(parent, &["Enter"], "Schiessen");
    key_row(parent, &["R-Shift"], "Waffe wechseln");
    key_row(parent, &["R-Ctrl"], "Reload");

    parent.spawn(Node { height: Val::Px(12.0), ..default() });

    // Allgemein
    section_header(parent, "Allgemein", Color::srgb(0.8, 0.8, 0.8));
    key_row(parent, &["ESC"], "Pause / Zurueck");
    key_row(parent, &["M"], "Waffenuebersicht");
    key_row(parent, &["F11"], "Fullscreen");
    key_row(parent, &["F5"], "Settings speichern");
    key_row(parent, &["F6"], "Defaults laden");
    key_row(parent, &["1", "2"], "Spieleranzahl (Lobby)");
}

fn section_header(parent: &mut ChildSpawnerCommands, label: &str, color: Color) {
    parent.spawn((
        Text::new(format!("--- {} ---", label)),
        TextFont { font_size: 18.0, ..default() },
        TextColor(color),
        Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() },
    ));
}

fn key_row(parent: &mut ChildSpawnerCommands, keys: &[&str], description: &str) {
    parent.spawn(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(6.0),
        margin: UiRect::bottom(Val::Px(3.0)),
        ..default()
    })
    .with_children(|row| {
        // Keys
        row.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(3.0),
            width: Val::Px(200.0),
            justify_content: JustifyContent::FlexEnd,
            ..default()
        })
        .with_children(|keys_container| {
            for key in keys {
                spawn_key_cap(keys_container, key);
            }
        });

        // Description
        row.spawn((
            Text::new(description),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
        ));
    });
}

fn spawn_key_cap(parent: &mut ChildSpawnerCommands, label: &str) {
    parent
        .spawn((
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                min_width: Val::Px(28.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
            BorderColor::from(Color::srgb(0.4, 0.4, 0.5)),
        ))
        .with_children(|cap| {
            cap.spawn((
                Text::new(label),
                TextFont { font_size: 12.0, ..default() },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });
}

// --- Input ---

pub fn unlock_screen_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ui_state: ResMut<UnlockUiState>,
    mut commands: Commands,
    root_query: Query<Entity, With<UnlockUiRoot>>,
    score: Res<Score>,
    settings: Res<GameSettings>,
    weapons_btn: Query<(&Interaction, &WeaponsTabButton), Changed<Interaction>>,
    controls_btn: Query<(&Interaction, &ControlsTabButton), Changed<Interaction>>,
) {
    let mut switch_tab = None;

    // Keyboard Tab-Wechsel
    if keyboard.just_pressed(KeyCode::Tab) {
        switch_tab = Some((ui_state.tab + 1) % 2);
    }

    // Button-Clicks
    for (interaction, _) in weapons_btn.iter() {
        if *interaction == Interaction::Pressed && ui_state.tab != 0 {
            switch_tab = Some(0);
        }
    }
    for (interaction, _) in controls_btn.iter() {
        if *interaction == Interaction::Pressed && ui_state.tab != 1 {
            switch_tab = Some(1);
        }
    }

    if let Some(new_tab) = switch_tab {
        ui_state.tab = new_tab;
        // Komplett neu aufbauen
        for entity in root_query.iter() {
            commands.entity(entity).try_despawn();
        }
        // Inline rebuild
        let tab = ui_state.tab;
        commands
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
                UnlockUiRoot,
            ))
            .with_children(|root| {
                root.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Px(850.0),
                    max_height: Val::Percent(90.0),
                    padding: UiRect::all(Val::Px(20.0)),
                    row_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|panel| {
                    panel.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_tab_button(row, "Waffen", WeaponsTabButton, tab == 0);
                        spawn_tab_button(row, "Steuerung", ControlsTabButton, tab == 1);
                        row.spawn(Node { width: Val::Px(20.0), ..default() });
                        row.spawn((
                            Text::new("Tab: Wechseln | M/ESC: Schliessen"),
                            TextFont { font_size: 12.0, ..default() },
                            TextColor(Color::srgb(0.5, 0.5, 0.5)),
                            Node { align_self: AlignSelf::Center, ..default() },
                        ));
                    });
                    panel.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            overflow: Overflow::scroll_y(),
                            max_height: Val::Px(550.0),
                            ..default()
                        },
                        UnlockContent,
                    ))
                    .with_children(|content| {
                        if tab == 0 {
                            build_weapons_tab_ui(content, &score, &settings);
                        } else {
                            build_controls_tab_ui(content);
                        }
                    });
                });
            });
    }
}

pub fn cleanup_unlock_screen(
    mut commands: Commands,
    query: Query<Entity, With<UnlockUiRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).try_despawn();
    }
}
