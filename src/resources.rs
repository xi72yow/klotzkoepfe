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
    Lobby,
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

/// Berechnet Level 2/3 Stats aus Basis-Stats mit den alten Scaling-Formeln.
/// Wird nur fuer Defaults und Migration alter Settings verwendet.
fn legacy_scale(base: &WeaponSettings, w: WeaponType, level: u32) -> WeaponSettings {
    let lvl = level.clamp(1, 3);
    if lvl <= 1 { return base.clone(); }
    match w {
        WeaponType::Boomerang => WeaponSettings {
            magazine: lvl,
            cooldown: base.cooldown * (1.0 - 0.2 * (lvl - 1) as f32),
            damage: base.damage * (1.0 + 0.25 * (lvl - 1) as f32),
            ..base.clone()
        },
        WeaponType::Shotgun => WeaponSettings {
            pellet_count: base.pellet_count + (lvl - 1) * 2,
            spread_angle: base.spread_angle * (1.0 - 0.15 * (lvl - 1) as f32),
            damage: base.damage * (1.0 + 0.15 * (lvl - 1) as f32),
            ..base.clone()
        },
        WeaponType::Uzi => WeaponSettings {
            cooldown: base.cooldown * (1.0 - 0.2 * (lvl - 1) as f32),
            range: base.range * (1.0 + 0.25 * (lvl - 1) as f32),
            magazine: base.magazine + (lvl - 1) * 10,
            ..base.clone()
        },
        WeaponType::Tesla => WeaponSettings {
            chain_count: base.chain_count + (lvl - 1) * 2,
            chain_range: base.chain_range * (1.0 + 0.3 * (lvl - 1) as f32),
            damage: base.damage * (1.0 + 0.2 * (lvl - 1) as f32),
            ..base.clone()
        },
        WeaponType::FreezeGun => WeaponSettings {
            slow_duration: base.slow_duration * (1.0 + 0.4 * (lvl - 1) as f32),
            slow_factor: (base.slow_factor * (1.0 - 0.2 * (lvl - 1) as f32)).max(0.05),
            magazine: base.magazine + (lvl - 1) * 5,
            ..base.clone()
        },
        WeaponType::Mine => WeaponSettings {
            explosion_radius_override: base.explosion_radius_override * (1.0 + 0.3 * (lvl - 1) as f32),
            damage: base.damage * (1.0 + 0.3 * (lvl - 1) as f32),
            magazine: base.magazine + (lvl - 1) * 2,
            ..base.clone()
        },
        WeaponType::Grenade | WeaponType::Rocket => WeaponSettings {
            damage: base.damage * (1.0 + 0.3 * (lvl - 1) as f32),
            explosion_radius_override: base.explosion_radius_override * (1.0 + 0.25 * (lvl - 1) as f32),
            magazine: base.magazine + (lvl - 1),
            ..base.clone()
        },
        _ => WeaponSettings {
            damage: base.damage * (1.0 + 0.2 * (lvl - 1) as f32),
            cooldown: base.cooldown * (1.0 - 0.15 * (lvl - 1) as f32),
            magazine: base.magazine + (lvl - 1) * 3,
            range: base.range * (1.0 + 0.15 * (lvl - 1) as f32),
            ..base.clone()
        },
    }
}

/// Erzeugt 3-Level Array aus Basis-Stats
fn make_levels(w: WeaponType, base: WeaponSettings) -> [WeaponSettings; 3] {
    let lv2 = legacy_scale(&base, w, 2);
    let lv3 = legacy_scale(&base, w, 3);
    [base, lv2, lv3]
}

