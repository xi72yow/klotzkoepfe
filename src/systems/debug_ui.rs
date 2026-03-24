use bevy::prelude::*;

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
    Value(Entry),
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
}

macro_rules! w {
    ($label:expr, $help:expr, $field:ident . $prop:ident, $step:expr, $min:expr, $max:expr) => {
        Item::Value(Entry { label: $label, help: $help, get: |s| s.$field.$prop as f32, set: |s,v| s.$field.$prop = v as _, step: $step, min: $min, max: $max, display: DisplayMode::Float })
    };
}

fn all_items() -> Vec<Item> {
    vec![
        // === Spieler ===
        Item::Category("Spieler"),
        Item::Value(Entry { label: "Gamemaster", help: "0=Aus, 1/2/3=Alle Waffen auf Lv1/Lv2/Lv3", get: |s| s.gamemaster_level as f32, set: |s,v| s.gamemaster_level = v as u32, step: 1.0, min: 0.0, max: 3.0, display: DisplayMode::Float }),
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
        Item::Category("Anzeige"),
        Item::Value(Entry { label: "Volume", help: "Gesamtlautstaerke (0-100%)", get: |s| s.volume * 100.0, set: |s,v| s.volume = v / 100.0, step: 0.5, min: 0.0, max: 100.0, display: DisplayMode::Percent }),
        Item::Value(Entry { label: "Waffen-Vol", help: "Lautstaerke Waffen/Explosionen (0-100%)", get: |s| s.vol_weapons * 100.0, set: |s,v| s.vol_weapons = v / 100.0, step: 0.5, min: 0.0, max: 100.0, display: DisplayMode::Percent }),
        Item::Value(Entry { label: "Gegner-Vol", help: "Lautstaerke Zombie-Sounds (0-100%)", get: |s| s.vol_enemies * 100.0, set: |s,v| s.vol_enemies = v / 100.0, step: 0.5, min: 0.0, max: 100.0, display: DisplayMode::Percent }),
        Item::Value(Entry { label: "Spieler-Vol", help: "Lautstaerke Spieler-Sounds (0-100%)", get: |s| s.vol_player * 100.0, set: |s,v| s.vol_player = v / 100.0, step: 0.5, min: 0.0, max: 100.0, display: DisplayMode::Percent }),
        Item::Value(Entry { label: "Fullscreen", help: "Vollbildmodus (auch mit F11)", get: |s| if s.fullscreen { 1.0 } else { 0.0 }, set: |s,v| s.fullscreen = v >= 0.5, step: 1.0, min: 0.0, max: 1.0, display: DisplayMode::Bool }),
        Item::Value(Entry { label: "Pixelation", help: "Pixelierter Retro-Look", get: |s| if s.pixelation_enabled { 1.0 } else { 0.0 }, set: |s,v| s.pixelation_enabled = v >= 0.5, step: 1.0, min: 0.0, max: 1.0, display: DisplayMode::Bool }),
        Item::Value(Entry { label: "Pixel-Groesse", help: "Groesse der Pixel (1.1=subtil, 1.5=mittel, 2.0=grob)", get: |s| s.pixel_size, set: |s,v| s.pixel_size = v, step: 0.05, min: 1.05, max: 2.5, display: DisplayMode::Float }),

        // === Gore ===
        Item::Category("Gore"),
        Item::Value(Entry { label: "Blut-Partikel", help: "Anzahl Blut-Partikel pro Treffer", get: |s| s.blood_particles as f32, set: |s,v| s.blood_particles = v as u32, step: 1.0, min: 0.0, max: 30.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Blut-Spread", help: "Geschwindigkeit der Blut-Ausbreitung", get: |s| s.blood_spread_speed, set: |s,v| s.blood_spread_speed = v, step: 10.0, min: 0.0, max: 500.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Dismember", help: "Chance auf Gliedmassen-Abtrennung bei Treffer", get: |s| s.dismember_chance * 100.0, set: |s,v| s.dismember_chance = v / 100.0, step: 5.0, min: 0.0, max: 100.0, display: DisplayMode::Percent }),
        Item::Value(Entry { label: "Gib-Verfall", help: "Sekunden bis abgetrennte Teile verschwinden", get: |s| s.gib_decay_time, set: |s,v| s.gib_decay_time = v, step: 0.5, min: 0.5, max: 30.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Explosion-Radius", help: "Standard-Explosionsradius (Granate etc.)", get: |s| s.explosion_radius, set: |s,v| s.explosion_radius = v, step: 5.0, min: 20.0, max: 1000.0, display: DisplayMode::Float }),

        // === Pistole ===
        Item::Category("Pistole"),
        w!("Cooldown", "Sekunden zwischen Schuessen", pistol.cooldown, 0.05, 0.02, 10.0),
        w!("Magazin", "Schuss pro Magazin", pistol.magazine, 1.0, 1.0, 500.0),
        w!("Damage", "Schaden pro Treffer", pistol.damage, 1.0, 1.0, 2000.0),
        w!("Range", "Maximale Reichweite", pistol.range, 25.0, 50.0, 5000.0),
        w!("Proj-Speed", "Projektilgeschwindigkeit", pistol.bullet_speed, 25.0, 50.0, 5000.0),
        w!("Spread", "Streuwinkel (0=exakt)", pistol.spread_angle, 0.01, 0.0, 3.14),
        w!("Score", "Score zum Freischalten", pistol.score_required, 1.0, 0.0, 99999.0),
        w!("Lv2 Score", "Score fuer Level 2", pistol.score_level_2, 1.0, 0.0, 99999.0),
        w!("Lv3 Score", "Score fuer Level 3", pistol.score_level_3, 1.0, 0.0, 99999.0),
        Item::Value(Entry { label: "Lv2 Dmg", help: "Berechneter Schaden auf Level 2", get: |s| s.weapon_at_level(crate::components::WeaponType::Pistol, 2).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),
        Item::Value(Entry { label: "Lv3 Dmg", help: "Berechneter Schaden auf Level 3", get: |s| s.weapon_at_level(crate::components::WeaponType::Pistol, 3).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),

        // === Shotgun ===
        Item::Category("Shotgun"),
        w!("Cooldown", "Sekunden zwischen Schuessen", shotgun.cooldown, 0.1, 0.1, 10.0),
        w!("Magazin", "Schuss pro Magazin", shotgun.magazine, 1.0, 1.0, 200.0),
        w!("Damage", "Schaden pro Pellet", shotgun.damage, 1.0, 1.0, 500.0),
        w!("Pellets", "Anzahl Kugeln pro Schuss", shotgun.pellet_count, 1.0, 1.0, 100.0),
        w!("Spread", "Streuwinkel der Pellets", shotgun.spread_angle, 0.05, 0.1, 3.14),
        w!("Proj-Speed", "Projektilgeschwindigkeit", shotgun.bullet_speed, 25.0, 50.0, 5000.0),
        w!("Score", "Score zum Freischalten", shotgun.score_required, 1.0, 0.0, 99999.0),
        w!("Lv2 Score", "Score fuer Level 2", shotgun.score_level_2, 1.0, 0.0, 99999.0),
        w!("Lv3 Score", "Score fuer Level 3", shotgun.score_level_3, 1.0, 0.0, 99999.0),
        Item::Value(Entry { label: "Lv2 Dmg", help: "Berechneter Schaden Lv2", get: |s| s.weapon_at_level(crate::components::WeaponType::Shotgun, 2).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),
        Item::Value(Entry { label: "Lv3 Dmg", help: "Berechneter Schaden Lv3", get: |s| s.weapon_at_level(crate::components::WeaponType::Shotgun, 3).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),

        // === Uzi ===
        Item::Category("Uzi"),
        w!("Cooldown", "Sekunden zwischen Schuessen", uzi.cooldown, 0.01, 0.02, 5.0),
        w!("Magazin", "Schuss pro Magazin", uzi.magazine, 5.0, 5.0, 1000.0),
        w!("Damage", "Schaden pro Treffer", uzi.damage, 1.0, 1.0, 500.0),
        w!("Proj-Speed", "Projektilgeschwindigkeit", uzi.bullet_speed, 25.0, 50.0, 5000.0),
        w!("Spread", "Streuwinkel (0=exakt)", uzi.spread_angle, 0.01, 0.0, 3.14),
        w!("Score", "Score zum Freischalten", uzi.score_required, 1.0, 0.0, 99999.0),
        w!("Lv2 Score", "Score fuer Level 2", uzi.score_level_2, 1.0, 0.0, 99999.0),
        w!("Lv3 Score", "Score fuer Level 3", uzi.score_level_3, 1.0, 0.0, 99999.0),
        Item::Value(Entry { label: "Lv2 Dmg", help: "Berechneter Schaden Lv2", get: |s| s.weapon_at_level(crate::components::WeaponType::Uzi, 2).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),
        Item::Value(Entry { label: "Lv3 Dmg", help: "Berechneter Schaden Lv3", get: |s| s.weapon_at_level(crate::components::WeaponType::Uzi, 3).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),

        // === Flammenwerfer ===
        Item::Category("Flammenwerfer"),
        w!("Cooldown", "Sekunden zwischen Flammen-Partikeln", flamethrower.cooldown, 0.01, 0.01, 5.0),
        w!("Magazin", "Partikel pro Tank", flamethrower.magazine, 5.0, 10.0, 2000.0),
        w!("Damage", "Schaden pro Partikel", flamethrower.damage, 0.5, 0.5, 200.0),
        w!("Range", "Reichweite der Flammen", flamethrower.range, 10.0, 50.0, 2000.0),
        w!("Proj-Speed", "Flammengeschwindigkeit", flamethrower.bullet_speed, 10.0, 50.0, 5000.0),
        w!("Score", "Score zum Freischalten", flamethrower.score_required, 1.0, 0.0, 99999.0),
        w!("Lv2 Score", "Score fuer Level 2", flamethrower.score_level_2, 1.0, 0.0, 99999.0),
        w!("Lv3 Score", "Score fuer Level 3", flamethrower.score_level_3, 1.0, 0.0, 99999.0),
        Item::Value(Entry { label: "Lv2 Dmg", help: "Berechneter Schaden Lv2", get: |s| s.weapon_at_level(crate::components::WeaponType::Flamethrower, 2).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),
        Item::Value(Entry { label: "Lv3 Dmg", help: "Berechneter Schaden Lv3", get: |s| s.weapon_at_level(crate::components::WeaponType::Flamethrower, 3).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),

        // === Granate ===
        Item::Category("Granate"),
        w!("Cooldown", "Sekunden zwischen Wuerfen", grenade.cooldown, 0.1, 0.2, 30.0),
        w!("Magazin", "Granaten pro Magazin", grenade.magazine, 1.0, 1.0, 200.0),
        w!("Damage", "Explosionsschaden", grenade.damage, 5.0, 5.0, 5000.0),
        w!("Range", "Wurfweite (beeinflusst Zuender)", grenade.range, 25.0, 50.0, 5000.0),
        w!("Proj-Speed", "Wurfgeschwindigkeit", grenade.bullet_speed, 25.0, 50.0, 5000.0),
        Item::Value(Entry { label: "Fuse (berechnet)", help: "Zuenderzeit = Range / Speed", get: |s| s.grenade.range / s.grenade.bullet_speed.max(1.0), set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),
        w!("Score", "Score zum Freischalten", grenade.score_required, 1.0, 0.0, 99999.0),
        w!("Lv2 Score", "Score fuer Level 2", grenade.score_level_2, 1.0, 0.0, 99999.0),
        w!("Lv3 Score", "Score fuer Level 3", grenade.score_level_3, 1.0, 0.0, 99999.0),
        Item::Value(Entry { label: "Lv2 Dmg", help: "Berechneter Schaden Lv2", get: |s| s.weapon_at_level(crate::components::WeaponType::Grenade, 2).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),
        Item::Value(Entry { label: "Lv3 Dmg", help: "Berechneter Schaden Lv3", get: |s| s.weapon_at_level(crate::components::WeaponType::Grenade, 3).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),

        // === Railgun ===
        Item::Category("Railgun"),
        w!("Cooldown", "Sekunden zwischen Schuessen", railgun.cooldown, 0.1, 0.1, 30.0),
        w!("Magazin", "Schuss pro Magazin", railgun.magazine, 1.0, 1.0, 200.0),
        w!("Damage", "Schaden pro Treffer (durchdringend)", railgun.damage, 5.0, 5.0, 5000.0),
        w!("Range", "Maximale Reichweite", railgun.range, 50.0, 100.0, 10000.0),
        w!("Proj-Speed", "Projektilgeschwindigkeit", railgun.bullet_speed, 50.0, 100.0, 10000.0),
        w!("Spread", "Streuwinkel (0=exakt)", railgun.spread_angle, 0.01, 0.0, 3.14),
        w!("Pierce", "Gegner die durchdrungen werden", railgun.pierce_count, 1.0, 1.0, 999.0),
        w!("Score", "Score zum Freischalten", railgun.score_required, 1.0, 0.0, 99999.0),
        w!("Lv2 Score", "Score fuer Level 2", railgun.score_level_2, 1.0, 0.0, 99999.0),
        w!("Lv3 Score", "Score fuer Level 3", railgun.score_level_3, 1.0, 0.0, 99999.0),
        Item::Value(Entry { label: "Lv2 Dmg", help: "Berechneter Schaden Lv2", get: |s| s.weapon_at_level(crate::components::WeaponType::Railgun, 2).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),
        Item::Value(Entry { label: "Lv3 Dmg", help: "Berechneter Schaden Lv3", get: |s| s.weapon_at_level(crate::components::WeaponType::Railgun, 3).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),

        // === Freeze Gun ===
        Item::Category("Freeze Gun"),
        w!("Cooldown", "Sekunden zwischen Schuessen", freezegun.cooldown, 0.05, 0.05, 10.0),
        w!("Magazin", "Schuss pro Magazin", freezegun.magazine, 1.0, 1.0, 500.0),
        w!("Damage", "Schaden pro Treffer", freezegun.damage, 0.5, 0.5, 200.0),
        w!("Slow-Faktor", "Verlangsamung (0.25=75% langsamer)", freezegun.slow_factor, 0.05, 0.05, 0.99),
        w!("Slow-Dauer", "Dauer der Verlangsamung in Sek.", freezegun.slow_duration, 0.5, 0.5, 60.0),
        w!("Proj-Speed", "Projektilgeschwindigkeit", freezegun.bullet_speed, 25.0, 50.0, 5000.0),
        w!("Score", "Score zum Freischalten", freezegun.score_required, 1.0, 0.0, 99999.0),
        w!("Lv2 Score", "Score fuer Level 2", freezegun.score_level_2, 1.0, 0.0, 99999.0),
        w!("Lv3 Score", "Score fuer Level 3", freezegun.score_level_3, 1.0, 0.0, 99999.0),
        Item::Value(Entry { label: "Lv2 Dmg", help: "Berechneter Schaden Lv2", get: |s| s.weapon_at_level(crate::components::WeaponType::FreezeGun, 2).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),
        Item::Value(Entry { label: "Lv3 Dmg", help: "Berechneter Schaden Lv3", get: |s| s.weapon_at_level(crate::components::WeaponType::FreezeGun, 3).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),

        // === Kreissaege ===
        Item::Category("Kreissaege"),
        w!("Cooldown", "Sekunden zwischen Wuerfen", buzzsaw.cooldown, 0.1, 0.2, 30.0),
        w!("Magazin", "Saegen pro Magazin", buzzsaw.magazine, 1.0, 1.0, 200.0),
        w!("Damage", "Schaden pro Treffer (durchdringend)", buzzsaw.damage, 1.0, 1.0, 500.0),
        w!("Proj-Speed", "Fluggeschwindigkeit der Saege", buzzsaw.bullet_speed, 10.0, 30.0, 2000.0),
        w!("Range", "Maximale Flugdistanz", buzzsaw.range, 50.0, 100.0, 5000.0),
        w!("Pierce", "Gegner die durchdrungen werden", buzzsaw.pierce_count, 1.0, 1.0, 999.0),
        w!("Score", "Score zum Freischalten", buzzsaw.score_required, 1.0, 0.0, 99999.0),
        w!("Lv2 Score", "Score fuer Level 2", buzzsaw.score_level_2, 1.0, 0.0, 99999.0),
        w!("Lv3 Score", "Score fuer Level 3", buzzsaw.score_level_3, 1.0, 0.0, 99999.0),
        Item::Value(Entry { label: "Lv2 Dmg", help: "Berechneter Schaden Lv2", get: |s| s.weapon_at_level(crate::components::WeaponType::Buzzsaw, 2).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),
        Item::Value(Entry { label: "Lv3 Dmg", help: "Berechneter Schaden Lv3", get: |s| s.weapon_at_level(crate::components::WeaponType::Buzzsaw, 3).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),

        // === Tesla ===
        Item::Category("Tesla"),
        w!("Cooldown", "Sekunden zwischen Schuessen", tesla.cooldown, 0.05, 0.1, 10.0),
        w!("Magazin", "Schuss pro Magazin", tesla.magazine, 1.0, 1.0, 300.0),
        w!("Damage", "Schaden Haupttreffer", tesla.damage, 1.0, 1.0, 1000.0),
        w!("Chains", "Anzahl Kettenblitz-Spruenge", tesla.chain_count, 1.0, 0.0, 50.0),
        w!("Chain-Range", "Max. Distanz fuer Kettenblitz", tesla.chain_range, 10.0, 20.0, 2000.0),
        w!("Proj-Speed", "Projektilgeschwindigkeit", tesla.bullet_speed, 25.0, 50.0, 5000.0),
        w!("Score", "Score zum Freischalten", tesla.score_required, 1.0, 0.0, 99999.0),
        w!("Lv2 Score", "Score fuer Level 2", tesla.score_level_2, 1.0, 0.0, 99999.0),
        w!("Lv3 Score", "Score fuer Level 3", tesla.score_level_3, 1.0, 0.0, 99999.0),
        Item::Value(Entry { label: "Lv2 Dmg", help: "Berechneter Schaden Lv2", get: |s| s.weapon_at_level(crate::components::WeaponType::Tesla, 2).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),
        Item::Value(Entry { label: "Lv3 Dmg", help: "Berechneter Schaden Lv3", get: |s| s.weapon_at_level(crate::components::WeaponType::Tesla, 3).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),

        // === Mine ===
        Item::Category("Mine"),
        w!("Cooldown", "Sekunden zwischen Platzierungen", mine.cooldown, 0.1, 0.2, 30.0),
        w!("Magazin", "Minen pro Magazin", mine.magazine, 1.0, 1.0, 200.0),
        w!("Damage", "Explosionsschaden", mine.damage, 5.0, 5.0, 5000.0),
        w!("Trigger-R", "Ausloeseradius (Zombie-Naehe)", mine.trigger_radius, 5.0, 10.0, 500.0),
        w!("Expl-Radius", "Explosionsradius", mine.explosion_radius_override, 5.0, 20.0, 1000.0),
        w!("Score", "Score zum Freischalten", mine.score_required, 1.0, 0.0, 99999.0),
        w!("Lv2 Score", "Score fuer Level 2", mine.score_level_2, 1.0, 0.0, 99999.0),
        w!("Lv3 Score", "Score fuer Level 3", mine.score_level_3, 1.0, 0.0, 99999.0),
        Item::Value(Entry { label: "Lv2 Dmg", help: "Berechneter Schaden Lv2", get: |s| s.weapon_at_level(crate::components::WeaponType::Mine, 2).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),
        Item::Value(Entry { label: "Lv3 Dmg", help: "Berechneter Schaden Lv3", get: |s| s.weapon_at_level(crate::components::WeaponType::Mine, 3).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),

        // === Boomerang ===
        Item::Category("Boomerang"),
        w!("Cooldown", "Sekunden zwischen Wuerfen", boomerang.cooldown, 0.1, 0.2, 30.0),
        w!("Magazin", "Wuerfe vor Reload", boomerang.magazine, 1.0, 1.0, 100.0),
        w!("Damage", "Schaden pro Treffer", boomerang.damage, 1.0, 1.0, 1000.0),
        w!("Range", "Maximale Flugdistanz", boomerang.range, 25.0, 50.0, 5000.0),
        w!("Proj-Speed", "Fluggeschwindigkeit", boomerang.bullet_speed, 25.0, 100.0, 3000.0),
        w!("Score", "Score zum Freischalten", boomerang.score_required, 1.0, 0.0, 99999.0),
        w!("Lv2 Score", "Score fuer Level 2", boomerang.score_level_2, 1.0, 0.0, 99999.0),
        w!("Lv3 Score", "Score fuer Level 3", boomerang.score_level_3, 1.0, 0.0, 99999.0),
        Item::Value(Entry { label: "Lv2 Dmg", help: "Berechneter Schaden Lv2", get: |s| s.weapon_at_level(crate::components::WeaponType::Boomerang, 2).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),
        Item::Value(Entry { label: "Lv3 Dmg", help: "Berechneter Schaden Lv3", get: |s| s.weapon_at_level(crate::components::WeaponType::Boomerang, 3).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),

        // === Rakete ===
        Item::Category("Rakete"),
        w!("Cooldown", "Sekunden zwischen Schuessen", rocket.cooldown, 0.1, 0.3, 30.0),
        w!("Magazin", "Raketen pro Magazin", rocket.magazine, 1.0, 1.0, 100.0),
        w!("Damage", "Explosionsschaden", rocket.damage, 5.0, 5.0, 5000.0),
        w!("Range", "Maximale Flugdistanz", rocket.range, 25.0, 50.0, 5000.0),
        w!("Proj-Speed", "Raketengeschwindigkeit", rocket.bullet_speed, 25.0, 50.0, 5000.0),
        w!("Expl-Radius", "Explosionsradius", rocket.explosion_radius_override, 5.0, 20.0, 1000.0),
        w!("Score", "Score zum Freischalten", rocket.score_required, 1.0, 0.0, 99999.0),
        w!("Lv2 Score", "Score fuer Level 2", rocket.score_level_2, 1.0, 0.0, 99999.0),
        w!("Lv3 Score", "Score fuer Level 3", rocket.score_level_3, 1.0, 0.0, 99999.0),
        Item::Value(Entry { label: "Lv2 Dmg", help: "Berechneter Schaden Lv2", get: |s| s.weapon_at_level(crate::components::WeaponType::Rocket, 2).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),
        Item::Value(Entry { label: "Lv3 Dmg", help: "Berechneter Schaden Lv3", get: |s| s.weapon_at_level(crate::components::WeaponType::Rocket, 3).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),

        // === Laser ===
        Item::Category("Laser"),
        w!("Cooldown", "Sekunden zwischen Schuessen", laser.cooldown, 0.01, 0.01, 5.0),
        w!("Magazin", "Schuss pro Batterie", laser.magazine, 5.0, 10.0, 2000.0),
        w!("Damage", "Schaden pro Treffer", laser.damage, 0.5, 0.5, 500.0),
        w!("Range", "Maximale Reichweite", laser.range, 50.0, 100.0, 10000.0),
        w!("Proj-Speed", "Lasergeschwindigkeit", laser.bullet_speed, 100.0, 500.0, 10000.0),
        w!("Pierce", "Gegner die durchdrungen werden", laser.pierce_count, 1.0, 1.0, 999.0),
        w!("Score", "Score zum Freischalten", laser.score_required, 1.0, 0.0, 99999.0),
        w!("Lv2 Score", "Score fuer Level 2", laser.score_level_2, 1.0, 0.0, 99999.0),
        w!("Lv3 Score", "Score fuer Level 3", laser.score_level_3, 1.0, 0.0, 99999.0),
        Item::Value(Entry { label: "Lv2 Dmg", help: "Berechneter Schaden Lv2", get: |s| s.weapon_at_level(crate::components::WeaponType::Laser, 2).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),
        Item::Value(Entry { label: "Lv3 Dmg", help: "Berechneter Schaden Lv3", get: |s| s.weapon_at_level(crate::components::WeaponType::Laser, 3).damage, set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),
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
            .filter(|i| matches!(items[*i], Item::Value(_)))
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
                        ..default()
                    },
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
        if let Item::Value(entry) = &items[item_idx] {
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
) {
    ui_state.repeat_timer.tick(time.delta());
    let items = all_items();
    let visible = visible_indices(&items, ui_state.open_category);
    let count = visible.len();
    if count == 0 { return; }

    // Navigation (ArrowUp/W, ArrowDown/S)
    if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
        ui_state.selected = (ui_state.selected + count - 1) % count;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        ui_state.selected = (ui_state.selected + 1) % count;
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
        *settings = GameSettings::default();
        settings.show_debug = show;
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
    list_container: Query<Entity, With<SettingsListContainer>>,
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
        if let Ok(container) = list_container.single() {
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
                            ..default()
                        },
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
    save_query: Query<&Interaction, (Changed<Interaction>, With<SaveButton>)>,
    defaults_query: Query<&Interaction, (Changed<Interaction>, With<DefaultsButton>)>,
    resume_query: Query<&Interaction, (Changed<Interaction>, With<ResumeButton>)>,
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
    for interaction in save_query.iter() {
        if *interaction == Interaction::Pressed {
            settings.save();
            ui_state.feedback_message = "Gespeichert!".into();
            ui_state.feedback_timer = 2.0;
        }
    }
    for interaction in defaults_query.iter() {
        if *interaction == Interaction::Pressed {
            let show = settings.show_debug;
            *settings = GameSettings::default();
            settings.show_debug = show;
            ui_state.feedback_message = "Defaults geladen!".into();
            ui_state.feedback_timer = 2.0;
            ui_state.needs_rebuild = true;
        }
    }
    for interaction in resume_query.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Playing);
        }
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
