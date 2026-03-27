use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;

#[derive(Component)]
pub struct PauseUiRoot;

#[derive(Component)]
pub struct SettingsPanel; // Legacy-Marker fuer cleanup_settings_panel

#[derive(Component)]
pub struct CategoryTab(pub usize);

#[derive(Component)]
pub struct SettingsValueText(pub usize); // Index in visible_indices

#[derive(Component)]
pub struct SettingsRowMarker(pub usize);

#[derive(Component)]
pub struct SettingsHelpText;

#[derive(Component)]
pub struct SettingsListContainer;

#[derive(Component)]
pub struct SaveButton;

#[derive(Component)]
pub struct DefaultsButton;

#[derive(Component)]
pub struct ResumeButton;

#[derive(Component)]
pub struct FeedbackText;

#[derive(Resource)]
pub struct SettingsUiState {
    pub selected: usize,
    pub repeat_timer: Timer,
    pub open_category: usize,
    pub needs_rebuild: bool,
    pub feedback_timer: f32,
    pub feedback_message: String,
}

impl Default for SettingsUiState {
    fn default() -> Self {
        Self {
            selected: 0,
            repeat_timer: Timer::from_seconds(0.08, TimerMode::Once),
            open_category: 0,
            needs_rebuild: false,
            feedback_timer: 0.0,
            feedback_message: String::new(),
        }
    }
}

// --- Datenmodell (unveraendert) ---

enum Item {
    Category(&'static str),
    SubHeader(String),
    Value(Entry),
}

fn level_header(w: crate::components::WeaponType, lvl: u32) -> Item {
    Item::SubHeader(format!("Level {} - {}", lvl, w.name_at_level(lvl)))
}

struct Entry {
    label: &'static str,
    help: &'static str,
    get: fn(&GameSettings) -> f32,
    set: fn(&mut GameSettings, f32),
    step: f32,
    min: f32,
    max: f32,
    display: DisplayMode,
}

#[derive(Clone, Copy, PartialEq)]
enum DisplayMode {
    Float,
    Bool,
    Percent,
    ReadOnly,
    Carousel(&'static [&'static str]),
}

macro_rules! wl {
    ($label:expr, $help:expr, $field:ident[$lvl:expr] . $prop:ident, $step:expr, $min:expr, $max:expr) => {
        Item::Value(Entry { label: $label, help: $help, get: |s| s.$field[$lvl].$prop as f32, set: |s,v| s.$field[$lvl].$prop = v as _, step: $step, min: $min, max: $max, display: DisplayMode::Float })
    };
}

