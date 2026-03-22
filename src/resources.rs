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
    pub last_wave_zombies: u32,
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
            last_wave_zombies: 0,
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
    pub multiplier_index: usize,
    pub kill_streak: u32,
    pub streak_timer: Timer,
}

impl Default for ComboMeter {
    fn default() -> Self {
        Self {
            position: 0.5,
            multiplier_index: 0,
            kill_streak: 0,
            streak_timer: Timer::from_seconds(3.0, TimerMode::Once),
        }
    }
}

impl ComboMeter {
    pub const MULTIPLIER_TIERS: &'static [u32] = &[1, 2, 5, 10, 25, 50, 100, 250, 500, 1000, 1500];

    pub fn current_multiplier(&self) -> u32 {
        Self::MULTIPLIER_TIERS[self.multiplier_index.min(Self::MULTIPLIER_TIERS.len() - 1)]
    }
}

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    #[default]
    Playing,
    Paused,
    GameOver,
    Restarting,
    UnlockScreen,
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
    pub pierce_count: u32,
    #[serde(default = "default_max_magazines")]
    pub max_magazines: u32,
    #[serde(default)]
    pub explosion_radius_override: f32,
    #[serde(default)]
    pub trigger_radius: f32,
    #[serde(default)]
    pub score_level_2: i32,
    #[serde(default)]
    pub score_level_3: i32,
}

#[derive(Resource, Clone, serde::Serialize, serde::Deserialize)]
pub struct GameSettings {
    #[serde(skip)]
    pub show_debug: bool,

    #[serde(default = "default_player_count")]
    pub player_count: u32,
    pub player_speed: f32,
    pub player_hp: f32,
    #[serde(default)]
    pub player_regen_rate: f32,
    #[serde(default = "default_regen_delay")]
    pub player_regen_delay: f32,

    pub zombie_speed: f32,
    pub zombie_hp: f32,
    pub zombie_damage: f32,
    pub zombie_damage_cooldown: f32,

    #[serde(default = "default_big_zombie_hp")]
    pub big_zombie_hp: f32,
    #[serde(default = "default_big_zombie_speed")]
    pub big_zombie_speed: f32,
    #[serde(default = "default_big_zombie_damage")]
    pub big_zombie_damage: f32,
    #[serde(default = "default_big_zombie_scale")]
    pub big_zombie_scale: f32,
    #[serde(default = "default_big_zombie_spawn_chance")]
    pub big_zombie_spawn_chance: f32,
    #[serde(default = "default_big_zombie_start_wave")]
    pub big_zombie_start_wave: u32,

    pub combo_drain_speed: f32,
    pub combo_kill_boost: f32,

    #[serde(default = "default_multiplier_decay")]
    pub multiplier_decay_rate: f32,
    #[serde(default = "default_multiplier_window")]
    pub multiplier_kill_window: f32,

    pub wave_base_zombies: u32,
    pub wave_zombie_increment: u32,
    pub spawn_interval: f32,
    pub wave_pause: f32,

    #[serde(default = "default_spawn_rate_decrease")]
    pub spawn_rate_decrease_per_wave: f32,
    #[serde(default = "default_min_spawn_interval")]
    pub min_spawn_interval: f32,
    #[serde(default = "default_percent_mode_wave")]
    pub percent_mode_after_wave: u32,
    #[serde(default = "default_zombie_increase_percent")]
    pub zombie_increase_percent: f32,

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

    #[serde(default = "default_crate_spawn_chance")]
    pub crate_spawn_chance: f32,
    #[serde(default = "default_crate_despawn_time")]
    pub crate_despawn_time: f32,
    #[serde(default = "default_base_crate_respawn")]
    pub base_crate_respawn_time: f32,

    #[serde(default)]
    pub gamemaster_level: u32,
    #[serde(default)]
    pub friendly_fire: bool,
    #[serde(default = "default_true")]
    pub explosion_friendly_fire: bool,

