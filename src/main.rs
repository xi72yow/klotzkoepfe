mod components;
mod constants;
mod resources;
mod systems;

use bevy::asset::embedded_asset;
use bevy::prelude::*;

use constants::*;
use resources::*;
use systems::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Klotzkoepfe".to_string(),
                resolution: bevy::window::WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32),
                resizable: false,
                ..default()
            }),
            ..default()
        }));
    embedded_asset!(app, "shaders/explosion.wgsl");
    embedded_asset!(app, "shaders/pixelation.wgsl");
    embedded_asset!(app, "shaders/muzzle_flash.wgsl");
    app.insert_resource(ClearColor(FLOOR_COLOR))
        .add_plugins(bevy::sprite_render::Material2dPlugin::<explosion_fx::ExplosionMaterial>::default())
        .add_plugins(bevy::sprite_render::Material2dPlugin::<explosion_fx::MuzzleFlashMaterial>::default())
        .add_plugins(bevy::core_pipeline::fullscreen_material::FullscreenMaterialPlugin::<pixelation::PixelationMaterial>::default())
        .init_state::<GameState>()
        .init_resource::<WaveState>()
        .init_resource::<Score>()
        .init_resource::<ComboMeter>()
        .init_resource::<debug_ui::SettingsUiState>()
        .init_resource::<unlock_ui::UnlockUiState>()
        .init_resource::<weapons::UnlockedWeapons>()
        .insert_resource(GameSettings::load())
        .init_resource::<audio::SoundQueue>()
        // Erster Spielstart
        .add_systems(
            Startup,
            (
                pixelation::setup_pixelation,
                room::setup_room,
                player::spawn_players,
                hud::setup_hud,
                crates::setup_base_crates,
                ground_decals::setup_ground_decals,
                audio::setup_audio,
            ),
        )
        // Restart nach Game Over: exclusive system despawnt sofort, dann neu spawnen
        .add_systems(
            OnEnter(GameState::Restarting),
            hud::restart_game,
        )
        // Pause-Toggle, Fullscreen, Pixelation und Audio laufen immer
        .add_systems(Update, (hud::pause_toggle, fullscreen_toggle, pixelation::update_pixelation, audio::play_sounds))
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
        // Unlock-Screen
        .add_systems(OnEnter(GameState::UnlockScreen), unlock_ui::setup_unlock_screen)
        .add_systems(
            Update,
            unlock_ui::unlock_screen_input.run_if(in_state(GameState::UnlockScreen)),
        )
        .add_systems(OnExit(GameState::UnlockScreen), unlock_ui::cleanup_unlock_screen)
        // Gameplay-Systeme (aufgeteilt wegen Bevy max 20 pro Tuple)
        .add_systems(
            Update,
            (
                player::player2_join,
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
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                wave::wave_system,
                hud::combo_system,
                hud::update_hud,
                weapons::mine_system,
                weapons::boomerang_system,
                weapons::spinning_system,
                weapons::zombie_freeze_update,
                weapons::weapon_unlock_check,
                weapons::weapon_unlock_fade,
                player::player_walk_animation,
                player::player_regeneration,
                collision::explosion_player_collision,
                collision::apply_knockback,
                crates::crate_system,
                crates::base_crate_respawn,
                zombie::zombie_animation,
                blood::gib_update,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                zombie::burning_system,
                zombie::stun_system,
                zombie::freeze_stack_system,
                zombie::lightning_arc_system,
                zombie::zombie_groan,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                explosion_fx::update_shader_explosions,
                explosion_fx::update_muzzle_flashes,
                ground_decals::process_decal_stamps,
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

fn fullscreen_toggle(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<GameSettings>,
    mut windows: Query<&mut Window>,
) {
    if keyboard.just_pressed(KeyCode::F11) {
        settings.fullscreen = !settings.fullscreen;
    }
    if let Ok(mut window) = windows.single_mut() {
        let target_mode = if settings.fullscreen {
            bevy::window::WindowMode::BorderlessFullscreen(bevy::window::MonitorSelection::Current)
        } else {
            bevy::window::WindowMode::Windowed
        };
        if window.mode != target_mode {
            window.mode = target_mode;
        }
    }
}
