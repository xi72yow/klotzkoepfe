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
    PlayerDeath,
    WaveStart,
    WeaponSwitch,
    IceFreeze,
}

#[derive(Resource)]
pub struct GameAudio {
    pub shoot_pistol: Handle<AudioSource>,
    pub shoot_uzi: Handle<AudioSource>,
    pub shoot_shotgun: Handle<AudioSource>,
    pub shoot_rocket: Handle<AudioSource>,
    pub reloads: Vec<Handle<AudioSource>>,
    pub shoot_freeze: Handle<AudioSource>,
    pub shoot_flamethrower: Handle<AudioSource>,
    pub explosion_grenade: Vec<Handle<AudioSource>>,
    pub explosion_rocket: Vec<Handle<AudioSource>>,
    pub explosion_mine: Vec<Handle<AudioSource>>,
    pub zombie_groans: Vec<Handle<AudioSource>>,
    pub zombie_deaths: Vec<Handle<AudioSource>>,
    pub player_hurts: Vec<Handle<AudioSource>>,
    pub player_death: Handle<AudioSource>,
    pub ice_freezes: Vec<Handle<AudioSource>>,
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
        reloads: vec![
            audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/reload_gunreload1.wav")) }),
            audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/reload_assaultriflereload1.wav")) }),
            audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/reload_handgun_reload.wav")) }),
        ],
        shoot_freeze: audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/freeze_shot.wav")) }),
        shoot_flamethrower: audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/flamethrower_loop.wav")) }),
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
        player_hurts: vec![
            audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/player_hurt_1.wav")) }),
            audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/player_hurt_2.wav")) }),
            audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/player_hurt_3.wav")) }),
            audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/player_hurt_4.wav")) }),
        ],
        player_death: audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/player_death.wav")) }),
        ice_freezes: vec![
            audio_sources.add(AudioSource { bytes: load_wav(include_bytes!("../sounds/ice_freeze_snap.wav")) }),
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
    let vol = settings.volume.powf(4.6); // steile Kurve: 50% Slider ≈ 0.04 effektiv
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
    const MAX_PLAYER_HURTS: u32 = 1;
    const MAX_ICE_FREEZES: u32 = 6;
    let mut zombie_death_count: u32 = 0;
    let mut zombie_groan_count: u32 = 0;
    let mut explosion_count: u32 = 0;
    let mut shoot_count: u32 = 0;
    let mut player_hurt_count: u32 = 0;
    let mut ice_freeze_count: u32 = 0;

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
                    WeaponType::FreezeGun | WeaponType::Flamethrower => continue, // Loop-Sounds im Player-System
                    WeaponType::Grenade => continue,
                    _ => continue,
                }
            },
            SoundEvent::Reload => {
                let idx = pseudo_random() as usize % audio.reloads.len();
                (&audio.reloads[idx], 0.5, vol_weapons)
            },
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
            SoundEvent::PlayerDamage => {
                player_hurt_count += 1;
                if player_hurt_count > MAX_PLAYER_HURTS { continue; }
                let idx = pseudo_random() as usize % audio.player_hurts.len();
                (&audio.player_hurts[idx], 0.5, vol_player)
            },
            SoundEvent::PlayerDeath => (&audio.player_death, 0.7, vol_player),
            SoundEvent::IceFreeze => {
                ice_freeze_count += 1;
                if ice_freeze_count > MAX_ICE_FREEZES { continue; }
                let idx = pseudo_random() as usize % audio.ice_freezes.len();
                (&audio.ice_freezes[idx], 0.3, vol_weapons)
            },
            SoundEvent::CratePickup => continue, // TODO: Sound hinzufuegen
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
