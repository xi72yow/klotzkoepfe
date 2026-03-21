use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;

pub fn wave_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<GameSettings>,
    mut wave: ResMut<WaveState>,
) {
    if wave.pausing {
        wave.pause_timer.tick(time.delta());
        if wave.pause_timer.is_finished() {
            wave.pausing = false;

            // Tote Spieler wiederbeleben
            let dead = std::mem::take(&mut wave.dead_players);
            for player_id in dead {
                respawn_player(&mut commands, &settings, player_id);
            }

            start_wave(&settings, &mut wave);
        }
        return;
    }

    if !wave.active {
        start_wave(&settings, &mut wave);
        return;
    }

    if wave.zombies_alive == 0 && wave.zombies_to_spawn == 0 {
        wave.active = false;
        wave.pausing = true;
        wave.pause_timer = Timer::from_seconds(settings.wave_pause, TimerMode::Once);
    }
}

fn start_wave(settings: &GameSettings, wave: &mut WaveState) {
    wave.current_wave += 1;
    wave.zombies_to_spawn =
        settings.wave_base_zombies + (wave.current_wave - 1) * settings.wave_zombie_increment;
    wave.spawn_timer = Timer::from_seconds(settings.spawn_interval, TimerMode::Repeating);
    wave.active = true;
}

fn respawn_player(commands: &mut Commands, settings: &GameSettings, id: PlayerId) {
    let (x, color, facing) = match id {
        PlayerId::P1 => (-80.0, PLAYER_COLOR_P1, Vec2::X),
        PlayerId::P2 => (80.0, PLAYER_COLOR_P2, Vec2::NEG_X),
    };

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
