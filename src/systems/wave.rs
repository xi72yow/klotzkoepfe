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

            // Tote Spieler wiederbeleben (Duplikate entfernen)
            let mut dead = std::mem::take(&mut wave.dead_players);
            dead.dedup();
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
    crate::systems::player::spawn_one_player_pub(commands, settings, id, x, color, facing);
}
