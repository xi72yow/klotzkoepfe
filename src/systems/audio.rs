use bevy::prelude::*;
use bevy::audio::AudioSource;
use std::sync::Arc;
use crate::components::WeaponType;
use crate::resources::GameSettings;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExplosionType {
    Grenade,
    Rocket,
    Mine,
}

#[derive(Clone)]
pub enum SoundEvent {
    Shoot(WeaponType),
    Reload,
    Explosion(ExplosionType),
    CratePickup,
    ZombieDeath,
    ZombieGroan(u8),
    PlayerDamage,
    WaveStart,
    WeaponSwitch,
}

#[derive(Resource)]
pub struct GameAudio {
    pub shoot_pistol: Handle<AudioSource>,
    pub shoot_uzi: Handle<AudioSource>,
    pub shoot_shotgun: Handle<AudioSource>,
    pub shoot_rocket: Handle<AudioSource>,
    pub reload: Handle<AudioSource>,
    pub explosion_grenade: Vec<Handle<AudioSource>>,
    pub explosion_rocket: Vec<Handle<AudioSource>>,
    pub explosion_mine: Vec<Handle<AudioSource>>,
    pub zombie_groans: Vec<Handle<AudioSource>>,
    pub zombie_deaths: Vec<Handle<AudioSource>>,
}

fn load_wav(bytes: &'static [u8]) -> Arc<[u8]> {
    Arc::from(bytes)
}

pub fn setup_audio(
    mut commands: Commands,
    mut audio_sources: ResMut<Assets<AudioSource>>,
) {
    let game_audio = GameAudio {
        shoot_pistol: audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/pistol_shot.wav")) }),
        shoot_uzi: audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/uzi_shot.wav")) }),
        shoot_shotgun: audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/shotgun_shot.wav")) }),
        shoot_rocket: audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/rocket_shot.wav")) }),
        reload: audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/reload.wav")) }),
        explosion_grenade: vec![
            audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/explosion_small.wav")) }),
        ],
        explosion_rocket: vec![
            audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/explosion_big.wav")) }),
        ],
        explosion_mine: vec![
            audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/explosion_rumble.wav")) }),
        ],
        zombie_groans: vec![
            audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/zombie_groan_1.wav")) }),
            audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/zombie_groan_2.wav")) }),
            audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/zombie_groan_3.wav")) }),
            audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/zombie_extra_14.wav")) }),
            audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/zombie_extra_15.wav")) }),
            audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/zombie_extra_16.wav")) }),
        ],
        zombie_deaths: vec![
            audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/zombie_death_1.wav")) }),
            audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/zombie_death_2.wav")) }),
            audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/zombie_death_3.wav")) }),
            audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/zombie_extra_17.wav")) }),
            audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/zombie_extra_18.wav")) }),
            audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/zombie_extra_19.wav")) }),
        ],
    };
    commands.insert_resource(game_audio);
}

#[derive(Resource, Default)]
pub struct SoundQueue(pub Vec<SoundEvent>);

fn pseudo_random() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos()
}

pub fn play_sounds(
    mut commands: Commands,
    mut queue: ResMut<SoundQueue>,
    audio: Res<GameAudio>,
    settings: Res<GameSettings>,
) {
    let vol = settings.volume;
    if vol <= 0.0 { queue.0.clear(); return; }
    let events: Vec<SoundEvent> = queue.0.drain(..).collect();

    // Kategorie-Lautstaerke aus Settings
    let vol_weapons = settings.vol_weapons;
    let vol_enemies = settings.vol_enemies;
    let vol_player = settings.vol_player;

    // Sound-Stacking-Limiter: max Sounds pro Typ pro Frame
    const MAX_ZOMBIE_DEATHS: u32 = 2;
    const MAX_ZOMBIE_GROANS: u32 = 2;
    const MAX_EXPLOSIONS: u32 = 2;
    const MAX_SHOOTS: u32 = 3;
    let mut zombie_death_count: u32 = 0;
    let mut zombie_groan_count: u32 = 0;
    let mut explosion_count: u32 = 0;
    let mut shoot_count: u32 = 0;

    for event in events.iter() {
        let (handle, vol_scale, category) = match event {
            SoundEvent::Shoot(w) => {
                shoot_count += 1;
                if shoot_count > MAX_SHOOTS { continue; }
                match w {
                    WeaponType::Pistol => (&audio.shoot_pistol, 0.5, vol_weapons),
                    WeaponType::Uzi => (&audio.shoot_uzi, 0.4, vol_weapons),
                    WeaponType::Shotgun => (&audio.shoot_shotgun, 0.5, vol_weapons),
                    WeaponType::Rocket => (&audio.shoot_rocket, 0.5, vol_weapons),
                    WeaponType::Railgun => (&audio.shoot_pistol, 0.4, vol_weapons),
                    WeaponType::Grenade => continue,
                    _ => continue,
                }
            },
            SoundEvent::Reload => (&audio.reload, 0.5, vol_weapons),
            SoundEvent::Explosion(etype) => {
                explosion_count += 1;
                if explosion_count > MAX_EXPLOSIONS { continue; }
                let pool = match etype {
                    ExplosionType::Grenade => &audio.explosion_grenade,
                    ExplosionType::Rocket => &audio.explosion_rocket,
                    ExplosionType::Mine => &audio.explosion_mine,
                };
                let idx = pseudo_random() as usize % pool.len();
                (&pool[idx], 0.6, vol_weapons)
            },
            SoundEvent::ZombieGroan(_) => {
                zombie_groan_count += 1;
                if zombie_groan_count > MAX_ZOMBIE_GROANS { continue; }
                let idx = pseudo_random() as usize % audio.zombie_groans.len();
                (&audio.zombie_groans[idx], 0.3, vol_enemies)
            },
            SoundEvent::ZombieDeath => {
                zombie_death_count += 1;
                if zombie_death_count > MAX_ZOMBIE_DEATHS { continue; }
                let idx = pseudo_random() as usize % audio.zombie_deaths.len();
                (&audio.zombie_deaths[idx], 0.4, vol_enemies)
            },
            SoundEvent::CratePickup => continue, // TODO: Sound hinzufuegen
            SoundEvent::PlayerDamage => continue, // TODO: Sound hinzufuegen
            SoundEvent::WaveStart => continue, // TODO: Sound hinzufuegen
            SoundEvent::WeaponSwitch => continue, // TODO: Sound hinzufuegen
        };
        commands.spawn((
            AudioPlayer::new(handle.clone()),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                volume: bevy::audio::Volume::Linear(vol * category * vol_scale),
                ..default()
            },
        ));
    }
}