/// Custom Deserializer: akzeptiert altes Format (einzelnes Objekt) und neues (Array von 3)
fn deserialize_weapon_levels<'de, D>(deserializer: D) -> Result<[WeaponSettings; 3], D::Error>
where D: serde::Deserializer<'de>
{
    use serde::de::Error;
    use serde::Deserialize;
    let value = serde_json::Value::deserialize(deserializer)?;
    match &value {
        serde_json::Value::Array(arr) if arr.len() == 3 => {
            let l1: WeaponSettings = serde_json::from_value(arr[0].clone()).map_err(D::Error::custom)?;
            let l2: WeaponSettings = serde_json::from_value(arr[1].clone()).map_err(D::Error::custom)?;
            let l3: WeaponSettings = serde_json::from_value(arr[2].clone()).map_err(D::Error::custom)?;
            Ok([l1, l2, l3])
        }
        serde_json::Value::Object(_) => {
            let base: WeaponSettings = serde_json::from_value(value).map_err(D::Error::custom)?;
            // Altes Format: 3x kopieren, migrate_old_values macht Scaling
            Ok([base.clone(), base.clone(), base])
        }
        _ => Err(D::Error::custom("expected array of 3 or object for weapon settings"))
    }
}

#[derive(Resource, Clone, serde::Serialize, serde::Deserialize)]
pub struct GameSettings {
    #[serde(skip)]
    pub show_debug: bool,
    #[serde(skip)]
    pub show_cone_debug: bool,
    #[serde(skip)]
    pub show_grid: bool,
    #[serde(default = "default_grid_size")]
    pub grid_size: f32,
    #[serde(skip)]
    pub show_weapon_range: bool,
    #[serde(skip)]
    pub show_hitboxes: bool,

    #[serde(default = "default_player_count")]
    pub player_count: u32,
    pub player_speed: f32,
    pub player_hp: f32,
    #[serde(default)]
    pub player_regen_rate: f32,
    #[serde(default = "default_regen_delay")]
    pub player_regen_delay: f32,
    #[serde(default = "default_crouch_speed")]
    pub crouch_speed_factor: f32,

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

    #[serde(deserialize_with = "deserialize_weapon_levels")]
    pub pistol: [WeaponSettings; 3],
    #[serde(deserialize_with = "deserialize_weapon_levels")]
    pub uzi: [WeaponSettings; 3],
    #[serde(deserialize_with = "deserialize_weapon_levels")]
    pub grenade: [WeaponSettings; 3],
    #[serde(deserialize_with = "deserialize_weapon_levels")]
    pub railgun: [WeaponSettings; 3],
    #[serde(deserialize_with = "deserialize_weapon_levels")]
    pub flamethrower: [WeaponSettings; 3],
    #[serde(default = "default_shotgun_levels", deserialize_with = "deserialize_weapon_levels")]
    pub shotgun: [WeaponSettings; 3],
    #[serde(default = "default_laser_levels", deserialize_with = "deserialize_weapon_levels")]
    pub laser: [WeaponSettings; 3],
    #[serde(default = "default_mine_levels", deserialize_with = "deserialize_weapon_levels")]
    pub mine: [WeaponSettings; 3],
    #[serde(default = "default_boomerang_levels", deserialize_with = "deserialize_weapon_levels")]
    pub boomerang: [WeaponSettings; 3],
    #[serde(default = "default_tesla_levels", deserialize_with = "deserialize_weapon_levels")]
    pub tesla: [WeaponSettings; 3],
    #[serde(default = "default_buzzsaw_levels", deserialize_with = "deserialize_weapon_levels")]
    pub buzzsaw: [WeaponSettings; 3],
    #[serde(default = "default_rocket_levels", deserialize_with = "deserialize_weapon_levels")]
    pub rocket: [WeaponSettings; 3],
    #[serde(default = "default_freezegun_levels", deserialize_with = "deserialize_weapon_levels")]
    pub freezegun: [WeaponSettings; 3],

    pub explosion_radius: f32,

    #[serde(default = "default_crate_spawn_chance")]
    pub crate_spawn_chance: f32,
    #[serde(default = "default_crate_despawn_time")]
    pub crate_despawn_time: f32,
    #[serde(default = "default_base_crate_respawn")]
    pub base_crate_respawn_time: f32,
    #[serde(default = "default_flare_duration")]
    pub flare_duration: f32,