fn all_items() -> Vec<Item> {
    vec![
        // === Gamemaster ===
        Item::Category("Gamemaster"),
        Item::Value(Entry { label: "Level", help: "Alle Waffen auf gewaehltes Level", get: |s| s.gamemaster_level as f32, set: |s,v| s.gamemaster_level = v as u32, step: 1.0, min: 0.0, max: 3.0, display: DisplayMode::Carousel(&["Aus", "Lv1", "Lv2", "Lv3"]) }),
        Item::Value(Entry { label: "Game Speed", help: "Spielgeschwindigkeit (0.1=Zeitlupe, 2=doppelt)", get: |s| s.gm_game_speed, set: |s,v| s.gm_game_speed = v, step: 0.05, min: 0.1, max: 4.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Startwelle", help: "Welle bei Spielstart (0=normal)", get: |s| s.gm_start_wave as f32, set: |s,v| { s.gm_start_wave = v as u32; s.gm_wave_dirty = true; }, step: 1.0, min: 0.0, max: 100.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Startwaffe", help: "Waffe bei Spielstart", get: |s| s.gm_start_weapon as f32, set: |s,v| { s.gm_start_weapon = v as u32; s.gm_weapon_dirty = true; }, step: 1.0, min: 0.0, max: 12.0, display: DisplayMode::Carousel(&["Pistole", "Shotgun", "Uzi", "Flammenwerfer", "Granate", "Railgun", "Freeze Gun", "Kreissaege", "Tesla", "Mine", "Boomerang", "Rakete", "Laser"]) }),

        // === Spieler ===
        Item::Category("Spieler"),
        Item::Value(Entry { label: "Speed", help: "Bewegungsgeschwindigkeit der Spieler", get: |s| s.player_speed, set: |s,v| s.player_speed = v, step: 10.0, min: 50.0, max: 2000.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "HP", help: "Maximale Lebenspunkte der Spieler", get: |s| s.player_hp, set: |s,v| s.player_hp = v, step: 10.0, min: 10.0, max: 5000.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Regen/s", help: "HP-Regeneration pro Sekunde (0=aus)", get: |s| s.player_regen_rate, set: |s,v| s.player_regen_rate = v, step: 0.5, min: 0.0, max: 100.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Regen-Delay", help: "Sekunden nach Schaden bis Regen startet", get: |s| s.player_regen_delay, set: |s,v| s.player_regen_delay = v, step: 0.5, min: 0.5, max: 30.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Friendly Fire", help: "Spieler-Projektile treffen anderen Spieler", get: |s| if s.friendly_fire { 1.0 } else { 0.0 }, set: |s,v| s.friendly_fire = v >= 0.5, step: 1.0, min: 0.0, max: 1.0, display: DisplayMode::Bool }),
        Item::Value(Entry { label: "Expl. FF", help: "Explosionen verletzen auch Spieler", get: |s| if s.explosion_friendly_fire { 1.0 } else { 0.0 }, set: |s,v| s.explosion_friendly_fire = v >= 0.5, step: 1.0, min: 0.0, max: 1.0, display: DisplayMode::Bool }),
        Item::Value(Entry { label: "KB Spieler", help: "Knockback-Staerke auf Spieler bei Treffer", get: |s| s.knockback_strength_player, set: |s,v| s.knockback_strength_player = v, step: 25.0, min: 0.0, max: 1000.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "KB Dauer", help: "Dauer des Knockback-Effekts in Sekunden", get: |s| s.knockback_duration, set: |s,v| s.knockback_duration = v, step: 0.05, min: 0.05, max: 2.0, display: DisplayMode::Float }),

        // === Zombies ===
        Item::Category("Zombies"),
        Item::Value(Entry { label: "Speed", help: "Bewegungsgeschwindigkeit normaler Zombies", get: |s| s.zombie_speed, set: |s,v| s.zombie_speed = v, step: 5.0, min: 10.0, max: 1000.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "HP", help: "Lebenspunkte normaler Zombies", get: |s| s.zombie_hp, set: |s,v| s.zombie_hp = v, step: 5.0, min: 5.0, max: 2000.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Damage", help: "Schaden den Zombies pro Treffer machen", get: |s| s.zombie_damage, set: |s,v| s.zombie_damage = v, step: 1.0, min: 1.0, max: 500.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Dmg CD", help: "Sekunden zwischen Zombie-Angriffen", get: |s| s.zombie_damage_cooldown, set: |s,v| s.zombie_damage_cooldown = v, step: 0.1, min: 0.1, max: 10.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "KB Staerke", help: "Knockback-Staerke auf Zombies bei Treffer", get: |s| s.knockback_strength_zombie, set: |s,v| s.knockback_strength_zombie = v, step: 25.0, min: 0.0, max: 1000.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Big HP", help: "Lebenspunkte grosser Zombies", get: |s| s.big_zombie_hp, set: |s,v| s.big_zombie_hp = v, step: 10.0, min: 10.0, max: 5000.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Big Speed", help: "Geschwindigkeit grosser Zombies", get: |s| s.big_zombie_speed, set: |s,v| s.big_zombie_speed = v, step: 5.0, min: 10.0, max: 500.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Big Damage", help: "Schaden grosser Zombies", get: |s| s.big_zombie_damage, set: |s,v| s.big_zombie_damage = v, step: 5.0, min: 1.0, max: 500.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Big Scale", help: "Groessenfaktor (1.0=normal, 2.0=doppelt)", get: |s| s.big_zombie_scale, set: |s,v| s.big_zombie_scale = v, step: 0.1, min: 1.1, max: 5.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Big Chance", help: "Wahrscheinlichkeit dass ein Zombie gross ist", get: |s| s.big_zombie_spawn_chance * 100.0, set: |s,v| s.big_zombie_spawn_chance = v / 100.0, step: 5.0, min: 0.0, max: 100.0, display: DisplayMode::Percent }),
        Item::Value(Entry { label: "Big ab Welle", help: "Ab welcher Welle grosse Zombies spawnen", get: |s| s.big_zombie_start_wave as f32, set: |s,v| s.big_zombie_start_wave = v as u32, step: 1.0, min: 1.0, max: 100.0, display: DisplayMode::Float }),

        // === Wellen ===
        Item::Category("Wellen"),
        Item::Value(Entry { label: "Basis-Anzahl", help: "Zombies in der ersten Welle", get: |s| s.wave_base_zombies as f32, set: |s,v| s.wave_base_zombies = v as u32, step: 1.0, min: 1.0, max: 500.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Inkrement", help: "Zusaetzliche Zombies pro Welle (linear)", get: |s| s.wave_zombie_increment as f32, set: |s,v| s.wave_zombie_increment = v as u32, step: 1.0, min: 0.0, max: 100.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Spawn-Intervall", help: "Sekunden zwischen einzelnen Zombie-Spawns", get: |s| s.spawn_interval, set: |s,v| s.spawn_interval = v, step: 0.1, min: 0.1, max: 30.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Wellen-Pause", help: "Pause zwischen Wellen in Sekunden", get: |s| s.wave_pause, set: |s,v| s.wave_pause = v, step: 0.5, min: 0.5, max: 60.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "SI-Abnahme/W", help: "Spawn-Intervall wird pro Welle kuerzer", get: |s| s.spawn_rate_decrease_per_wave, set: |s,v| s.spawn_rate_decrease_per_wave = v, step: 0.01, min: 0.0, max: 1.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Min SI", help: "Minimales Spawn-Intervall (Untergrenze)", get: |s| s.min_spawn_interval, set: |s,v| s.min_spawn_interval = v, step: 0.05, min: 0.05, max: 5.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "%-Modus ab W.", help: "Ab dieser Welle: prozentuale statt lineare Steigerung", get: |s| s.percent_mode_after_wave as f32, set: |s,v| s.percent_mode_after_wave = v as u32, step: 1.0, min: 1.0, max: 100.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "%-Steigerung", help: "Prozentuale Zombie-Steigerung pro Welle (%-Modus)", get: |s| s.zombie_increase_percent, set: |s,v| s.zombie_increase_percent = v, step: 1.0, min: 1.0, max: 200.0, display: DisplayMode::Float }),

        // === Combo ===
        Item::Category("Combo"),
        Item::Value(Entry { label: "Drain-Speed", help: "Wie schnell die Combo-Leiste sinkt", get: |s| s.combo_drain_speed, set: |s,v| s.combo_drain_speed = v, step: 0.01, min: 0.01, max: 5.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Kill-Boost", help: "Combo-Leisten-Boost pro Kill", get: |s| s.combo_kill_boost, set: |s,v| s.combo_kill_boost = v, step: 0.05, min: 0.05, max: 20.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Multi-Decay", help: "Wie schnell der Multiplikator faellt (hoeher=schneller)", get: |s| s.multiplier_decay_rate, set: |s,v| s.multiplier_decay_rate = v, step: 0.1, min: 0.1, max: 10.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Kill-Window", help: "Zeitfenster fuer Kill-Streak in Sekunden", get: |s| s.multiplier_kill_window, set: |s,v| s.multiplier_kill_window = v, step: 0.5, min: 0.5, max: 30.0, display: DisplayMode::Float }),

        // === Kisten ===
        Item::Category("Kisten"),
        Item::Value(Entry { label: "Spawn-Chance", help: "Chance dass ein Zombie eine rote Kiste droppt", get: |s| s.crate_spawn_chance * 100.0, set: |s,v| s.crate_spawn_chance = v / 100.0, step: 5.0, min: 0.0, max: 100.0, display: DisplayMode::Percent }),
        Item::Value(Entry { label: "Despawn-Zeit", help: "Sekunden bis rote Kiste verschwindet", get: |s| s.crate_despawn_time, set: |s,v| s.crate_despawn_time = v, step: 1.0, min: 5.0, max: 120.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Basis-Respawn", help: "Sekunden bis goldene Basis-Kiste respawnt", get: |s| s.base_crate_respawn_time, set: |s,v| s.base_crate_respawn_time = v, step: 5.0, min: 5.0, max: 300.0, display: DisplayMode::Float }),

        // === Anzeige ===
        Item::Category("Audio"),
        Item::Value(Entry { label: "Volume", help: "Gesamtlautstaerke (0-100%)", get: |s| s.volume * 100.0, set: |s,v| s.volume = v / 100.0, step: 0.5, min: 0.0, max: 100.0, display: DisplayMode::Percent }),
        Item::Value(Entry { label: "Waffen-Vol", help: "Lautstaerke Waffen/Explosionen (0-100%)", get: |s| s.vol_weapons * 100.0, set: |s,v| s.vol_weapons = v / 100.0, step: 0.5, min: 0.0, max: 100.0, display: DisplayMode::Percent }),
        Item::Value(Entry { label: "Gegner-Vol", help: "Lautstaerke Zombie-Sounds (0-100%)", get: |s| s.vol_enemies * 100.0, set: |s,v| s.vol_enemies = v / 100.0, step: 0.5, min: 0.0, max: 100.0, display: DisplayMode::Percent }),
        Item::Value(Entry { label: "Spieler-Vol", help: "Lautstaerke Spieler-Sounds (0-100%)", get: |s| s.vol_player * 100.0, set: |s,v| s.vol_player = v / 100.0, step: 0.5, min: 0.0, max: 100.0, display: DisplayMode::Percent }),

        // === Anzeige ===
        Item::Category("Anzeige"),
        Item::Value(Entry { label: "Fullscreen", help: "Vollbildmodus (auch mit F11)", get: |s| if s.fullscreen { 1.0 } else { 0.0 }, set: |s,v| s.fullscreen = v >= 0.5, step: 1.0, min: 0.0, max: 1.0, display: DisplayMode::Bool }),
        Item::Value(Entry { label: "Pixelation", help: "Pixelierter Retro-Look", get: |s| if s.pixelation_enabled { 1.0 } else { 0.0 }, set: |s,v| s.pixelation_enabled = v >= 0.5, step: 1.0, min: 0.0, max: 1.0, display: DisplayMode::Bool }),
        Item::Value(Entry { label: "Pixel-Groesse", help: "Groesse der Pixel (1.1=subtil, 1.5=mittel, 2.0=grob)", get: |s| s.pixel_size, set: |s,v| s.pixel_size = v, step: 0.05, min: 1.05, max: 2.5, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Retro CRT", help: "CRT-Monitor Effekt (Scanlines, Aberration, Vignette)", get: |s| if s.retro_crt_enabled { 1.0 } else { 0.0 }, set: |s,v| s.retro_crt_enabled = v >= 0.5, step: 1.0, min: 0.0, max: 1.0, display: DisplayMode::Bool }),
        Item::Value(Entry { label: "Scanlines", help: "Staerke der CRT-Scanlines (0=aus, 0.5=stark)", get: |s| s.scanline_intensity, set: |s,v| s.scanline_intensity = v, step: 0.05, min: 0.0, max: 0.8, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Aberration", help: "Chromatische Aberration in Pixeln (0=aus, 2=stark)", get: |s| s.chromatic_aberration, set: |s,v| s.chromatic_aberration = v, step: 0.1, min: 0.0, max: 4.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Vignette", help: "Abdunklung an den Raendern (0=aus, 0.5=stark)", get: |s| s.vignette_intensity, set: |s,v| s.vignette_intensity = v, step: 0.05, min: 0.0, max: 1.0, display: DisplayMode::Float }),

        // === Debug ===
        Item::Category("Debug"),
        Item::Value(Entry { label: "Cone-Gizmos", help: "Zeige Cone-Beam Debug-Linien (Flammenwerfer/Freeze)", get: |s| if s.show_cone_debug { 1.0 } else { 0.0 }, set: |s,v| s.show_cone_debug = v >= 0.5, step: 1.0, min: 0.0, max: 1.0, display: DisplayMode::Bool }),
        Item::Value(Entry { label: "Gitter", help: "Zeige Gitter-Overlay im Spielfeld", get: |s| if s.show_grid { 1.0 } else { 0.0 }, set: |s,v| s.show_grid = v >= 0.5, step: 1.0, min: 0.0, max: 1.0, display: DisplayMode::Bool }),
        Item::Value(Entry { label: "Reichweite", help: "Zeige Waffen-Reichweite als Kreis um Spieler", get: |s| if s.show_weapon_range { 1.0 } else { 0.0 }, set: |s,v| s.show_weapon_range = v >= 0.5, step: 1.0, min: 0.0, max: 1.0, display: DisplayMode::Bool }),
        Item::Value(Entry { label: "Hitboxen", help: "Zeige Kollisionsboxen fuer Spieler, Zombies und Projektile", get: |s| if s.show_hitboxes { 1.0 } else { 0.0 }, set: |s,v| s.show_hitboxes = v >= 0.5, step: 1.0, min: 0.0, max: 1.0, display: DisplayMode::Bool }),

        // === Gore ===
        Item::Category("Gore"),
        Item::Value(Entry { label: "Blut-Partikel", help: "Anzahl Blut-Partikel pro Treffer", get: |s| s.blood_particles as f32, set: |s,v| s.blood_particles = v as u32, step: 1.0, min: 0.0, max: 30.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Blut-Spread", help: "Geschwindigkeit der Blut-Ausbreitung", get: |s| s.blood_spread_speed, set: |s,v| s.blood_spread_speed = v, step: 10.0, min: 0.0, max: 500.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Dismember", help: "Chance auf Gliedmassen-Abtrennung bei Treffer", get: |s| s.dismember_chance * 100.0, set: |s,v| s.dismember_chance = v / 100.0, step: 5.0, min: 0.0, max: 100.0, display: DisplayMode::Percent }),
        Item::Value(Entry { label: "Gib-Verfall", help: "Sekunden bis abgetrennte Teile verschwinden", get: |s| s.gib_decay_time, set: |s,v| s.gib_decay_time = v, step: 0.5, min: 0.5, max: 30.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Explosion-Radius", help: "Standard-Explosionsradius (Granate etc.)", get: |s| s.explosion_radius, set: |s,v| s.explosion_radius = v, step: 5.0, min: 20.0, max: 1000.0, display: DisplayMode::Float }),

        // === Pistole ===
        Item::Category("Pistole"),
        wl!("Reload", "Nachladezeit in Sekunden", pistol[0].reload_time, 0.1, 0.1, 30.0),
        level_header(WeaponType::Pistol, 1),
        wl!("Score", "Score zum Freischalten", pistol[0].score_required, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", pistol[0].cooldown, 0.05, 0.02, 10.0),
        wl!("Magazin", "Schuss pro Magazin", pistol[0].magazine, 1.0, 1.0, 500.0),
        wl!("Damage", "Schaden pro Treffer", pistol[0].damage, 1.0, 1.0, 2000.0),
        wl!("Range", "Maximale Reichweite", pistol[0].range, 25.0, 50.0, 5000.0),
        wl!("Proj-Speed", "Projektilgeschwindigkeit", pistol[0].bullet_speed, 25.0, 50.0, 5000.0),
        wl!("Spread", "Streuwinkel (0=exakt)", pistol[0].spread_angle, 0.01, 0.0, 3.14),
        level_header(WeaponType::Pistol, 2),
        wl!("Score", "Score fuer Level 2", pistol[0].score_level_2, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", pistol[1].cooldown, 0.05, 0.02, 10.0),
        wl!("Magazin", "Schuss pro Magazin", pistol[1].magazine, 1.0, 1.0, 500.0),
        wl!("Damage", "Schaden pro Treffer", pistol[1].damage, 1.0, 1.0, 2000.0),
        wl!("Range", "Maximale Reichweite", pistol[1].range, 25.0, 50.0, 5000.0),
        wl!("Proj-Speed", "Projektilgeschwindigkeit", pistol[1].bullet_speed, 25.0, 50.0, 5000.0),
        wl!("Spread", "Streuwinkel (0=exakt)", pistol[1].spread_angle, 0.01, 0.0, 3.14),
        level_header(WeaponType::Pistol, 3),
        wl!("Score", "Score fuer Level 3", pistol[0].score_level_3, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", pistol[2].cooldown, 0.05, 0.02, 10.0),
        wl!("Magazin", "Schuss pro Magazin", pistol[2].magazine, 1.0, 1.0, 500.0),
        wl!("Damage", "Schaden pro Treffer", pistol[2].damage, 1.0, 1.0, 2000.0),
        wl!("Range", "Maximale Reichweite", pistol[2].range, 25.0, 50.0, 5000.0),
        wl!("Proj-Speed", "Projektilgeschwindigkeit", pistol[2].bullet_speed, 25.0, 50.0, 5000.0),
        wl!("Spread", "Streuwinkel (0=exakt)", pistol[2].spread_angle, 0.01, 0.0, 3.14),

        // === Shotgun ===
        Item::Category("Shotgun"),
        wl!("Reload", "Nachladezeit in Sekunden", shotgun[0].reload_time, 0.1, 0.1, 30.0),
        level_header(WeaponType::Shotgun, 1),
        wl!("Score", "Score zum Freischalten", shotgun[0].score_required, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", shotgun[0].cooldown, 0.1, 0.1, 10.0),
        wl!("Magazin", "Schuss pro Magazin", shotgun[0].magazine, 1.0, 1.0, 200.0),
        wl!("Damage", "Schaden pro Pellet", shotgun[0].damage, 1.0, 1.0, 500.0),
        wl!("Pellets", "Anzahl Kugeln pro Schuss", shotgun[0].pellet_count, 1.0, 1.0, 100.0),
        wl!("Spread", "Streuwinkel der Pellets", shotgun[0].spread_angle, 0.05, 0.1, 3.14),
        wl!("Proj-Speed", "Projektilgeschwindigkeit", shotgun[0].bullet_speed, 25.0, 50.0, 5000.0),
        level_header(WeaponType::Shotgun, 2),
        wl!("Score", "Score fuer Level 2", shotgun[0].score_level_2, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", shotgun[1].cooldown, 0.1, 0.1, 10.0),
        wl!("Magazin", "Schuss pro Magazin", shotgun[1].magazine, 1.0, 1.0, 200.0),
        wl!("Damage", "Schaden pro Pellet", shotgun[1].damage, 1.0, 1.0, 500.0),
        wl!("Pellets", "Anzahl Kugeln pro Schuss", shotgun[1].pellet_count, 1.0, 1.0, 100.0),
        wl!("Spread", "Streuwinkel der Pellets", shotgun[1].spread_angle, 0.05, 0.1, 3.14),
        wl!("Proj-Speed", "Projektilgeschwindigkeit", shotgun[1].bullet_speed, 25.0, 50.0, 5000.0),
        level_header(WeaponType::Shotgun, 3),
        wl!("Score", "Score fuer Level 3", shotgun[0].score_level_3, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", shotgun[2].cooldown, 0.1, 0.1, 10.0),
        wl!("Magazin", "Schuss pro Magazin", shotgun[2].magazine, 1.0, 1.0, 200.0),
        wl!("Damage", "Schaden pro Pellet", shotgun[2].damage, 1.0, 1.0, 500.0),
        wl!("Pellets", "Anzahl Kugeln pro Schuss", shotgun[2].pellet_count, 1.0, 1.0, 100.0),
        wl!("Spread", "Streuwinkel der Pellets", shotgun[2].spread_angle, 0.05, 0.1, 3.14),
        wl!("Proj-Speed", "Projektilgeschwindigkeit", shotgun[2].bullet_speed, 25.0, 50.0, 5000.0),

        // === Uzi ===
        Item::Category("Uzi"),
        wl!("Reload", "Nachladezeit in Sekunden", uzi[0].reload_time, 0.1, 0.1, 30.0),
        level_header(WeaponType::Uzi, 1),
        wl!("Score", "Score zum Freischalten", uzi[0].score_required, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", uzi[0].cooldown, 0.01, 0.02, 5.0),
        wl!("Magazin", "Schuss pro Magazin", uzi[0].magazine, 5.0, 5.0, 1000.0),
        wl!("Damage", "Schaden pro Treffer", uzi[0].damage, 1.0, 1.0, 500.0),
        wl!("Proj-Speed", "Projektilgeschwindigkeit", uzi[0].bullet_speed, 25.0, 50.0, 5000.0),
        wl!("Spread", "Streuwinkel (0=exakt)", uzi[0].spread_angle, 0.01, 0.0, 3.14),
        level_header(WeaponType::Uzi, 2),
        wl!("Score", "Score fuer Level 2", uzi[0].score_level_2, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", uzi[1].cooldown, 0.01, 0.02, 5.0),
        wl!("Magazin", "Schuss pro Magazin", uzi[1].magazine, 5.0, 5.0, 1000.0),
        wl!("Damage", "Schaden pro Treffer", uzi[1].damage, 1.0, 1.0, 500.0),
        wl!("Proj-Speed", "Projektilgeschwindigkeit", uzi[1].bullet_speed, 25.0, 50.0, 5000.0),
        wl!("Spread", "Streuwinkel (0=exakt)", uzi[1].spread_angle, 0.01, 0.0, 3.14),
        level_header(WeaponType::Uzi, 3),
        wl!("Score", "Score fuer Level 3", uzi[0].score_level_3, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", uzi[2].cooldown, 0.01, 0.02, 5.0),
        wl!("Magazin", "Schuss pro Magazin", uzi[2].magazine, 5.0, 5.0, 1000.0),
        wl!("Damage", "Schaden pro Treffer", uzi[2].damage, 1.0, 1.0, 500.0),
        wl!("Proj-Speed", "Projektilgeschwindigkeit", uzi[2].bullet_speed, 25.0, 50.0, 5000.0),
        wl!("Spread", "Streuwinkel (0=exakt)", uzi[2].spread_angle, 0.01, 0.0, 3.14),

        // === Flammenwerfer ===
        Item::Category("Flammenwerfer"),
        wl!("Reload", "Nachladezeit in Sekunden", flamethrower[0].reload_time, 0.1, 0.1, 30.0),
        level_header(WeaponType::Flamethrower, 1),
        wl!("Score", "Score zum Freischalten", flamethrower[0].score_required, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Flammen-Partikeln", flamethrower[0].cooldown, 0.01, 0.01, 5.0),
        wl!("Magazin", "Partikel pro Tank", flamethrower[0].magazine, 5.0, 10.0, 2000.0),
        wl!("Damage", "Schaden pro Partikel", flamethrower[0].damage, 0.5, 0.5, 200.0),
        wl!("Range", "Reichweite der Flammen", flamethrower[0].range, 10.0, 50.0, 2000.0),
        wl!("Proj-Speed", "Flammengeschwindigkeit", flamethrower[0].bullet_speed, 10.0, 50.0, 5000.0),
        level_header(WeaponType::Flamethrower, 2),
        wl!("Score", "Score fuer Level 2", flamethrower[0].score_level_2, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Flammen-Partikeln", flamethrower[1].cooldown, 0.01, 0.01, 5.0),
        wl!("Magazin", "Partikel pro Tank", flamethrower[1].magazine, 5.0, 10.0, 2000.0),
        wl!("Damage", "Schaden pro Partikel", flamethrower[1].damage, 0.5, 0.5, 200.0),
        wl!("Range", "Reichweite der Flammen", flamethrower[1].range, 10.0, 50.0, 2000.0),
        wl!("Proj-Speed", "Flammengeschwindigkeit", flamethrower[1].bullet_speed, 10.0, 50.0, 5000.0),
        level_header(WeaponType::Flamethrower, 3),
        wl!("Score", "Score fuer Level 3", flamethrower[0].score_level_3, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Flammen-Partikeln", flamethrower[2].cooldown, 0.01, 0.01, 5.0),
        wl!("Magazin", "Partikel pro Tank", flamethrower[2].magazine, 5.0, 10.0, 2000.0),
        wl!("Damage", "Schaden pro Partikel", flamethrower[2].damage, 0.5, 0.5, 200.0),
        wl!("Range", "Reichweite der Flammen", flamethrower[2].range, 10.0, 50.0, 2000.0),
        wl!("Proj-Speed", "Flammengeschwindigkeit", flamethrower[2].bullet_speed, 10.0, 50.0, 5000.0),

        // === Granate ===
        Item::Category("Granate"),
        wl!("Reload", "Nachladezeit in Sekunden", grenade[0].reload_time, 0.1, 0.1, 30.0),
        level_header(WeaponType::Grenade, 1),
        wl!("Score", "Score zum Freischalten", grenade[0].score_required, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Wuerfen", grenade[0].cooldown, 0.1, 0.2, 30.0),
        wl!("Magazin", "Granaten pro Magazin", grenade[0].magazine, 1.0, 1.0, 200.0),
        wl!("Damage", "Explosionsschaden", grenade[0].damage, 5.0, 5.0, 5000.0),
        wl!("Range", "Wurfweite (beeinflusst Zuender)", grenade[0].range, 25.0, 50.0, 5000.0),
        wl!("Proj-Speed", "Wurfgeschwindigkeit", grenade[0].bullet_speed, 25.0, 50.0, 5000.0),
        level_header(WeaponType::Grenade, 2),
        wl!("Score", "Score fuer Level 2", grenade[0].score_level_2, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Wuerfen", grenade[1].cooldown, 0.1, 0.2, 30.0),
        wl!("Magazin", "Granaten pro Magazin", grenade[1].magazine, 1.0, 1.0, 200.0),
        wl!("Damage", "Explosionsschaden", grenade[1].damage, 5.0, 5.0, 5000.0),
        wl!("Range", "Wurfweite (beeinflusst Zuender)", grenade[1].range, 25.0, 50.0, 5000.0),
        wl!("Proj-Speed", "Wurfgeschwindigkeit", grenade[1].bullet_speed, 25.0, 50.0, 5000.0),
        level_header(WeaponType::Grenade, 3),
        wl!("Score", "Score fuer Level 3", grenade[0].score_level_3, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Wuerfen", grenade[2].cooldown, 0.1, 0.2, 30.0),
        wl!("Magazin", "Granaten pro Magazin", grenade[2].magazine, 1.0, 1.0, 200.0),
        wl!("Damage", "Explosionsschaden", grenade[2].damage, 5.0, 5.0, 5000.0),
        wl!("Range", "Wurfweite (beeinflusst Zuender)", grenade[2].range, 25.0, 50.0, 5000.0),
        wl!("Proj-Speed", "Wurfgeschwindigkeit", grenade[2].bullet_speed, 25.0, 50.0, 5000.0),

        // === Railgun ===
        Item::Category("Railgun"),
        wl!("Reload", "Nachladezeit in Sekunden", railgun[0].reload_time, 0.1, 0.1, 30.0),
        level_header(WeaponType::Railgun, 1),
        wl!("Score", "Score zum Freischalten", railgun[0].score_required, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", railgun[0].cooldown, 0.1, 0.1, 30.0),
        wl!("Magazin", "Schuss pro Magazin", railgun[0].magazine, 1.0, 1.0, 200.0),
        wl!("Damage", "Schaden pro Treffer", railgun[0].damage, 5.0, 5.0, 5000.0),
        wl!("Range", "Maximale Reichweite", railgun[0].range, 50.0, 100.0, 10000.0),
        wl!("Proj-Speed", "Projektilgeschwindigkeit", railgun[0].bullet_speed, 50.0, 100.0, 10000.0),
        wl!("Spread", "Streuwinkel (0=exakt)", railgun[0].spread_angle, 0.01, 0.0, 3.14),
        wl!("Pierce", "Gegner die durchdrungen werden", railgun[0].pierce_count, 1.0, 1.0, 999.0),
        level_header(WeaponType::Railgun, 2),
        wl!("Score", "Score fuer Level 2", railgun[0].score_level_2, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", railgun[1].cooldown, 0.1, 0.1, 30.0),
        wl!("Magazin", "Schuss pro Magazin", railgun[1].magazine, 1.0, 1.0, 200.0),
        wl!("Damage", "Schaden pro Treffer", railgun[1].damage, 5.0, 5.0, 5000.0),
        wl!("Range", "Maximale Reichweite", railgun[1].range, 50.0, 100.0, 10000.0),
        wl!("Proj-Speed", "Projektilgeschwindigkeit", railgun[1].bullet_speed, 50.0, 100.0, 10000.0),
        wl!("Spread", "Streuwinkel (0=exakt)", railgun[1].spread_angle, 0.01, 0.0, 3.14),
        wl!("Pierce", "Gegner die durchdrungen werden", railgun[1].pierce_count, 1.0, 1.0, 999.0),
        level_header(WeaponType::Railgun, 3),
        wl!("Score", "Score fuer Level 3", railgun[0].score_level_3, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", railgun[2].cooldown, 0.1, 0.1, 30.0),
        wl!("Magazin", "Schuss pro Magazin", railgun[2].magazine, 1.0, 1.0, 200.0),
        wl!("Damage", "Schaden pro Treffer", railgun[2].damage, 5.0, 5.0, 5000.0),
        wl!("Range", "Maximale Reichweite", railgun[2].range, 50.0, 100.0, 10000.0),
        wl!("Proj-Speed", "Projektilgeschwindigkeit", railgun[2].bullet_speed, 50.0, 100.0, 10000.0),
        wl!("Spread", "Streuwinkel (0=exakt)", railgun[2].spread_angle, 0.01, 0.0, 3.14),
        wl!("Pierce", "Gegner die durchdrungen werden", railgun[2].pierce_count, 1.0, 1.0, 999.0),

        // === Freeze Gun ===
        Item::Category("Freeze Gun"),
        wl!("Reload", "Nachladezeit in Sekunden", freezegun[0].reload_time, 0.1, 0.1, 30.0),
        level_header(WeaponType::FreezeGun, 1),
        wl!("Score", "Score zum Freischalten", freezegun[0].score_required, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", freezegun[0].cooldown, 0.05, 0.05, 10.0),
        wl!("Magazin", "Schuss pro Magazin", freezegun[0].magazine, 1.0, 1.0, 500.0),
        wl!("Damage", "Schaden pro Treffer", freezegun[0].damage, 0.5, 0.5, 200.0),
        wl!("Slow-Faktor", "Verlangsamung (0.25=75% langsamer)", freezegun[0].slow_factor, 0.05, 0.05, 0.99),
        wl!("Slow-Dauer", "Dauer der Verlangsamung in Sek.", freezegun[0].slow_duration, 0.5, 0.5, 60.0),
        wl!("Proj-Speed", "Projektilgeschwindigkeit", freezegun[0].bullet_speed, 25.0, 50.0, 5000.0),
        level_header(WeaponType::FreezeGun, 2),
        wl!("Score", "Score fuer Level 2", freezegun[0].score_level_2, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", freezegun[1].cooldown, 0.05, 0.05, 10.0),
        wl!("Magazin", "Schuss pro Magazin", freezegun[1].magazine, 1.0, 1.0, 500.0),
        wl!("Damage", "Schaden pro Treffer", freezegun[1].damage, 0.5, 0.5, 200.0),
        wl!("Slow-Faktor", "Verlangsamung (0.25=75% langsamer)", freezegun[1].slow_factor, 0.05, 0.05, 0.99),
        wl!("Slow-Dauer", "Dauer der Verlangsamung in Sek.", freezegun[1].slow_duration, 0.5, 0.5, 60.0),
        wl!("Proj-Speed", "Projektilgeschwindigkeit", freezegun[1].bullet_speed, 25.0, 50.0, 5000.0),
        level_header(WeaponType::FreezeGun, 3),
        wl!("Score", "Score fuer Level 3", freezegun[0].score_level_3, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", freezegun[2].cooldown, 0.05, 0.05, 10.0),
        wl!("Magazin", "Schuss pro Magazin", freezegun[2].magazine, 1.0, 1.0, 500.0),
        wl!("Damage", "Schaden pro Treffer", freezegun[2].damage, 0.5, 0.5, 200.0),
        wl!("Slow-Faktor", "Verlangsamung (0.25=75% langsamer)", freezegun[2].slow_factor, 0.05, 0.05, 0.99),
        wl!("Slow-Dauer", "Dauer der Verlangsamung in Sek.", freezegun[2].slow_duration, 0.5, 0.5, 60.0),
        wl!("Proj-Speed", "Projektilgeschwindigkeit", freezegun[2].bullet_speed, 25.0, 50.0, 5000.0),

        // === Kreissaege ===
        Item::Category("Kreissaege"),
        wl!("Reload", "Nachladezeit in Sekunden", buzzsaw[0].reload_time, 0.1, 0.1, 30.0),
        level_header(WeaponType::Buzzsaw, 1),
        wl!("Score", "Score zum Freischalten", buzzsaw[0].score_required, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Wuerfen", buzzsaw[0].cooldown, 0.1, 0.2, 30.0),
        wl!("Magazin", "Saegen pro Magazin", buzzsaw[0].magazine, 1.0, 1.0, 200.0),
        wl!("Damage", "Schaden pro Treffer", buzzsaw[0].damage, 1.0, 1.0, 500.0),
        wl!("Proj-Speed", "Fluggeschwindigkeit der Saege", buzzsaw[0].bullet_speed, 10.0, 30.0, 2000.0),
        wl!("Range", "Maximale Flugdistanz", buzzsaw[0].range, 50.0, 100.0, 5000.0),
        wl!("Pierce", "Gegner die durchdrungen werden", buzzsaw[0].pierce_count, 1.0, 1.0, 999.0),
        level_header(WeaponType::Buzzsaw, 2),
        wl!("Score", "Score fuer Level 2", buzzsaw[0].score_level_2, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Wuerfen", buzzsaw[1].cooldown, 0.1, 0.2, 30.0),
        wl!("Magazin", "Saegen pro Magazin", buzzsaw[1].magazine, 1.0, 1.0, 200.0),
        wl!("Damage", "Schaden pro Treffer", buzzsaw[1].damage, 1.0, 1.0, 500.0),
        wl!("Proj-Speed", "Fluggeschwindigkeit der Saege", buzzsaw[1].bullet_speed, 10.0, 30.0, 2000.0),
        wl!("Range", "Maximale Flugdistanz", buzzsaw[1].range, 50.0, 100.0, 5000.0),
        wl!("Pierce", "Gegner die durchdrungen werden", buzzsaw[1].pierce_count, 1.0, 1.0, 999.0),
        level_header(WeaponType::Buzzsaw, 3),
        wl!("Score", "Score fuer Level 3", buzzsaw[0].score_level_3, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Wuerfen", buzzsaw[2].cooldown, 0.1, 0.2, 30.0),
        wl!("Magazin", "Saegen pro Magazin", buzzsaw[2].magazine, 1.0, 1.0, 200.0),
        wl!("Damage", "Schaden pro Treffer", buzzsaw[2].damage, 1.0, 1.0, 500.0),
        wl!("Proj-Speed", "Fluggeschwindigkeit der Saege", buzzsaw[2].bullet_speed, 10.0, 30.0, 2000.0),
        wl!("Range", "Maximale Flugdistanz", buzzsaw[2].range, 50.0, 100.0, 5000.0),
        wl!("Pierce", "Gegner die durchdrungen werden", buzzsaw[2].pierce_count, 1.0, 1.0, 999.0),

        // === Tesla ===
        Item::Category("Tesla"),
        wl!("Reload", "Nachladezeit in Sekunden", tesla[0].reload_time, 0.1, 0.1, 30.0),
        level_header(WeaponType::Tesla, 1),
        wl!("Score", "Score zum Freischalten", tesla[0].score_required, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", tesla[0].cooldown, 0.05, 0.1, 10.0),
        wl!("Magazin", "Schuss pro Magazin", tesla[0].magazine, 1.0, 1.0, 300.0),
        wl!("Damage", "Schaden Haupttreffer", tesla[0].damage, 1.0, 1.0, 1000.0),
        wl!("Chains", "Anzahl Kettenblitz-Spruenge", tesla[0].chain_count, 1.0, 0.0, 50.0),
        wl!("Chain-Range", "Max. Distanz fuer Kettenblitz", tesla[0].chain_range, 10.0, 20.0, 2000.0),
        wl!("Proj-Speed", "Projektilgeschwindigkeit", tesla[0].bullet_speed, 25.0, 50.0, 5000.0),
        level_header(WeaponType::Tesla, 2),
        wl!("Score", "Score fuer Level 2", tesla[0].score_level_2, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", tesla[1].cooldown, 0.05, 0.1, 10.0),
        wl!("Magazin", "Schuss pro Magazin", tesla[1].magazine, 1.0, 1.0, 300.0),
        wl!("Damage", "Schaden Haupttreffer", tesla[1].damage, 1.0, 1.0, 1000.0),
        wl!("Chains", "Anzahl Kettenblitz-Spruenge", tesla[1].chain_count, 1.0, 0.0, 50.0),
        wl!("Chain-Range", "Max. Distanz fuer Kettenblitz", tesla[1].chain_range, 10.0, 20.0, 2000.0),
        wl!("Proj-Speed", "Projektilgeschwindigkeit", tesla[1].bullet_speed, 25.0, 50.0, 5000.0),
        level_header(WeaponType::Tesla, 3),
        wl!("Score", "Score fuer Level 3", tesla[0].score_level_3, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", tesla[2].cooldown, 0.05, 0.1, 10.0),
        wl!("Magazin", "Schuss pro Magazin", tesla[2].magazine, 1.0, 1.0, 300.0),
        wl!("Damage", "Schaden Haupttreffer", tesla[2].damage, 1.0, 1.0, 1000.0),
        wl!("Chains", "Anzahl Kettenblitz-Spruenge", tesla[2].chain_count, 1.0, 0.0, 50.0),
        wl!("Chain-Range", "Max. Distanz fuer Kettenblitz", tesla[2].chain_range, 10.0, 20.0, 2000.0),
        wl!("Proj-Speed", "Projektilgeschwindigkeit", tesla[2].bullet_speed, 25.0, 50.0, 5000.0),

        // === Mine ===
        Item::Category("Mine"),
        wl!("Reload", "Nachladezeit in Sekunden", mine[0].reload_time, 0.1, 0.1, 30.0),
        level_header(WeaponType::Mine, 1),
        wl!("Score", "Score zum Freischalten", mine[0].score_required, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Platzierungen", mine[0].cooldown, 0.1, 0.2, 30.0),
        wl!("Magazin", "Minen pro Magazin", mine[0].magazine, 1.0, 1.0, 200.0),
        wl!("Damage", "Explosionsschaden", mine[0].damage, 5.0, 5.0, 5000.0),
        wl!("Trigger-R", "Ausloeseradius (Zombie-Naehe)", mine[0].trigger_radius, 5.0, 10.0, 500.0),
        wl!("Expl-Radius", "Explosionsradius", mine[0].explosion_radius_override, 5.0, 20.0, 1000.0),
        level_header(WeaponType::Mine, 2),
        wl!("Score", "Score fuer Level 2", mine[0].score_level_2, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Platzierungen", mine[1].cooldown, 0.1, 0.2, 30.0),
        wl!("Magazin", "Minen pro Magazin", mine[1].magazine, 1.0, 1.0, 200.0),
        wl!("Damage", "Explosionsschaden", mine[1].damage, 5.0, 5.0, 5000.0),
        wl!("Trigger-R", "Ausloeseradius (Zombie-Naehe)", mine[1].trigger_radius, 5.0, 10.0, 500.0),
        wl!("Expl-Radius", "Explosionsradius", mine[1].explosion_radius_override, 5.0, 20.0, 1000.0),
        level_header(WeaponType::Mine, 3),
        wl!("Score", "Score fuer Level 3", mine[0].score_level_3, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Platzierungen", mine[2].cooldown, 0.1, 0.2, 30.0),
        wl!("Magazin", "Minen pro Magazin", mine[2].magazine, 1.0, 1.0, 200.0),
        wl!("Damage", "Explosionsschaden", mine[2].damage, 5.0, 5.0, 5000.0),
        wl!("Trigger-R", "Ausloeseradius (Zombie-Naehe)", mine[2].trigger_radius, 5.0, 10.0, 500.0),
        wl!("Expl-Radius", "Explosionsradius", mine[2].explosion_radius_override, 5.0, 20.0, 1000.0),

        // === Boomerang ===
        Item::Category("Boomerang"),
        wl!("Reload", "Nachladezeit in Sekunden", boomerang[0].reload_time, 0.1, 0.1, 30.0),
        level_header(WeaponType::Boomerang, 1),
        wl!("Score", "Score zum Freischalten", boomerang[0].score_required, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Wuerfen", boomerang[0].cooldown, 0.1, 0.2, 30.0),
        wl!("Magazin", "Wuerfe vor Reload", boomerang[0].magazine, 1.0, 1.0, 100.0),
        wl!("Damage", "Schaden pro Treffer", boomerang[0].damage, 1.0, 1.0, 1000.0),
        wl!("Range", "Maximale Flugdistanz", boomerang[0].range, 25.0, 50.0, 5000.0),
        wl!("Proj-Speed", "Fluggeschwindigkeit", boomerang[0].bullet_speed, 25.0, 100.0, 3000.0),
        level_header(WeaponType::Boomerang, 2),
        wl!("Score", "Score fuer Level 2", boomerang[0].score_level_2, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Wuerfen", boomerang[1].cooldown, 0.1, 0.2, 30.0),
        wl!("Magazin", "Wuerfe vor Reload", boomerang[1].magazine, 1.0, 1.0, 100.0),
        wl!("Damage", "Schaden pro Treffer", boomerang[1].damage, 1.0, 1.0, 1000.0),
        wl!("Range", "Maximale Flugdistanz", boomerang[1].range, 25.0, 50.0, 5000.0),
        wl!("Proj-Speed", "Fluggeschwindigkeit", boomerang[1].bullet_speed, 25.0, 100.0, 3000.0),
        level_header(WeaponType::Boomerang, 3),
        wl!("Score", "Score fuer Level 3", boomerang[0].score_level_3, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Wuerfen", boomerang[2].cooldown, 0.1, 0.2, 30.0),
        wl!("Magazin", "Wuerfe vor Reload", boomerang[2].magazine, 1.0, 1.0, 100.0),
        wl!("Damage", "Schaden pro Treffer", boomerang[2].damage, 1.0, 1.0, 1000.0),
        wl!("Range", "Maximale Flugdistanz", boomerang[2].range, 25.0, 50.0, 5000.0),
        wl!("Proj-Speed", "Fluggeschwindigkeit", boomerang[2].bullet_speed, 25.0, 100.0, 3000.0),

        // === Rakete ===
        Item::Category("Rakete"),
        wl!("Reload", "Nachladezeit in Sekunden", rocket[0].reload_time, 0.1, 0.1, 30.0),
        level_header(WeaponType::Rocket, 1),
        wl!("Score", "Score zum Freischalten", rocket[0].score_required, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", rocket[0].cooldown, 0.1, 0.3, 30.0),
        wl!("Magazin", "Raketen pro Magazin", rocket[0].magazine, 1.0, 1.0, 100.0),
        wl!("Damage", "Explosionsschaden", rocket[0].damage, 5.0, 5.0, 5000.0),
        wl!("Range", "Maximale Flugdistanz", rocket[0].range, 25.0, 50.0, 5000.0),
        wl!("Proj-Speed", "Raketengeschwindigkeit", rocket[0].bullet_speed, 25.0, 50.0, 5000.0),
        wl!("Expl-Radius", "Explosionsradius", rocket[0].explosion_radius_override, 5.0, 20.0, 1000.0),
        level_header(WeaponType::Rocket, 2),
        wl!("Score", "Score fuer Level 2", rocket[0].score_level_2, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", rocket[1].cooldown, 0.1, 0.3, 30.0),
        wl!("Magazin", "Raketen pro Magazin", rocket[1].magazine, 1.0, 1.0, 100.0),
        wl!("Damage", "Explosionsschaden", rocket[1].damage, 5.0, 5.0, 5000.0),
        wl!("Range", "Maximale Flugdistanz", rocket[1].range, 25.0, 50.0, 5000.0),
        wl!("Proj-Speed", "Raketengeschwindigkeit", rocket[1].bullet_speed, 25.0, 50.0, 5000.0),
        wl!("Expl-Radius", "Explosionsradius", rocket[1].explosion_radius_override, 5.0, 20.0, 1000.0),
        level_header(WeaponType::Rocket, 3),
        wl!("Score", "Score fuer Level 3", rocket[0].score_level_3, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", rocket[2].cooldown, 0.1, 0.3, 30.0),
        wl!("Magazin", "Raketen pro Magazin", rocket[2].magazine, 1.0, 1.0, 100.0),
        wl!("Damage", "Explosionsschaden", rocket[2].damage, 5.0, 5.0, 5000.0),
        wl!("Range", "Maximale Flugdistanz", rocket[2].range, 25.0, 50.0, 5000.0),
        wl!("Proj-Speed", "Raketengeschwindigkeit", rocket[2].bullet_speed, 25.0, 50.0, 5000.0),
        wl!("Expl-Radius", "Explosionsradius", rocket[2].explosion_radius_override, 5.0, 20.0, 1000.0),

        // === Laser ===
        Item::Category("Laser"),
        wl!("Reload", "Nachladezeit in Sekunden", laser[0].reload_time, 0.1, 0.1, 30.0),
        level_header(WeaponType::Laser, 1),
        wl!("Score", "Score zum Freischalten", laser[0].score_required, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", laser[0].cooldown, 0.01, 0.01, 5.0),
        wl!("Magazin", "Schuss pro Batterie", laser[0].magazine, 5.0, 10.0, 2000.0),
        wl!("Damage", "Schaden pro Treffer", laser[0].damage, 0.5, 0.5, 500.0),
        wl!("Range", "Maximale Reichweite", laser[0].range, 50.0, 100.0, 10000.0),
        wl!("Proj-Speed", "Lasergeschwindigkeit", laser[0].bullet_speed, 100.0, 500.0, 10000.0),
        wl!("Pierce", "Gegner die durchdrungen werden", laser[0].pierce_count, 1.0, 1.0, 999.0),
        level_header(WeaponType::Laser, 2),
        wl!("Score", "Score fuer Level 2", laser[0].score_level_2, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", laser[1].cooldown, 0.01, 0.01, 5.0),
        wl!("Magazin", "Schuss pro Batterie", laser[1].magazine, 5.0, 10.0, 2000.0),
        wl!("Damage", "Schaden pro Treffer", laser[1].damage, 0.5, 0.5, 500.0),
        wl!("Range", "Maximale Reichweite", laser[1].range, 50.0, 100.0, 10000.0),
        wl!("Proj-Speed", "Lasergeschwindigkeit", laser[1].bullet_speed, 100.0, 500.0, 10000.0),
        wl!("Pierce", "Gegner die durchdrungen werden", laser[1].pierce_count, 1.0, 1.0, 999.0),
        level_header(WeaponType::Laser, 3),
        wl!("Score", "Score fuer Level 3", laser[0].score_level_3, 1.0, 0.0, 99999.0),
        wl!("Cooldown", "Sekunden zwischen Schuessen", laser[2].cooldown, 0.01, 0.01, 5.0),
        wl!("Magazin", "Schuss pro Batterie", laser[2].magazine, 5.0, 10.0, 2000.0),
        wl!("Damage", "Schaden pro Treffer", laser[2].damage, 0.5, 0.5, 500.0),
        wl!("Range", "Maximale Reichweite", laser[2].range, 50.0, 100.0, 10000.0),
        wl!("Proj-Speed", "Lasergeschwindigkeit", laser[2].bullet_speed, 100.0, 500.0, 10000.0),
        wl!("Pierce", "Gegner die durchdrungen werden", laser[2].pierce_count, 1.0, 1.0, 999.0),
    ]
}

// --- Hilfsfunktionen ---

fn build_categories(items: &[Item]) -> Vec<(usize, &'static str)> {
    items.iter().enumerate()
        .filter_map(|(i, item)| match item {
            Item::Category(name) => Some((i, *name)),
            _ => None,
        })
        .collect()
}

fn visible_indices(items: &[Item], open_cat: usize) -> Vec<usize> {
    let cats = build_categories(items);
    if let Some((start_idx, _)) = cats.get(open_cat) {
        let end = cats.get(open_cat + 1).map(|(i, _)| *i).unwrap_or(items.len());
        ((*start_idx + 1)..end)
            .filter(|i| matches!(items[*i], Item::Value(_) | Item::SubHeader(_)))
            .collect()
    } else {
        Vec::new()
    }
}

fn format_value(val: f32, display: DisplayMode) -> String {
    match display {
        DisplayMode::Bool => if val >= 0.5 { "ON".into() } else { "OFF".into() },
        DisplayMode::Percent => format!("{:.0}%", val),
        DisplayMode::ReadOnly => format!("{:.2}", val),
        DisplayMode::Float => format!("{:.2}", val),
        DisplayMode::Carousel(labels) => {
            let idx = val as usize;
            if idx < labels.len() { format!("< {} >", labels[idx]) } else { format!("< {} >", idx) }
        }
    }
}

// --- Farben ---

const TAB_NORMAL: Color = Color::srgb(0.2, 0.2, 0.25);
const TAB_ACTIVE: Color = Color::srgb(0.15, 0.4, 0.2);
const TAB_HOVER: Color = Color::srgb(0.3, 0.3, 0.35);
const ROW_NORMAL: Color = Color::srgba(0.0, 0.0, 0.0, 0.0);
const ROW_SELECTED: Color = Color::srgba(0.0, 0.4, 0.0, 0.3);
const BTN_NORMAL: Color = Color::srgb(0.25, 0.25, 0.3);
const BTN_HOVER: Color = Color::srgb(0.35, 0.35, 0.45);

// --- Setup (OnEnter Paused) ---

pub fn setup_pause_ui(
    mut commands: Commands,
    settings: Res<GameSettings>,
    ui_state: Res<SettingsUiState>,
) {
    let items = all_items();
    let cats = build_categories(&items);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            PauseUiRoot,
        ))
        .with_children(|root| {
            // Panel
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                width: Val::Px(500.0),
                max_height: Val::Percent(90.0),
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|panel| {
                // Titel
                panel.spawn((
                    Text::new("KLOTZKOEPFE - PAUSE"),
                    TextFont { font_size: 22.0, ..default() },
                    TextColor(Color::srgb(0.0, 1.0, 0.0)),
                ));

                // Kategorie-Tabs
                panel.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(4.0),
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|tabs_row| {
                    for (cat_idx, (_, name)) in cats.iter().enumerate() {
                        let is_active = cat_idx == ui_state.open_category;
                        let bg = if is_active { TAB_ACTIVE } else { TAB_NORMAL };
                        tabs_row
                            .spawn((
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                                    ..default()
                                },
                                BackgroundColor(bg),
                                CategoryTab(cat_idx),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new(*name),
                                    TextFont { font_size: 12.0, ..default() },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                });

                // Settings-Liste Container
                panel.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        min_height: Val::Px(300.0),
                        max_height: Val::Px(500.0),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    ScrollPosition::default(),
                    SettingsListContainer,
                ))
                .with_children(|list| {
                    spawn_settings_rows(list, &items, &settings, &ui_state);
                });

                // Feedback Text
                panel.spawn((
                    Text::new(""),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.2, 1.0, 0.2)),
                    Node { height: Val::Px(18.0), ..default() },
                    FeedbackText,
                ));

                // Help Text
                panel.spawn((
                    Text::new(""),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    SettingsHelpText,
                ));

                // Hints
                panel.spawn((
                    Text::new("W/S: Navigieren | A/D: Wert | Q/E: Kategorie | Shift: 10x"),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(Color::srgb(0.4, 0.4, 0.4)),
                ));

                // Action Buttons
                panel.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    margin: UiRect::top(Val::Px(5.0)),
                    ..default()
                })
                .with_children(|row| {
                    spawn_action_button(row, "Speichern [F5]", SaveButton);
                    spawn_action_button(row, "Defaults [F6]", DefaultsButton);
                    spawn_action_button(row, "Weiter [ESC]", ResumeButton);
                });
            });
        });
}

fn spawn_settings_rows(
    parent: &mut ChildSpawnerCommands,
    items: &[Item],
    settings: &GameSettings,
    ui_state: &SettingsUiState,
) {
    let visible = visible_indices(items, ui_state.open_category);

    for (row_idx, &item_idx) in visible.iter().enumerate() {
        match &items[item_idx] {
            Item::SubHeader(text) => {
                parent.spawn((
                    Node {
                        padding: UiRect::new(Val::Px(4.0), Val::Px(4.0), Val::Px(6.0), Val::Px(2.0)),
                        ..default()
                    },
                    SettingsRowMarker(row_idx),
                ))
                .with_children(|row| {
                    row.spawn((
                        Text::new(text.clone()),
                        TextFont { font_size: 11.0, ..default() },
                        TextColor(Color::srgb(1.0, 0.8, 0.2)),
                    ));
                });
            }
            Item::Value(entry) => {
                let is_selected = row_idx == ui_state.selected;
                let bg = if is_selected { ROW_SELECTED } else { ROW_NORMAL };
                let val = (entry.get)(settings);
                let val_str = format_value(val, entry.display);
                let is_readonly = entry.display == DisplayMode::ReadOnly;

                let label_color = if is_readonly {
                    Color::srgb(0.5, 0.5, 0.5)
                } else if is_selected {
                    Color::srgb(0.0, 1.0, 0.0)
                } else {
                    Color::srgb(0.8, 0.8, 0.8)
                };

                parent
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(bg),
                        SettingsRowMarker(row_idx),
                    ))
                    .with_children(|row| {
                        row.spawn((
                            Text::new(entry.label),
                            TextFont { font_size: 13.0, ..default() },
                            TextColor(label_color),
                        ));
                        row.spawn((
                            Text::new(val_str),
                            TextFont { font_size: 13.0, ..default() },
                            TextColor(if is_selected { Color::srgb(1.0, 1.0, 0.0) } else { Color::srgb(0.7, 0.7, 0.7) }),
                            SettingsValueText(row_idx),
                        ));
                    });
            }
            _ => {}
        }
    }
}

fn spawn_action_button(parent: &mut ChildSpawnerCommands, label: &str, marker: impl Component) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(BTN_NORMAL),
            marker,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
}

// --- Keyboard Input (leicht angepasst) ---

pub fn settings_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut ui_state: ResMut<SettingsUiState>,
    mut settings: ResMut<GameSettings>,
    mut next_state: ResMut<NextState<GameState>>,
    mut list_scroll: Query<(&mut ScrollPosition, &ComputedNode), With<SettingsListContainer>>,
) {
    ui_state.repeat_timer.tick(time.delta());
    let items = all_items();
    let visible = visible_indices(&items, ui_state.open_category);
    let count = visible.len();
    if count == 0 { return; }

    // Navigation (ArrowUp/W, ArrowDown/S) - SubHeaders ueberspringen
    if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
        let mut next = (ui_state.selected + count - 1) % count;
        for _ in 0..count {
            if matches!(items[visible[next]], Item::SubHeader(_)) {
                next = (next + count - 1) % count;
            } else { break; }
        }
        ui_state.selected = next;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        let mut next = (ui_state.selected + 1) % count;
        for _ in 0..count {
            if matches!(items[visible[next]], Item::SubHeader(_)) {
                next = (next + 1) % count;
            } else { break; }
        }
        ui_state.selected = next;
    }

    // Auto-Scroll zur selektierten Zeile (proportional basierend auf ComputedNode)
    if let Ok((mut scroll, computed)) = list_scroll.single_mut() {
        let content_h = computed.content_size().y * computed.inverse_scale_factor();
        let view_h = computed.size().y * computed.inverse_scale_factor();
        let max_scroll = (content_h - view_h).max(0.0);
        let sel = ui_state.selected.min(count.saturating_sub(1));
        // Proportionale Position: sel / count * content_h
        let frac = if count > 0 { sel as f32 / count as f32 } else { 0.0 };
        let row_h_approx = if count > 0 { content_h / count as f32 } else { 20.0 };
        let target_top = frac * content_h;
        let target_bottom = target_top + row_h_approx;
        if target_top < scroll.0.y {
            scroll.0.y = target_top.max(0.0);
        } else if target_bottom > scroll.0.y + view_h {
            scroll.0.y = (target_bottom - view_h).min(max_scroll);
        }
    }

    // Kategorie-Wechsel (Tab, Q/E)
    let cat_switch = if keyboard.just_pressed(KeyCode::Tab) {
        if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
            Some(false)
        } else {
            Some(true)
        }
    } else if keyboard.just_pressed(KeyCode::KeyE) {
        Some(true)
    } else if keyboard.just_pressed(KeyCode::KeyQ) {
        Some(false)
    } else {
        None
    };

    if let Some(forward) = cat_switch {
        let cats = build_categories(&items);
        if !cats.is_empty() {
            let dir = if forward { 1 } else { cats.len() - 1 };
            ui_state.open_category = (ui_state.open_category + dir) % cats.len();
            ui_state.selected = 0;
            ui_state.needs_rebuild = true;
        }
    }

    // Werte aendern (ArrowLeft/A = decrease, ArrowRight/D = increase)
    let selected_item_idx = visible[ui_state.selected.min(count - 1)];

    let pressing_increase = keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD);
    let pressing_decrease = keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA);
    let just_increase = keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::KeyD);
    let just_decrease = keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::KeyA);

    let can_change = just_increase || just_decrease
        || ((pressing_increase || pressing_decrease) && ui_state.repeat_timer.is_finished());

    if can_change {
        if let Item::Value(entry) = &items[selected_item_idx] {
            if entry.display != DisplayMode::ReadOnly {
                ui_state.repeat_timer.reset();
                let val = (entry.get)(&settings);
                let step = if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
                    entry.step * 10.0
                } else {
                    entry.step
                };
                if pressing_increase {
                    (entry.set)(&mut settings, (val + step).min(entry.max));
                }
                if pressing_decrease {
                    (entry.set)(&mut settings, (val - step).max(entry.min));
                }
            }
        }
    }

    // F5: Speichern
    if keyboard.just_pressed(KeyCode::F5) {
        settings.save();
        ui_state.feedback_message = "Gespeichert!".into();
        ui_state.feedback_timer = 2.0;
    }

    // F6: Defaults
    if keyboard.just_pressed(KeyCode::F6) {
        let show = settings.show_debug;
        let show_cone = settings.show_cone_debug;
        let show_grid = settings.show_grid;
        let show_range = settings.show_weapon_range;
        let show_hitboxes = settings.show_hitboxes;
        *settings = GameSettings::default();
        settings.show_debug = show;
        settings.show_cone_debug = show_cone;
        settings.show_grid = show_grid;
        settings.show_weapon_range = show_range;
        settings.show_hitboxes = show_hitboxes;
        ui_state.feedback_message = "Defaults geladen!".into();
        ui_state.feedback_timer = 2.0;
        ui_state.needs_rebuild = true;
    }
}

// --- UI Update System (jeden Frame, nur Text+Farbe aendern) ---

pub fn settings_update_ui(
    mut commands: Commands,
    settings: Res<GameSettings>,
    time: Res<Time>,
    mut ui_state: ResMut<SettingsUiState>,
    mut value_texts: Query<(&mut Text, &mut TextColor, &SettingsValueText)>,
    mut row_bgs: Query<(&mut BackgroundColor, &SettingsRowMarker), Without<SettingsValueText>>,
    mut help_text: Query<&mut Text, (With<SettingsHelpText>, Without<SettingsValueText>, Without<SettingsRowMarker>, Without<FeedbackText>)>,
    mut feedback_query: Query<(&mut Text, &mut TextColor), (With<FeedbackText>, Without<SettingsHelpText>, Without<SettingsValueText>, Without<SettingsRowMarker>)>,
    mut list_container: Query<(Entity, &mut ScrollPosition, &ComputedNode), With<SettingsListContainer>>,
    mut mouse_wheel: MessageReader<bevy::input::mouse::MouseWheel>,
    mut tab_query: Query<(&mut BackgroundColor, &Interaction, &CategoryTab), (Without<SettingsRowMarker>, Without<SettingsValueText>)>,
    root_query: Query<Entity, With<PauseUiRoot>>,
) {
    // Feedback Timer
    if ui_state.feedback_timer > 0.0 {
        ui_state.feedback_timer -= time.delta_secs();
    }
    if let Ok((mut text, mut color)) = feedback_query.single_mut() {
        if ui_state.feedback_timer > 0.0 {
            if **text != ui_state.feedback_message {
                **text = ui_state.feedback_message.clone();
            }
            let alpha = (ui_state.feedback_timer / 2.0).min(1.0);
            color.0 = Color::srgba(0.2, 1.0, 0.2, alpha);
        } else if !text.is_empty() {
            **text = String::new();
        }
    }
    let items = all_items();
    let visible = visible_indices(&items, ui_state.open_category);
    let count = visible.len();
    let sel = ui_state.selected.min(count.saturating_sub(1));

    // Rebuild bei Kategorie-Wechsel
    if ui_state.needs_rebuild {
        ui_state.needs_rebuild = false;
        if let Ok((container, _, _)) = list_container.single() {
            // Container despawnen und neu spawnen geht nicht einfach,
            // daher despawnen wir den ganzen Root und triggern setup neu
            // Einfacher: despawn container, re-create
            commands.entity(container).try_despawn();
            // Neuen Container mit Rows spawnen - wird als Kind des Panels eingefuegt
            // Da der Container weg ist, muessen wir ihn am PauseUiRoot neu anhaengen
            // Einfacher Ansatz: ganzes UI neu spawnen
        }
        // Kompletten Pause-UI Rebuild (einfach und zuverlaessig)
        for entity in root_query.iter() {
            commands.entity(entity).try_despawn();
        }
        // Wir spawnen das ganze UI neu im naechsten Frame nicht moeglich hier,
        // daher inline:
        let cats_rebuild = build_categories(&items);
        commands
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
                PauseUiRoot,
            ))
            .with_children(|root| {
                root.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Px(500.0),
                    max_height: Val::Percent(90.0),
                    padding: UiRect::all(Val::Px(20.0)),
                    row_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("KLOTZKOEPFE - PAUSE"),
                        TextFont { font_size: 22.0, ..default() },
                        TextColor(Color::srgb(0.0, 1.0, 0.0)),
                    ));
                    panel.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(4.0),
                        row_gap: Val::Px(4.0),
                        ..default()
                    })
                    .with_children(|tabs_row| {
                        for (cat_idx, (_, name)) in cats_rebuild.iter().enumerate() {
                            let is_active = cat_idx == ui_state.open_category;
                            let bg = if is_active { TAB_ACTIVE } else { TAB_NORMAL };
                            tabs_row
                                .spawn((
                                    Button,
                                    Node {
                                        padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                                        ..default()
                                    },
                                    BackgroundColor(bg),
                                    CategoryTab(cat_idx),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new(*name),
                                        TextFont { font_size: 12.0, ..default() },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                        }
                    });
                    panel.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(2.0),
                            min_height: Val::Px(300.0),
                            max_height: Val::Px(500.0),
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                        ScrollPosition::default(),
                        SettingsListContainer,
                    ))
                    .with_children(|list| {
                        spawn_settings_rows(list, &items, &settings, &ui_state);
                    });
                    panel.spawn((
                        Text::new(""),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::srgb(0.2, 1.0, 0.2)),
                        Node { height: Val::Px(18.0), ..default() },
                        FeedbackText,
                    ));
                    panel.spawn((
                        Text::new(""),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                        SettingsHelpText,
                    ));
                    panel.spawn((
                        Text::new("W/S: Navigieren | A/D: Wert | Q/E: Kategorie | Shift: 10x"),
                        TextFont { font_size: 11.0, ..default() },
                        TextColor(Color::srgb(0.4, 0.4, 0.4)),
                    ));
                    panel.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(10.0),
                        margin: UiRect::top(Val::Px(5.0)),
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_action_button(row, "Speichern [F5]", SaveButton);
                        spawn_action_button(row, "Defaults [F6]", DefaultsButton);
                        spawn_action_button(row, "Weiter [ESC]", ResumeButton);
                    });
                });
            });
        // Tab-Farben updaten
        for (mut bg, interaction, tab) in tab_query.iter_mut() {
            let is_active = tab.0 == ui_state.open_category;
            *bg = if is_active {
                BackgroundColor(TAB_ACTIVE)
            } else if *interaction == Interaction::Hovered {
                BackgroundColor(TAB_HOVER)
            } else {
                BackgroundColor(TAB_NORMAL)
            };
        }
        return;
    }

    // Werte + Highlight updaten
    for (mut text, mut color, svt) in value_texts.iter_mut() {
        if svt.0 < visible.len() {
            let item_idx = visible[svt.0];
            if let Item::Value(entry) = &items[item_idx] {
                let val = (entry.get)(&settings);
                let val_str = format_value(val, entry.display);
                if **text != val_str {
                    **text = val_str;
                }
                let is_selected = svt.0 == sel;
                let target = if is_selected { Color::srgb(1.0, 1.0, 0.0) } else { Color::srgb(0.7, 0.7, 0.7) };
                color.0 = target;
            }
        }
    }

    // Row Highlight
    for (mut bg, row) in row_bgs.iter_mut() {
        let is_selected = row.0 == sel;
        *bg = BackgroundColor(if is_selected { ROW_SELECTED } else { ROW_NORMAL });
    }

    // Mouse-Wheel Scrolling
    if let Ok((_, mut scroll, computed)) = list_container.single_mut() {
        let max_scroll = ((computed.content_size().y - computed.size().y) * computed.inverse_scale_factor()).max(0.0);
        for ev in mouse_wheel.read() {
            scroll.0.y = (scroll.0.y - ev.y * 30.0).clamp(0.0, max_scroll);
        }
    }

    // Help Text
    if let Ok(mut text) = help_text.single_mut() {
        if sel < visible.len() {
            let item_idx = visible[sel];
            if let Item::Value(entry) = &items[item_idx] {
                let help = format!("? {}", entry.help);
                if **text != help {
                    **text = help;
                }
            }
        }
    }

    // Tab-Farben
    for (mut bg, interaction, tab) in tab_query.iter_mut() {
        let is_active = tab.0 == ui_state.open_category;
        *bg = if is_active {
            BackgroundColor(TAB_ACTIVE)
        } else if *interaction == Interaction::Hovered {
            BackgroundColor(TAB_HOVER)
        } else {
            BackgroundColor(TAB_NORMAL)
        };
    }
}

// --- Button Interaction ---

pub fn settings_button_interaction(
    mut ui_state: ResMut<SettingsUiState>,
    mut settings: ResMut<GameSettings>,
    mut next_state: ResMut<NextState<GameState>>,
    tab_query: Query<(&Interaction, &CategoryTab), Changed<Interaction>>,
    mut save_query: Query<(&Interaction, &mut BackgroundColor), (With<SaveButton>, Without<DefaultsButton>, Without<ResumeButton>)>,
    mut defaults_query: Query<(&Interaction, &mut BackgroundColor), (With<DefaultsButton>, Without<SaveButton>, Without<ResumeButton>)>,
    mut resume_query: Query<(&Interaction, &mut BackgroundColor), (With<ResumeButton>, Without<SaveButton>, Without<DefaultsButton>)>,
) {
    // Tab-Clicks
    for (interaction, tab) in tab_query.iter() {
        if *interaction == Interaction::Pressed {
            ui_state.open_category = tab.0;
            ui_state.selected = 0;
            ui_state.needs_rebuild = true;
        }
    }

    // Action Buttons
    for (interaction, mut bg) in save_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                settings.save();
                ui_state.feedback_message = "Gespeichert!".into();
                ui_state.feedback_timer = 2.0;
            }
            Interaction::Hovered => { *bg = BackgroundColor(BTN_HOVER); }
            Interaction::None => { *bg = BackgroundColor(BTN_NORMAL); }
        }
    }
    for (interaction, mut bg) in defaults_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                let show = settings.show_debug;
                let show_cone = settings.show_cone_debug;
                let show_grid = settings.show_grid;
                let show_range = settings.show_weapon_range;
                *settings = GameSettings::default();
                settings.show_debug = show;
                settings.show_cone_debug = show_cone;
                settings.show_grid = show_grid;
                settings.show_weapon_range = show_range;
                ui_state.feedback_message = "Defaults geladen!".into();
                ui_state.feedback_timer = 2.0;
                ui_state.needs_rebuild = true;
            }
            Interaction::Hovered => { *bg = BackgroundColor(BTN_HOVER); }
            Interaction::None => { *bg = BackgroundColor(BTN_NORMAL); }
        }
    }
    for (interaction, mut bg) in resume_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => { next_state.set(GameState::Playing); }
            Interaction::Hovered => { *bg = BackgroundColor(BTN_HOVER); }
            Interaction::None => { *bg = BackgroundColor(BTN_NORMAL); }
        }
    }
}

pub fn gm_wave_apply(
    mut settings: ResMut<GameSettings>,
    mut wave: ResMut<WaveState>,
) {
    if !settings.gm_wave_dirty {
        return;
    }
    settings.gm_wave_dirty = false;
    if settings.gm_start_wave > 0 {
        wave.current_wave = settings.gm_start_wave.saturating_sub(1);
        wave.active = false;
        wave.pausing = false;
    }
}

// --- Debug Gizmos ---

pub fn grid_overlay_gizmos(
    mut gizmos: Gizmos,
    settings: Res<GameSettings>,
) {
    if !settings.show_grid { return; }

    let half_w = WINDOW_WIDTH / 2.0 - WALL_THICKNESS;
    let half_h = WINDOW_HEIGHT / 2.0 - WALL_THICKNESS;
    let color = Color::srgba(1.0, 1.0, 1.0, 0.08);
    let z = 19.0;
    let step = 50.0;

    // Vertikale Linien
    let mut x = -half_w + step;
    while x < half_w {
        gizmos.line(Vec3::new(x, -half_h, z), Vec3::new(x, half_h, z), color);
        x += step;
    }
    // Horizontale Linien
    let mut y = -half_h + step;
    while y < half_h {
        gizmos.line(Vec3::new(-half_w, y, z), Vec3::new(half_w, y, z), color);
        y += step;
    }

    // Achsen hervorheben
    let axis_color = Color::srgba(1.0, 1.0, 1.0, 0.15);
    gizmos.line(Vec3::new(0.0, -half_h, z), Vec3::new(0.0, half_h, z), axis_color);
    gizmos.line(Vec3::new(-half_w, 0.0, z), Vec3::new(half_w, 0.0, z), axis_color);
}

pub fn weapon_range_gizmos(
    mut gizmos: Gizmos,
    settings: Res<GameSettings>,
    score: Res<Score>,
    player_query: Query<(&Player, &Transform)>,
) {
    if !settings.show_weapon_range { return; }

    for (player, transform) in player_query.iter() {
        let lvl = settings.weapon_level(player.weapon, score.points);
        let ws = settings.weapon_at_level(player.weapon, lvl);
        let pos = transform.translation.truncate();
        let range = ws.range;
        if range <= 0.0 { continue; }

        let color = match player.id {
            PlayerId::P1 => Color::srgba(0.2, 0.8, 0.2, 0.25),
            PlayerId::P2 => Color::srgba(0.2, 0.4, 0.9, 0.25),
        };

        gizmos.circle_2d(Isometry2d::from_translation(pos), range, color);

        // Richtungslinie
        let dir_color = match player.id {
            PlayerId::P1 => Color::srgba(0.2, 0.8, 0.2, 0.4),
            PlayerId::P2 => Color::srgba(0.2, 0.4, 0.9, 0.4),
        };
        let tip = pos + player.facing * range;
        gizmos.line(
            Vec3::new(pos.x, pos.y, 19.0),
            Vec3::new(tip.x, tip.y, 19.0),
            dir_color,
        );
    }
}

pub fn hitbox_gizmos(
    mut gizmos: Gizmos,
    settings: Res<GameSettings>,
    player_query: Query<&Transform, With<Player>>,
    zombie_query: Query<(&Transform, Option<&BigZombie>), With<Zombie>>,
    bullet_query: Query<&Transform, With<Bullet>>,
) {
    if !settings.show_hitboxes { return; }
    let z = 19.0;

    // Spieler-Hitboxen (gruen)
    for transform in player_query.iter() {
        let pos = transform.translation;
        gizmos.rect(
            Isometry3d::from_translation(Vec3::new(pos.x, pos.y, z)),
            PLAYER_SIZE,
            Color::srgba(0.0, 1.0, 0.0, 0.5),
        );
    }

    // Zombie-Hitboxen (rot)
    for (transform, big) in zombie_query.iter() {
        let pos = transform.translation;
        let size = if big.is_some() {
            ZOMBIE_SIZE * settings.big_zombie_scale
        } else {
            ZOMBIE_SIZE
        };
        gizmos.rect(
            Isometry3d::from_translation(Vec3::new(pos.x, pos.y, z)),
            size,
            Color::srgba(1.0, 0.0, 0.0, 0.4),
        );
    }

    // Projektil-Hitboxen (gelb)
    for transform in bullet_query.iter() {
        let pos = transform.translation;
        gizmos.rect(
            Isometry3d::from_translation(Vec3::new(pos.x, pos.y, z)),
            Vec2::new(8.0, 8.0),
            Color::srgba(1.0, 1.0, 0.0, 0.5),
        );
    }
}

// --- Cleanup ---

pub fn cleanup_settings_panel(
    mut commands: Commands,
    root_query: Query<Entity, With<PauseUiRoot>>,
    // Legacy: alte SettingsPanel Sprites falls noch vorhanden
    panel_query: Query<Entity, With<SettingsPanel>>,
) {
    for entity in root_query.iter() {
        commands.entity(entity).try_despawn();
    }
    for entity in panel_query.iter() {
        commands.entity(entity).try_despawn();
    }
}