    #[serde(default = "default_kb_zombie")]
    pub knockback_strength_zombie: f32,
    #[serde(default = "default_kb_player")]
    pub knockback_strength_player: f32,
    #[serde(default = "default_kb_duration")]
    pub knockback_duration: f32,

    #[serde(default)]
    pub fullscreen: bool,

    #[serde(default)]
    pub pixelation_enabled: bool,
    #[serde(default = "default_pixel_size")]
    pub pixel_size: f32,

    // Gore-Settings
    #[serde(default = "default_blood_particles")]
    pub blood_particles: u32,
    #[serde(default = "default_blood_spread")]
    pub blood_spread_speed: f32,
    #[serde(default = "default_dismember_chance")]
    pub dismember_chance: f32,
    #[serde(default = "default_gib_decay")]
    pub gib_decay_time: f32,
}

fn default_pixel_size() -> f32 { 1.3 }
fn default_player_count() -> u32 { 1 }
fn default_regen_delay() -> f32 { 5.0 }
fn default_true() -> bool { true }
fn default_crate_spawn_chance() -> f32 { 0.03 }
fn default_crate_despawn_time() -> f32 { 15.0 }
fn default_base_crate_respawn() -> f32 { 30.0 }
fn default_max_magazines() -> u32 { 999 }
fn default_big_zombie_hp() -> f32 { 100.0 }
fn default_big_zombie_speed() -> f32 { 50.0 }
fn default_big_zombie_damage() -> f32 { 25.0 }
fn default_big_zombie_scale() -> f32 { 1.8 }
fn default_big_zombie_spawn_chance() -> f32 { 0.15 }
fn default_big_zombie_start_wave() -> u32 { 5 }
fn default_spawn_rate_decrease() -> f32 { 0.02 }
fn default_min_spawn_interval() -> f32 { 0.2 }
fn default_percent_mode_wave() -> u32 { 20 }
fn default_zombie_increase_percent() -> f32 { 15.0 }
fn default_multiplier_decay() -> f32 { 0.5 }
fn default_multiplier_window() -> f32 { 3.0 }
fn default_kb_zombie() -> f32 { 150.0 }
fn default_kb_player() -> f32 { 200.0 }
fn default_kb_duration() -> f32 { 0.15 }
fn default_blood_particles() -> u32 { 4 }
fn default_blood_spread() -> f32 { 100.0 }
fn default_dismember_chance() -> f32 { 0.30 }
fn default_gib_decay() -> f32 { 3.0 }

