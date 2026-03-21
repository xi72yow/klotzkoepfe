use bevy::prelude::*;

use crate::components::WeaponType;

use crate::components::PlayerId;

#[derive(Resource)]
pub struct WaveState {
    pub current_wave: u32,
    pub zombies_to_spawn: u32,
    pub zombies_alive: u32,
    pub spawn_timer: Timer,
    pub active: bool,
    pub pause_timer: Timer,
    pub pausing: bool,
    pub dead_players: Vec<PlayerId>,
}

impl Default for WaveState {
    fn default() -> Self {
        Self {
            current_wave: 0,
            zombies_to_spawn: 0,
            zombies_alive: 0,
            spawn_timer: Timer::from_seconds(
                crate::constants::SPAWN_INTERVAL,
                TimerMode::Repeating,
            ),
            active: false,
            pause_timer: Timer::from_seconds(crate::constants::WAVE_PAUSE, TimerMode::Once),
            pausing: false,
            dead_players: Vec::new(),
        }
    }
}

#[derive(Resource, Default)]
pub struct Score {
    pub kills: u32,
    pub points: i32,
}

#[derive(Resource)]
pub struct ComboMeter {
    pub position: f32,
}

impl Default for ComboMeter {
    fn default() -> Self {
        Self { position: 0.5 }
    }
}

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    #[default]
    Playing,
    Paused,
    GameOver,
    Restarting,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct WeaponSettings {
    pub cooldown: f32,
    pub magazine: u32,
    pub reload_time: f32,
    pub range: f32,
    pub damage: f32,
    pub bullet_speed: f32,
    pub score_required: i32,
    // Waffen-spezifisch (defaults fuer alte Settings-Dateien)
    #[serde(default)]
    pub pellet_count: u32,
    #[serde(default)]
    pub spread_angle: f32,
    #[serde(default)]
    pub chain_count: u32,
    #[serde(default)]
    pub chain_range: f32,
    #[serde(default)]
    pub slow_factor: f32,
    #[serde(default)]
    pub slow_duration: f32,
    #[serde(default)]
    pub explosion_radius_override: f32,
    #[serde(default)]
    pub trigger_radius: f32,
}

#[derive(Resource, Clone, serde::Serialize, serde::Deserialize)]
pub struct GameSettings {
    #[serde(skip)]
    pub show_debug: bool,

    pub player_speed: f32,
    pub player_hp: f32,

    pub zombie_speed: f32,
    pub zombie_hp: f32,
    pub zombie_damage: f32,
    pub zombie_damage_cooldown: f32,

    pub combo_drain_speed: f32,
    pub combo_kill_boost: f32,

    pub wave_base_zombies: u32,
    pub wave_zombie_increment: u32,
    pub spawn_interval: f32,
    pub wave_pause: f32,

    pub pistol: WeaponSettings,
    pub uzi: WeaponSettings,
    pub grenade: WeaponSettings,
    pub railgun: WeaponSettings,
    pub flamethrower: WeaponSettings,
    #[serde(default = "default_shotgun")]
    pub shotgun: WeaponSettings,
    #[serde(default = "default_laser")]
    pub laser: WeaponSettings,
    #[serde(default = "default_mine")]
    pub mine: WeaponSettings,
    #[serde(default = "default_boomerang")]
    pub boomerang: WeaponSettings,
    #[serde(default = "default_tesla")]
    pub tesla: WeaponSettings,
    #[serde(default = "default_buzzsaw")]
    pub buzzsaw: WeaponSettings,
    #[serde(default = "default_rocket")]
    pub rocket: WeaponSettings,
    #[serde(default = "default_freezegun")]
    pub freezegun: WeaponSettings,

    pub explosion_radius: f32,
}

fn default_shotgun() -> WeaponSettings {
    WeaponSettings {
        cooldown: 0.6, magazine: 8, reload_time: 2.0, range: 200.0,
        damage: 8.0, bullet_speed: 400.0, score_required: 3,
        pellet_count: 7, spread_angle: 0.4,
        ..WeaponSettings::empty()
    }
}
fn default_laser() -> WeaponSettings {
    WeaponSettings {
        cooldown: 0.03, magazine: 60, reload_time: 3.0, range: 600.0,
        damage: 4.0, bullet_speed: 1800.0, score_required: 35,
        ..WeaponSettings::empty()
    }
}
fn default_mine() -> WeaponSettings {
    WeaponSettings {
        cooldown: 0.8, magazine: 5, reload_time: 3.0, range: 0.0,
        damage: 60.0, bullet_speed: 0.0, score_required: 25,
        trigger_radius: 40.0, explosion_radius_override: 90.0,
        ..WeaponSettings::empty()
    }
}
fn default_boomerang() -> WeaponSettings {
    WeaponSettings {
        cooldown: 0.8, magazine: 3, reload_time: 2.0, range: 250.0,
        damage: 20.0, bullet_speed: 350.0, score_required: 28,
        ..WeaponSettings::empty()
    }
}
fn default_tesla() -> WeaponSettings {
    WeaponSettings {
        cooldown: 0.5, magazine: 8, reload_time: 2.0, range: 300.0,
        damage: 15.0, bullet_speed: 500.0, score_required: 22,
        chain_count: 3, chain_range: 80.0,
        ..WeaponSettings::empty()
    }
}
fn default_buzzsaw() -> WeaponSettings {
    WeaponSettings {
        cooldown: 1.2, magazine: 4, reload_time: 2.5, range: 500.0,
        damage: 12.0, bullet_speed: 100.0, score_required: 20,
        ..WeaponSettings::empty()
    }
}
fn default_rocket() -> WeaponSettings {
    WeaponSettings {
        cooldown: 1.5, magazine: 2, reload_time: 3.0, range: 400.0,
        damage: 80.0, bullet_speed: 450.0, score_required: 30,
        explosion_radius_override: 120.0,
        ..WeaponSettings::empty()
    }
}
fn default_freezegun() -> WeaponSettings {
    WeaponSettings {
        cooldown: 0.3, magazine: 15, reload_time: 2.0, range: 250.0,
        damage: 3.0, bullet_speed: 400.0, score_required: 18,
        slow_factor: 0.25, slow_duration: 3.0,
        ..WeaponSettings::empty()
    }
}