    #[serde(default)]
    pub gamemaster_level: u32,
    #[serde(default)]
    pub gm_start_wave: u32,
    #[serde(default)]
    pub gm_start_weapon: u32,
    #[serde(default = "default_game_speed")]
    pub gm_game_speed: f32,
    #[serde(skip)]
    pub gm_weapon_dirty: bool,
    #[serde(skip)]
    pub gm_wave_dirty: bool,
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

    #[serde(default = "default_volume")]
    pub volume: f32,
    #[serde(default = "default_category_volume")]
    pub vol_weapons: f32,
    #[serde(default = "default_category_volume")]
    pub vol_enemies: f32,
    #[serde(default = "default_category_volume")]
    pub vol_player: f32,

    #[serde(default)]
    pub fullscreen: bool,

    #[serde(default)]
    pub pixelation_enabled: bool,
    #[serde(default = "default_pixel_size")]
    pub pixel_size: f32,

    #[serde(default)]
    pub retro_crt_enabled: bool,
    #[serde(default = "default_scanline_intensity")]
    pub scanline_intensity: f32,
    #[serde(default = "default_chromatic_aberration")]
    pub chromatic_aberration: f32,
    #[serde(default = "default_vignette_intensity")]
    pub vignette_intensity: f32,

    // Gore-Settings
    #[serde(default = "default_blood_particles")]
    pub blood_particles: u32,
    #[serde(default = "default_blood_spread")]
    pub blood_spread_speed: f32,
    #[serde(default = "default_dismember_chance")]
    pub dismember_chance: f32,
    #[serde(default = "default_gib_decay")]
    pub gib_decay_time: f32,

    /// Settings-Version fuer Migration (0 = altes Format mit einem WeaponSettings pro Waffe)
    #[serde(default)]
    pub settings_version: u32,
}

fn default_volume() -> f32 { 0.5 }
fn default_category_volume() -> f32 { 1.0 }
fn default_pixel_size() -> f32 { 1.3 }
fn default_scanline_intensity() -> f32 { 0.3 }
fn default_chromatic_aberration() -> f32 { 1.0 }
fn default_vignette_intensity() -> f32 { 0.3 }
fn default_player_count() -> u32 { 1 }
fn default_regen_delay() -> f32 { 5.0 }
fn default_true() -> bool { true }
fn default_game_speed() -> f32 { 1.0 }
fn default_crate_spawn_chance() -> f32 { 0.03 }
fn default_crate_despawn_time() -> f32 { 15.0 }
fn default_base_crate_respawn() -> f32 { 30.0 }
fn default_flare_duration() -> f32 { 5.0 }
fn default_max_magazines() -> u32 { 999 }
fn default_big_zombie_hp() -> f32 { 100.0 }
fn default_big_zombie_speed() -> f32 { 10.0 }
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
fn default_grid_size() -> f32 { 50.0 }
fn default_crouch_speed() -> f32 { 0.4 }