fn default_shotgun() -> WeaponSettings {
    WeaponSettings {
        cooldown: 0.6, magazine: 8, reload_time: 2.0, range: 200.0,
        damage: 8.0, bullet_speed: 400.0, score_required: 100,
        pellet_count: 7, spread_angle: 0.4, max_magazines: 6,
        score_level_2: 1000, score_level_3: 3500,
        ..WeaponSettings::empty()
    }
}
fn default_laser() -> WeaponSettings {
    WeaponSettings {
        cooldown: 0.03, magazine: 60, reload_time: 3.0, range: 600.0,
        damage: 4.0, bullet_speed: 1800.0, score_required: 15000,
        pierce_count: 999, max_magazines: 3,
        score_level_2: 30000, score_level_3: 50000,
        ..WeaponSettings::empty()
    }
}
fn default_mine() -> WeaponSettings {
    WeaponSettings {
        cooldown: 0.05, magazine: 5, reload_time: 3.0, range: 0.0,
        damage: 60.0, bullet_speed: 0.0, score_required: 7000,
        trigger_radius: 40.0, explosion_radius_override: 90.0, max_magazines: 3,
        score_level_2: 15000, score_level_3: 30000,
        ..WeaponSettings::empty()
    }
}
fn default_boomerang() -> WeaponSettings {
    WeaponSettings {
        cooldown: 0.8, magazine: 1, reload_time: 2.0, range: 250.0,
        damage: 20.0, bullet_speed: 350.0, score_required: 8000,
        max_magazines: 5,
        score_level_2: 18000, score_level_3: 35000,
        ..WeaponSettings::empty()
    }
}
fn default_tesla() -> WeaponSettings {
    WeaponSettings {
        cooldown: 0.5, magazine: 8, reload_time: 2.0, range: 300.0,
        damage: 15.0, bullet_speed: 500.0, score_required: 6000,
        chain_count: 3, chain_range: 80.0, max_magazines: 5,
        score_level_2: 14000, score_level_3: 28000,
        ..WeaponSettings::empty()
    }
}
fn default_buzzsaw() -> WeaponSettings {
    WeaponSettings {
        cooldown: 1.2, magazine: 4, reload_time: 2.5, range: 500.0,
        damage: 12.0, bullet_speed: 100.0, score_required: 3000,
        pierce_count: 999, max_magazines: 4,
        score_level_2: 8000, score_level_3: 20000,
        ..WeaponSettings::empty()
    }
}
fn default_rocket() -> WeaponSettings {
    WeaponSettings {
        cooldown: 1.5, magazine: 2, reload_time: 3.0, range: 400.0,
        damage: 80.0, bullet_speed: 450.0, score_required: 10000,
        explosion_radius_override: 120.0, max_magazines: 3,
        score_level_2: 22000, score_level_3: 40000,
        ..WeaponSettings::empty()
    }
}
fn default_freezegun() -> WeaponSettings {
    WeaponSettings {
        cooldown: 0.3, magazine: 15, reload_time: 2.0, range: 250.0,
        damage: 3.0, bullet_speed: 400.0, score_required: 1500,
        slow_factor: 0.25, slow_duration: 3.0, max_magazines: 5,
        score_level_2: 5000, score_level_3: 12000,
        ..WeaponSettings::empty()
    }
}

impl WeaponSettings {
    fn empty() -> Self {
        Self {
            cooldown: 0.0, magazine: 0, reload_time: 0.0, range: 0.0,
            damage: 0.0, bullet_speed: 0.0, score_required: 0,
            pellet_count: 0, spread_angle: 0.0, pierce_count: 0, max_magazines: 0,
            chain_count: 0, chain_range: 0.0,
            slow_factor: 0.0, slow_duration: 0.0,
            explosion_radius_override: 0.0, trigger_radius: 0.0,
            score_level_2: 0, score_level_3: 0,
        }
    }
}

