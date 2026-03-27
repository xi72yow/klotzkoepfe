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
    embedded_asset!(app, "shaders/cone_beam.wgsl");
    embedded_asset!(app, "shaders/elemental_overlay.wgsl");
    app.insert_resource(ClearColor(FLOOR_COLOR))
        .add_plugins(bevy::sprite_render::Material2dPlugin::<explosion_fx::ExplosionMaterial>::default())
        .add_plugins(bevy::sprite_render::Material2dPlugin::<explosion_fx::MuzzleFlashMaterial>::default())
        .add_plugins(bevy::sprite_render::Material2dPlugin::<cone_beam::ConeBeamMaterial>::default())
        .add_plugins(bevy::sprite_render::Material2dPlugin::<elemental_overlay::ElementalOverlayMaterial>::default())
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
        // Startup: nur globale Systeme (kein Spieler/HUD/Crates - das kommt bei OnEnter(Playing))
        .add_systems(
            Startup,
            (
                pixelation::setup_pixelation,
                room::setup_room,
                ground_decals::setup_ground_decals,
                audio::setup_audio,
            ),
        )
        // Lobby
        .add_systems(OnEnter(GameState::Lobby), lobby_ui::setup_lobby)
        .add_systems(Update, lobby_ui::lobby_input.run_if(in_state(GameState::Lobby)))
        .add_systems(OnExit(GameState::Lobby), lobby_ui::cleanup_lobby)
        // Spielstart: Spieler, HUD und Kisten spawnen
        .add_systems(OnEnter(GameState::Playing), start_playing)
        // Restart nach Game Over: exclusive system despawnt sofort, dann neu spawnen
        .add_systems(
            OnEnter(GameState::Restarting),
            hud::restart_game,
        )
        // Pause-Toggle, Fullscreen, Pixelation und Audio laufen immer
        .add_systems(Update, (hud::pause_toggle, fullscreen_toggle, pixelation::update_pixelation, audio::play_sounds))
        // Settings-UI nur im Pause-State
        .add_systems(OnEnter(GameState::Paused), debug_ui::setup_pause_ui)
        .add_systems(
            Update,
            (
                debug_ui::settings_input,
                debug_ui::settings_update_ui,
                debug_ui::settings_button_interaction,
                player::gm_weapon_apply,
                debug_ui::gm_wave_apply,
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
                crates::flare_system,
                crates::smoke_system,
                crates::airdrop_system,
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
                zombie::zombie_elemental_visuals,
                zombie::zombie_freeze_thaw,
                zombie::zombie_ash_death,
                zombie::zombie_ash_crumble,
                zombie::ash_particle_update,
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
                cone_beam::cone_beam_spawn,
                cone_beam::cone_beam_despawn,
                cone_beam::cone_beam_update,
                cone_beam::cone_beam_damage,
                cone_beam::cone_beam_debug_gizmos,
                debug_ui::grid_overlay_gizmos,
                debug_ui::weapon_range_gizmos,
                debug_ui::hitbox_gizmos,
                elemental_overlay::elemental_overlay_spawn,
                elemental_overlay::elemental_overlay_update,
                elemental_overlay::elemental_overlay_despawn,
            )
                .run_if(in_state(GameState::Playing)),
        )
        // Game Over
        .add_systems(OnEnter(GameState::GameOver), hud::setup_game_over)
        .add_systems(
            Update,
            hud::game_over_input.run_if(in_state(GameState::GameOver)),
        )
        .add_systems(OnExit(GameState::GameOver), hud::cleanup_game_over)
        .add_systems(Update, apply_game_speed)
        .run();
}

fn apply_game_speed(
    settings: Res<GameSettings>,
    mut time: ResMut<Time<Virtual>>,
) {
    let speed = settings.gm_game_speed.clamp(0.1, 4.0);
    if (time.relative_speed() - speed).abs() > 0.01 {
        time.set_relative_speed(speed);
    }
}

fn start_playing(
    mut commands: Commands,
    settings: Res<GameSettings>,
    mut wave: ResMut<resources::WaveState>,
    existing_players: Query<&components::Player>,
) {
    // Nur spawnen wenn noch keine Spieler existieren
    // (restart_game spawnt selbst, und Pause/Unlock kommen zurueck ohne Neuspawn)
    if existing_players.iter().count() > 0 {
        return;
    }
    // Gamemaster: Startwelle setzen
    if settings.gm_start_wave > 0 {
        wave.current_wave = settings.gm_start_wave.saturating_sub(1);
    }
    player::do_spawn_players(&mut commands, &settings);
    hud::setup_hud(commands.reborrow());
    crates::do_setup_base_crates(&mut commands, &settings);
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