// Basis-WeaponSettings (Level 1) fuer jede Waffe
fn base_shotgun() -> WeaponSettings {
    WeaponSettings {
        cooldown: 0.6, magazine: 8, reload_time: 2.0, range: 200.0,
        damage: 8.0, bullet_speed: 400.0, score_required: 100,
        pellet_count: 7, spread_angle: 0.4, max_magazines: 6,
        score_level_2: 1000, score_level_3: 3500,
        ..WeaponSettings::empty()
    }
}
fn base_laser() -> WeaponSettings {
    WeaponSettings {
        cooldown: 0.03, magazine: 60, reload_time: 3.0, range: 600.0,
        damage: 4.0, bullet_speed: 1800.0, score_required: 15000,
        spread_angle: 0.02, pierce_count: 999, max_magazines: 3,
        score_level_2: 30000, score_level_3: 50000,
        ..WeaponSettings::empty()
    }
}
fn base_mine() -> WeaponSettings {
    WeaponSettings {
        cooldown: 0.05, magazine: 5, reload_time: 3.0, range: 0.0,
        damage: 60.0, bullet_speed: 0.0, score_required: 7000,
        trigger_radius: 40.0, explosion_radius_override: 90.0, max_magazines: 3,
        score_level_2: 15000, score_level_3: 30000,
        ..WeaponSettings::empty()
    }
}
fn base_boomerang() -> WeaponSettings {
    WeaponSettings {
        cooldown: 0.8, magazine: 1, reload_time: 2.0, range: 250.0,
        damage: 20.0, bullet_speed: 350.0, score_required: 8000,
        max_magazines: 5,
        score_level_2: 18000, score_level_3: 35000,
        ..WeaponSettings::empty()
    }
}
fn base_tesla() -> WeaponSettings {
    WeaponSettings {
        cooldown: 0.5, magazine: 8, reload_time: 2.0, range: 300.0,
        damage: 15.0, bullet_speed: 500.0, score_required: 6000,
        spread_angle: 0.04, chain_count: 3, chain_range: 80.0, max_magazines: 5,
        score_level_2: 14000, score_level_3: 28000,
        ..WeaponSettings::empty()
    }
}
fn base_buzzsaw() -> WeaponSettings {
    WeaponSettings {
        cooldown: 1.2, magazine: 4, reload_time: 2.5, range: 500.0,
        damage: 12.0, bullet_speed: 100.0, score_required: 3000,
        spread_angle: 0.05, pierce_count: 999, max_magazines: 4,
        score_level_2: 8000, score_level_3: 20000,
        ..WeaponSettings::empty()
    }
}
fn base_rocket() -> WeaponSettings {
    WeaponSettings {
        cooldown: 1.5, magazine: 2, reload_time: 3.0, range: 400.0,
        damage: 80.0, bullet_speed: 450.0, score_required: 10000,
        spread_angle: 0.03, explosion_radius_override: 120.0, max_magazines: 3,
        score_level_2: 22000, score_level_3: 40000,
        ..WeaponSettings::empty()
    }
}
fn base_freezegun() -> WeaponSettings {
    WeaponSettings {
        cooldown: 0.3, magazine: 15, reload_time: 2.0, range: 250.0,
        damage: 3.0, bullet_speed: 400.0, score_required: 1500,
        spread_angle: 0.05, slow_factor: 0.25, slow_duration: 3.0, max_magazines: 5,
        score_level_2: 5000, score_level_3: 12000,
        ..WeaponSettings::empty()
    }
}

// Default-Funktionen fuer serde (geben 3-Level Arrays zurueck)
fn default_shotgun_levels() -> [WeaponSettings; 3] { make_levels(WeaponType::Shotgun, base_shotgun()) }
fn default_laser_levels() -> [WeaponSettings; 3] { make_levels(WeaponType::Laser, base_laser()) }
fn default_mine_levels() -> [WeaponSettings; 3] { make_levels(WeaponType::Mine, base_mine()) }
fn default_boomerang_levels() -> [WeaponSettings; 3] { make_levels(WeaponType::Boomerang, base_boomerang()) }
fn default_tesla_levels() -> [WeaponSettings; 3] { make_levels(WeaponType::Tesla, base_tesla()) }
fn default_buzzsaw_levels() -> [WeaponSettings; 3] { make_levels(WeaponType::Buzzsaw, base_buzzsaw()) }
fn default_rocket_levels() -> [WeaponSettings; 3] { make_levels(WeaponType::Rocket, base_rocket()) }
fn default_freezegun_levels() -> [WeaponSettings; 3] { make_levels(WeaponType::FreezeGun, base_freezegun()) }

