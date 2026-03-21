mod components;
mod constants;
mod resources;
mod systems;

use bevy::prelude::*;

use constants::*;
use resources::*;
use systems::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Klotzkoepfe".to_string(),
                resolution: bevy::window::WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(FLOOR_COLOR))
        .init_state::<GameState>()
        .init_resource::<WaveState>()
        .init_resource::<Score>()
        .init_resource::<ComboMeter>()
        .init_resource::<debug_ui::SettingsUiState>()
        .init_resource::<weapons::UnlockedWeapons>()
        .insert_resource(GameSettings::load())
        // Erster Spielstart
        .add_systems(
            Startup,
            (
                setup_camera,
                room::setup_room,
                player::spawn_players,
                hud::setup_hud,
            ),
        )
        // Restart nach Game Over: exclusive system despawnt sofort, dann neu spawnen
        .add_systems(
            OnEnter(GameState::Restarting),
            (hud::restart_despawn, hud::restart_spawn).chain(),
        )
        // Pause-Toggle laeuft immer
        .add_systems(Update, hud::pause_toggle)
        // Settings-UI nur im Pause-State
        .add_systems(
            Update,
            (
                debug_ui::settings_input,
                debug_ui::settings_render,
            )
                .run_if(in_state(GameState::Paused)),
        )
        .add_systems(OnExit(GameState::Paused), debug_ui::cleanup_settings_panel)
        // Gameplay-Systeme (aufgeteilt wegen Bevy max 20 pro Tuple)
        .add_systems(
            Update,
            (
                player::player_movement,
                player::player_weapon_switch,
                player::player_shoot,
                player::update_player_hp_bars,
                player::update_weapon_sprites,
                bullet::bullet_movement,
                bullet::grenade_movement,
                bullet::rocket_movement,
                bullet::explosion_update,
                zombie::zombie_spawn,
                zombie::zombie_ai,
                zombie::zombie_separation,
                collision::bullet_zombie_collision,
                collision::explosion_zombie_collision,
                collision::bullet_player_collision,
                collision::zombie_player_collision,
                blood::blood_update,
                wave::wave_system,
                hud::combo_system,
                hud::update_hud,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                weapons::mine_system,
                weapons::boomerang_system,
                weapons::spinning_system,
                weapons::zombie_freeze_update,
                weapons::weapon_unlock_check,
                weapons::weapon_unlock_fade,
                weapons::drop_pickup,
            )
                .run_if(in_state(GameState::Playing)),
        )
        // Game Over
        .add_systems(OnEnter(GameState::GameOver), hud::setup_game_over)
        .add_systems(
            Update,
            hud::game_over_input.run_if(in_state(GameState::GameOver)),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
