use bevy::prelude::*;

use crate::resources::*;

#[derive(Component)]
pub struct LobbyUiRoot;

#[derive(Component)]
pub struct OnePlayerButton;

#[derive(Component)]
pub struct TwoPlayerButton;

#[derive(Component)]
pub struct PlayerCountLabel;

#[derive(Resource)]
pub struct LobbySelection {
    pub player_count: u32,
}

impl Default for LobbySelection {
    fn default() -> Self {
        Self { player_count: 1 }
    }
}

const BTN_NORMAL: Color = Color::srgb(0.25, 0.25, 0.3);
const BTN_HOVER: Color = Color::srgb(0.35, 0.35, 0.45);
const BTN_PRESSED: Color = Color::srgb(0.15, 0.15, 0.2);
const BTN_SELECTED: Color = Color::srgb(0.2, 0.6, 0.3);
const BTN_SELECTED_HOVER: Color = Color::srgb(0.25, 0.7, 0.35);

pub fn setup_lobby(mut commands: Commands) {
    commands.insert_resource(LobbySelection::default());

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
            LobbyUiRoot,
        ))
        .with_children(|root| {
            // Panel
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                padding: UiRect::all(Val::Px(40.0)),
                ..default()
            })
            .with_children(|panel| {
                // Titel
                panel.spawn((
                    Text::new("KLOTZKOEPFE"),
                    TextFont { font_size: 64.0, ..default() },
                    TextColor(Color::srgb(0.9, 0.2, 0.2)),
                ));

                // Untertitel
                panel.spawn((
                    Text::new("Zombie Survival"),
                    TextFont { font_size: 20.0, ..default() },
                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                ));

                // Spacer
                panel.spawn(Node {
                    height: Val::Px(20.0),
                    ..default()
                });

                // Spieler waehlen Label
                panel.spawn((
                    Text::new("Spieler waehlen:"),
                    TextFont { font_size: 24.0, ..default() },
                    TextColor(Color::WHITE),
                ));

                // Buttons Row
                panel.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    ..default()
                })
                .with_children(|row| {
                    // 1 Spieler Button (default selected)
                    spawn_lobby_button(row, "1 Spieler", OnePlayerButton, true);
                    // 2 Spieler Button
                    spawn_lobby_button(row, "2 Spieler", TwoPlayerButton, false);
                });

                // Spacer
                panel.spawn(Node {
                    height: Val::Px(10.0),
                    ..default()
                });

                // Start Hint
                panel.spawn((
                    Text::new("Enter / Leertaste zum Starten"),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                ));
            });
        });
}

fn spawn_lobby_button(parent: &mut ChildSpawnerCommands, label: &str, marker: impl Component, selected: bool) {
    let bg = if selected { BTN_SELECTED } else { BTN_NORMAL };
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(30.0), Val::Px(15.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg),
            marker,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont { font_size: 22.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
}

pub fn lobby_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut settings: ResMut<GameSettings>,
    mut lobby: ResMut<LobbySelection>,
    mut one_btn: Query<(&Interaction, &mut BackgroundColor), (With<OnePlayerButton>, Without<TwoPlayerButton>)>,
    mut two_btn: Query<(&Interaction, &mut BackgroundColor), (With<TwoPlayerButton>, Without<OnePlayerButton>)>,
) {
    // Keyboard: 1/2 zum Waehlen
    if keyboard.just_pressed(KeyCode::Digit1) {
        lobby.player_count = 1;
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        lobby.player_count = 2;
    }

    // Button Clicks
    if let Ok((interaction, _)) = one_btn.single() {
        if *interaction == Interaction::Pressed {
            lobby.player_count = 1;
        }
    }
    if let Ok((interaction, _)) = two_btn.single() {
        if *interaction == Interaction::Pressed {
            lobby.player_count = 2;
        }
    }

    // Button Visuals updaten
    if let Ok((interaction, mut bg)) = one_btn.single_mut() {
        *bg = button_color(lobby.player_count == 1, interaction);
    }
    if let Ok((interaction, mut bg)) = two_btn.single_mut() {
        *bg = button_color(lobby.player_count == 2, interaction);
    }

    // Enter/Space zum Starten
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        settings.player_count = lobby.player_count;
        next_state.set(GameState::Playing);
    }
}

fn button_color(selected: bool, interaction: &Interaction) -> BackgroundColor {
    BackgroundColor(match (selected, interaction) {
        (true, Interaction::Hovered) => BTN_SELECTED_HOVER,
        (true, Interaction::Pressed) => BTN_SELECTED,
        (true, _) => BTN_SELECTED,
        (false, Interaction::Hovered) => BTN_HOVER,
        (false, Interaction::Pressed) => BTN_PRESSED,
        (false, _) => BTN_NORMAL,
    })
}

pub fn cleanup_lobby(mut commands: Commands, query: Query<Entity, With<LobbyUiRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).try_despawn();
    }
}