pub const MAX_WEAPON_LEVEL: u32 = 3;

impl GameSettings {
    /// Gibt Level-1 Stats zurueck (fuer Score-Thresholds, max_magazines etc.)
    pub fn weapon(&self, w: WeaponType) -> &WeaponSettings {
        &self.weapon_levels(w)[0]
    }

    /// Gibt alle 3 Level-Stats zurueck
    pub fn weapon_levels(&self, w: WeaponType) -> &[WeaponSettings; 3] {
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

    /// Gibt Stats fuer ein bestimmtes Level zurueck
    pub fn weapon_at_level(&self, w: WeaponType, level: u32) -> WeaponSettings {
        let levels = self.weapon_levels(w);
        levels[(level.clamp(1, MAX_WEAPON_LEVEL) - 1) as usize].clone()
    }

    /// Migrate old settings values to new defaults when they look outdated
    fn migrate_old_values(&mut self) {
        let defaults = Self::default();

        // Migration von altem Format (settings_version == 0): Level 2/3 aus Formeln berechnen
        if self.settings_version == 0 {
            eprintln!("Altes Settings-Format erkannt, migriere auf Per-Level Stats...");
            for weapon in WeaponType::all() {
                let base = self.weapon(*weapon).clone();
                let lv2 = legacy_scale(&base, *weapon, 2);
                let lv3 = legacy_scale(&base, *weapon, 3);
                let levels = self.weapon_levels_mut(*weapon);
                levels[1] = lv2;
                levels[2] = lv3;
            }
            self.settings_version = 1;
        }

        // If score values are from the old system (pre-multiplier, < 100),
        // reset all weapon scores to new defaults
        if self.pistol[0].score_level_2 < 100 || self.shotgun[0].score_required < 50 {
            eprintln!("Alte Score-Werte erkannt, migriere auf neue Defaults...");
            for weapon in WeaponType::all() {
                let def = defaults.weapon(*weapon);
                let ws = &mut self.weapon_levels_mut(*weapon)[0];
                ws.score_required = def.score_required;
                ws.score_level_2 = def.score_level_2;
                ws.score_level_3 = def.score_level_3;
            }
        }
        // Fix max_magazines if 0 (old format)
        for weapon in WeaponType::all() {
            let def_mags = defaults.weapon(*weapon).max_magazines;
            let ws = &mut self.weapon_levels_mut(*weapon)[0];
            if ws.max_magazines == 0 && def_mags > 0 {
                ws.max_magazines = def_mags;
            }
        }
    }

    pub fn weapon_mut(&mut self, w: WeaponType) -> &mut WeaponSettings {
        &mut self.weapon_levels_mut(w)[0]
    }

    pub fn weapon_levels_mut(&mut self, w: WeaponType) -> &mut [WeaponSettings; 3] {
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
        let pistol_levels: [WeaponSettings; 3] = [
            WeaponSettings {
                cooldown: 0.4, magazine: 12, reload_time: 1.5, range: 400.0,
                damage: 10.0, bullet_speed: 1000.0, score_required: 0,
                spread_angle: 0.05, max_magazines: 10,
                score_level_2: 500, score_level_3: 2000,
                ..WeaponSettings::empty()
            },
            WeaponSettings {
                cooldown: 0.34, magazine: 15, reload_time: 1.5, range: 600.0,
                damage: 12.0, bullet_speed: 2000.0, score_required: 0,
                spread_angle: 0.05, max_magazines: 10,
                score_level_2: 500, score_level_3: 2000,
                ..WeaponSettings::empty()
            },
            WeaponSettings {
                cooldown: 0.28, magazine: 18, reload_time: 1.5, range: 1000.0,
                damage: 14.0, bullet_speed: 4000.0, score_required: 0,
                spread_angle: 0.05, max_magazines: 10,
                score_level_2: 500, score_level_3: 2000,
                ..WeaponSettings::empty()
            },
        ];
        let uzi_base = WeaponSettings {
            cooldown: 0.08, magazine: 30, reload_time: 2.0, range: 250.0,
            damage: 5.0, bullet_speed: 450.0, score_required: 200,
            spread_angle: 0.06, max_magazines: 6,
            score_level_2: 1500, score_level_3: 4000,
            ..WeaponSettings::empty()
        };
        let grenade_base = WeaponSettings {
            cooldown: 1.0, magazine: 3, reload_time: 2.5, range: 200.0,
            damage: 50.0, bullet_speed: 300.0, score_required: 800,
            max_magazines: 4,
            score_level_2: 3000, score_level_3: 8000,
            ..WeaponSettings::empty()
        };
        let railgun_base = WeaponSettings {
            cooldown: 0.8, magazine: 5, reload_time: 2.0, range: 800.0,
            damage: 100.0, bullet_speed: 1500.0, score_required: 5000,
            pierce_count: 999, max_magazines: 4,
            score_level_2: 12000, score_level_3: 25000,
            ..WeaponSettings::empty()
        };
        let flamethrower_base = WeaponSettings {
            cooldown: 0.04, magazine: 80, reload_time: 3.0, range: 120.0,
            damage: 3.0, bullet_speed: 200.0, score_required: 400,
            spread_angle: 0.3, max_magazines: 3,
            score_level_2: 2000, score_level_3: 6000,
            ..WeaponSettings::empty()
        };

        Self {
            show_debug: false,
            show_cone_debug: false,
            show_grid: false,
            grid_size: default_grid_size(),
            show_weapon_range: false,
            show_hitboxes: false,
            player_count: 1,
            player_speed: 200.0,
            player_hp: 100.0,
            player_regen_rate: 0.0,
            player_regen_delay: 5.0,
            crouch_speed_factor: default_crouch_speed(),
            zombie_speed: 20.0,
            zombie_hp: 30.0,
            zombie_damage: 10.0,
            zombie_damage_cooldown: 0.8,
            big_zombie_hp: 100.0,
            big_zombie_speed: 10.0,
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
            pistol: pistol_levels,
            uzi: make_levels(WeaponType::Uzi, uzi_base),
            grenade: make_levels(WeaponType::Grenade, grenade_base),
            railgun: make_levels(WeaponType::Railgun, railgun_base),
            flamethrower: make_levels(WeaponType::Flamethrower, flamethrower_base),
            shotgun: default_shotgun_levels(),
            laser: default_laser_levels(),
            mine: default_mine_levels(),
            boomerang: default_boomerang_levels(),
            tesla: default_tesla_levels(),
            buzzsaw: default_buzzsaw_levels(),
            rocket: default_rocket_levels(),
            freezegun: default_freezegun_levels(),
            explosion_radius: 80.0,
            crate_spawn_chance: 0.03,
            crate_despawn_time: 15.0,
            base_crate_respawn_time: 30.0,
            flare_duration: 5.0,
            gamemaster_level: 0,
            gm_start_wave: 0,
            gm_start_weapon: 0,
            gm_game_speed: 1.0,
            gm_weapon_dirty: false,
            gm_wave_dirty: false,
            friendly_fire: false,
            explosion_friendly_fire: true,
            knockback_strength_zombie: 150.0,
            knockback_strength_player: 200.0,
            knockback_duration: 0.15,
            volume: 0.5,
            vol_weapons: 1.0,
            vol_enemies: 1.0,
            vol_player: 1.0,
            fullscreen: false,
            pixelation_enabled: false,
            pixel_size: 1.3,
            retro_crt_enabled: false,
            scanline_intensity: 0.3,
            chromatic_aberration: 1.0,
            vignette_intensity: 0.3,
            blood_particles: 4,
            blood_spread_speed: 100.0,
            dismember_chance: 0.30,
            gib_decay_time: 3.0,
            settings_version: 1,
        }
    }
}