impl WeaponSettings {
    fn empty() -> Self {
        Self {
            cooldown: 0.0, magazine: 0, reload_time: 0.0, range: 0.0,
            damage: 0.0, bullet_speed: 0.0, score_required: 0,
            pellet_count: 0, spread_angle: 0.0,
            chain_count: 0, chain_range: 0.0,
            slow_factor: 0.0, slow_duration: 0.0,
            explosion_radius_override: 0.0, trigger_radius: 0.0,
        }
    }
}

impl GameSettings {
    pub fn weapon(&self, w: WeaponType) -> &WeaponSettings {
        match w {
            WeaponType::Pistol => &self.pistol,
            WeaponType::Uzi => &self.uzi,
            WeaponType::Grenade => &self.grenade,
            WeaponType::Railgun => &self.railgun,
            WeaponType::Flamethrower => &self.flamethrower,
            WeaponType::Shotgun => &self.shotgun,
            WeaponType::Laser => &self.laser,
            WeaponType::Mine => &self.mine,
            WeaponType::Boomerang => &self.boomerang,
            WeaponType::Tesla => &self.tesla,
            WeaponType::Buzzsaw => &self.buzzsaw,
            WeaponType::Rocket => &self.rocket,
            WeaponType::FreezeGun => &self.freezegun,
        }
    }

    fn settings_path() -> std::path::PathBuf {
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                std::path::PathBuf::from(home).join(".config")
            });
        config_dir.join("klotzkoepfe")
    }

    fn settings_file() -> std::path::PathBuf {
        Self::settings_path().join("settings.json")
    }

    pub fn save(&self) {
        let dir = Self::settings_path();
        let _ = std::fs::create_dir_all(&dir);
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let path = Self::settings_file();
            if std::fs::write(&path, &json).is_ok() {
                eprintln!("Settings gespeichert: {}", path.display());
            }
        }
    }

    pub fn load() -> Self {
        let path = Self::settings_file();
        match std::fs::read_to_string(&path) {
            Ok(s) => {
                match serde_json::from_str(&s) {
                    Ok(settings) => {
                        eprintln!("Settings geladen: {}", path.display());
                        settings
                    }
                    Err(e) => {
                        eprintln!("Settings fehlerhaft ({}), nutze Defaults", e);
                        Self::default()
                    }
                }
            }
            Err(_) => {
                eprintln!("Keine Settings: {}", path.display());
                Self::default()
            }
        }
    }
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            show_debug: false,
            player_speed: 200.0,
            player_hp: 100.0,
            zombie_speed: 80.0,
            zombie_hp: 30.0,
            zombie_damage: 10.0,
            zombie_damage_cooldown: 0.8,
            combo_drain_speed: 0.08,
            combo_kill_boost: 0.35,
            wave_base_zombies: 5,
            wave_zombie_increment: 3,
            spawn_interval: 0.8,
            wave_pause: 2.0,
            pistol: WeaponSettings {
                cooldown: 0.4, magazine: 12, reload_time: 1.5, range: 350.0,
                damage: 10.0, bullet_speed: 500.0, score_required: 0,
                ..WeaponSettings::empty()
            },
            uzi: WeaponSettings {
                cooldown: 0.08, magazine: 30, reload_time: 2.0, range: 250.0,
                damage: 5.0, bullet_speed: 450.0, score_required: 5,
                ..WeaponSettings::empty()
            },
            grenade: WeaponSettings {
                cooldown: 1.0, magazine: 3, reload_time: 2.5, range: 200.0,
                damage: 50.0, bullet_speed: 300.0, score_required: 10,
                ..WeaponSettings::empty()
            },
            railgun: WeaponSettings {
                cooldown: 0.8, magazine: 5, reload_time: 2.0, range: 800.0,
                damage: 100.0, bullet_speed: 1500.0, score_required: 15,
                ..WeaponSettings::empty()
            },
            flamethrower: WeaponSettings {
                cooldown: 0.04, magazine: 80, reload_time: 3.0, range: 120.0,
                damage: 3.0, bullet_speed: 200.0, score_required: 8,
                spread_angle: 0.3,
                ..WeaponSettings::empty()
            },
            shotgun: default_shotgun(),
            laser: default_laser(),
            mine: default_mine(),
            boomerang: default_boomerang(),
            tesla: default_tesla(),
            buzzsaw: default_buzzsaw(),
            rocket: default_rocket(),
            freezegun: default_freezegun(),
            explosion_radius: 80.0,
        }
    }
}