pub const MAX_WEAPON_LEVEL: u32 = 3;

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

    /// Berechnet Waffen-Level basierend auf Score
    pub fn weapon_level(&self, w: WeaponType, score: i32) -> u32 {
        if self.gamemaster_level > 0 { return self.gamemaster_level.min(MAX_WEAPON_LEVEL); }
        let ws = self.weapon(w);
        if ws.score_level_3 > 0 && score >= ws.score_level_3 { 3 }
        else if ws.score_level_2 > 0 && score >= ws.score_level_2 { 2 }
        else { 1 }
    }

    /// Gibt Level-skalierte Waffen-Stats zurueck
    pub fn weapon_at_level(&self, w: WeaponType, level: u32) -> WeaponSettings {
        let base = self.weapon(w).clone();
        if level <= 1 { return base; }

        let lvl = level.min(MAX_WEAPON_LEVEL);

        // Spezial-Verhalten pro Waffe
        match w {
            WeaponType::Boomerang => {
                // Level = Anzahl Wuerfe bevor Reload
                WeaponSettings {
                    magazine: lvl,
                    cooldown: base.cooldown * (1.0 - 0.2 * (lvl - 1) as f32),
                    damage: base.damage * (1.0 + 0.25 * (lvl - 1) as f32),
                    ..base
                }
            }
            WeaponType::Shotgun => {
                // Mehr Pellets, engerer Spread
                WeaponSettings {
                    pellet_count: base.pellet_count + (lvl - 1) * 2,
                    spread_angle: base.spread_angle * (1.0 - 0.15 * (lvl - 1) as f32),
                    damage: base.damage * (1.0 + 0.15 * (lvl - 1) as f32),
                    ..base
                }
            }
            WeaponType::Uzi => {
                // Schneller, mehr Reichweite, groesseres Magazin
                WeaponSettings {
                    cooldown: base.cooldown * (1.0 - 0.2 * (lvl - 1) as f32),
                    range: base.range * (1.0 + 0.25 * (lvl - 1) as f32),
                    magazine: base.magazine + (lvl - 1) * 10,
                    ..base
                }
            }
            WeaponType::Tesla => {
                // Mehr Chains, mehr Chain-Range
                WeaponSettings {
                    chain_count: base.chain_count + (lvl - 1) * 2,
                    chain_range: base.chain_range * (1.0 + 0.3 * (lvl - 1) as f32),
                    damage: base.damage * (1.0 + 0.2 * (lvl - 1) as f32),
                    ..base
                }
            }
            WeaponType::FreezeGun => {
                // Laenger einfrieren, mehr Slow
                WeaponSettings {
                    slow_duration: base.slow_duration * (1.0 + 0.4 * (lvl - 1) as f32),
                    slow_factor: (base.slow_factor * (1.0 - 0.2 * (lvl - 1) as f32)).max(0.05),
                    magazine: base.magazine + (lvl - 1) * 5,
                    ..base
                }
            }
            WeaponType::Mine => {
                // Groesserer Explosionsradius, mehr Damage
                WeaponSettings {
                    explosion_radius_override: base.explosion_radius_override * (1.0 + 0.3 * (lvl - 1) as f32),
                    damage: base.damage * (1.0 + 0.3 * (lvl - 1) as f32),
                    magazine: base.magazine + (lvl - 1) * 2,
                    ..base
                }
            }
            WeaponType::Grenade | WeaponType::Rocket => {
                // Mehr Damage, groesserer Radius
                WeaponSettings {
                    damage: base.damage * (1.0 + 0.3 * (lvl - 1) as f32),
                    explosion_radius_override: base.explosion_radius_override * (1.0 + 0.25 * (lvl - 1) as f32),
                    magazine: base.magazine + (lvl - 1),
                    ..base
                }
            }
            // Generisch: alle anderen Waffen
            _ => {
                WeaponSettings {
                    damage: base.damage * (1.0 + 0.2 * (lvl - 1) as f32),
                    cooldown: base.cooldown * (1.0 - 0.15 * (lvl - 1) as f32),
                    magazine: base.magazine + (lvl - 1) * 3,
                    range: base.range * (1.0 + 0.15 * (lvl - 1) as f32),
                    ..base
                }
            }
        }
    }

    /// Migrate old settings values to new defaults when they look outdated
    fn migrate_old_values(&mut self) {
        let defaults = Self::default();
        // If score values are from the old system (pre-multiplier, < 100),
        // reset all weapon scores to new defaults
        if self.pistol.score_level_2 < 100 || self.shotgun.score_required < 50 {
            eprintln!("Alte Score-Werte erkannt, migriere auf neue Defaults...");
            for weapon in WeaponType::all() {
                let def = defaults.weapon(*weapon);
                let ws = self.weapon_mut(*weapon);
                ws.score_required = def.score_required;
                ws.score_level_2 = def.score_level_2;
                ws.score_level_3 = def.score_level_3;
            }
        }
        // Fix max_magazines if 0 (old format)
        for weapon in WeaponType::all() {
            let def_mags = defaults.weapon(*weapon).max_magazines;
            let ws = self.weapon_mut(*weapon);
            if ws.max_magazines == 0 && def_mags > 0 {
                ws.max_magazines = def_mags;
            }
        }
    }

    pub fn weapon_mut(&mut self, w: WeaponType) -> &mut WeaponSettings {
        match w {
            WeaponType::Pistol => &mut self.pistol,
            WeaponType::Uzi => &mut self.uzi,
            WeaponType::Grenade => &mut self.grenade,
            WeaponType::Railgun => &mut self.railgun,
            WeaponType::Flamethrower => &mut self.flamethrower,
            WeaponType::Shotgun => &mut self.shotgun,
            WeaponType::Laser => &mut self.laser,
            WeaponType::Mine => &mut self.mine,
            WeaponType::Boomerang => &mut self.boomerang,
            WeaponType::Tesla => &mut self.tesla,
            WeaponType::Buzzsaw => &mut self.buzzsaw,
            WeaponType::Rocket => &mut self.rocket,
            WeaponType::FreezeGun => &mut self.freezegun,
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
                        let mut settings: GameSettings = settings;
                        settings.migrate_old_values();
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
            player_count: 1,
            player_speed: 200.0,
            player_hp: 100.0,
            player_regen_rate: 0.0,
            player_regen_delay: 5.0,
            zombie_speed: 80.0,
            zombie_hp: 30.0,
            zombie_damage: 10.0,
            zombie_damage_cooldown: 0.8,
            big_zombie_hp: 100.0,
            big_zombie_speed: 50.0,
            big_zombie_damage: 25.0,
            big_zombie_scale: 1.8,
            big_zombie_spawn_chance: 0.15,
            big_zombie_start_wave: 5,
            combo_drain_speed: 0.08,
            combo_kill_boost: 0.35,
            multiplier_decay_rate: 0.5,
            multiplier_kill_window: 3.0,
            wave_base_zombies: 5,
            wave_zombie_increment: 3,
            spawn_interval: 0.8,
            wave_pause: 2.0,
            spawn_rate_decrease_per_wave: 0.02,
            min_spawn_interval: 0.2,
            percent_mode_after_wave: 20,
            zombie_increase_percent: 15.0,
            pistol: WeaponSettings {
                cooldown: 0.4, magazine: 12, reload_time: 1.5, range: 350.0,
                damage: 10.0, bullet_speed: 500.0, score_required: 0,
                max_magazines: 10,
                score_level_2: 500, score_level_3: 2000,
                ..WeaponSettings::empty()
            },
            uzi: WeaponSettings {
                cooldown: 0.08, magazine: 30, reload_time: 2.0, range: 250.0,
                damage: 5.0, bullet_speed: 450.0, score_required: 200,
                max_magazines: 6,
                score_level_2: 1500, score_level_3: 4000,
                ..WeaponSettings::empty()
            },
            grenade: WeaponSettings {
                cooldown: 1.0, magazine: 3, reload_time: 2.5, range: 200.0,
                damage: 50.0, bullet_speed: 300.0, score_required: 800,
                max_magazines: 4,
                score_level_2: 3000, score_level_3: 8000,
                ..WeaponSettings::empty()
            },
            railgun: WeaponSettings {
                cooldown: 0.8, magazine: 5, reload_time: 2.0, range: 800.0,
                damage: 100.0, bullet_speed: 1500.0, score_required: 5000,
                pierce_count: 999, max_magazines: 4,
                score_level_2: 12000, score_level_3: 25000,
                ..WeaponSettings::empty()
            },
            flamethrower: WeaponSettings {
                cooldown: 0.04, magazine: 80, reload_time: 3.0, range: 120.0,
                damage: 3.0, bullet_speed: 200.0, score_required: 400,
                spread_angle: 0.3, max_magazines: 3,
                score_level_2: 2000, score_level_3: 6000,
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
            crate_spawn_chance: 0.03,
            crate_despawn_time: 15.0,
            base_crate_respawn_time: 30.0,
            gamemaster_level: 0,
            friendly_fire: false,
            explosion_friendly_fire: true,
            knockback_strength_zombie: 150.0,
            knockback_strength_player: 200.0,
            knockback_duration: 0.15,
            fullscreen: false,
            pixelation_enabled: false,
            pixel_size: 1.3,
            blood_particles: 4,
            blood_spread_speed: 100.0,
            dismember_chance: 0.30,
            gib_decay_time: 3.0,
        }
    }
}
