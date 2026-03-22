use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;

pub fn wave_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<GameSettings>,
    mut wave: ResMut<WaveState>,
    mut sound_events: ResMut<super::audio::SoundQueue>,
) {
    if wave.pausing {
        wave.pause_timer.tick(time.delta());
        if wave.pause_timer.is_finished() {
            wave.pausing = false;

            // Tote Spieler wiederbeleben (Duplikate entfernen)
            let mut dead = std::mem::take(&mut wave.dead_players);
            dead.dedup();
            for player_id in dead {
                respawn_player(&mut commands, &settings, player_id);
            }

            start_wave(&settings, &mut wave);
            sound_events.0.push(super::audio::SoundEvent::WaveStart);
        }
        return;
    }

    if !wave.active {
        start_wave(&settings, &mut wave);
        sound_events.0.push(super::audio::SoundEvent::WaveStart);
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

    // Calculate zombie count
    if wave.current_wave > settings.percent_mode_after_wave && wave.last_wave_zombies > 0 {
        // Percent mode: grow exponentially
        let increase = (wave.last_wave_zombies as f32 * settings.zombie_increase_percent / 100.0).ceil() as u32;
        wave.zombies_to_spawn = wave.last_wave_zombies + increase.max(1);
    } else {
        // Linear mode
        wave.zombies_to_spawn =
            settings.wave_base_zombies + (wave.current_wave - 1) * settings.wave_zombie_increment;
    }
    wave.last_wave_zombies = wave.zombies_to_spawn;

    // Spawn interval decreases per wave
    let interval = (settings.spawn_interval - settings.spawn_rate_decrease_per_wave * (wave.current_wave - 1) as f32)
        .max(settings.min_spawn_interval);
    wave.spawn_timer = Timer::from_seconds(interval, TimerMode::Repeating);
    wave.active = true;
}

fn respawn_player(commands: &mut Commands, settings: &GameSettings, id: PlayerId) {
    // Only respawn players within player_count
    match id {
        PlayerId::P1 => {},
        PlayerId::P2 if settings.player_count < 2 => return,
        PlayerId::P2 => {},
    }
    let (x, color, facing) = match id {
        PlayerId::P1 => (-80.0, PLAYER_COLOR_P1, Vec2::X),
        PlayerId::P2 => (80.0, PLAYER_COLOR_P2, Vec2::NEG_X),
    };
    crate::systems::player::spawn_one_player_pub(commands, settings, id, x, color, facing);
}
