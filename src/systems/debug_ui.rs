use bevy::prelude::*;

use crate::resources::*;

#[derive(Component)]
pub struct SettingsPanel;

#[derive(Resource)]
pub struct SettingsUiState {
    pub selected: usize,
    pub repeat_timer: Timer,
    pub open_category: usize,
}

impl Default for SettingsUiState {
    fn default() -> Self {
        Self {
            selected: 0,
            repeat_timer: Timer::from_seconds(0.08, TimerMode::Once),
            open_category: 0,
        }
    }
}

// --- Datenmodell ---

enum Item {
    Category(&'static str),
    Value(Entry),
}

struct Entry {
    label: &'static str,
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
    ($label:expr, $field:ident . $prop:ident, $step:expr, $min:expr, $max:expr) => {
        Item::Value(Entry { label: $label, get: |s| s.$field.$prop as f32, set: |s,v| s.$field.$prop = v as _, step: $step, min: $min, max: $max, display: DisplayMode::Float })
    };
}

fn all_items() -> Vec<Item> {
    vec![
        // === Spieler ===
        Item::Category("Spieler"),
        Item::Value(Entry { label: "Speed",          get: |s| s.player_speed,    set: |s,v| s.player_speed = v,    step: 10.0, min: 50.0,  max: 2000.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "HP",             get: |s| s.player_hp,       set: |s,v| s.player_hp = v,       step: 10.0, min: 10.0,  max: 5000.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Friendly Fire",  get: |s| if s.friendly_fire { 1.0 } else { 0.0 }, set: |s,v| s.friendly_fire = v >= 0.5, step: 1.0, min: 0.0, max: 1.0, display: DisplayMode::Bool }),

        // === Zombies ===
        Item::Category("Zombies"),
        Item::Value(Entry { label: "Speed",     get: |s| s.zombie_speed,           set: |s,v| s.zombie_speed = v,           step: 5.0,  min: 10.0, max: 1000.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "HP",        get: |s| s.zombie_hp,              set: |s,v| s.zombie_hp = v,              step: 5.0,  min: 5.0,  max: 2000.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Damage",    get: |s| s.zombie_damage,          set: |s,v| s.zombie_damage = v,          step: 1.0,  min: 1.0,  max: 500.0,  display: DisplayMode::Float }),
        Item::Value(Entry { label: "Dmg CD",    get: |s| s.zombie_damage_cooldown, set: |s,v| s.zombie_damage_cooldown = v, step: 0.1,  min: 0.1,  max: 10.0,   display: DisplayMode::Float }),

        // === Wellen ===
        Item::Category("Wellen"),
        Item::Value(Entry { label: "Basis-Anzahl",  get: |s| s.wave_base_zombies as f32,      set: |s,v| s.wave_base_zombies = v as u32,      step: 1.0, min: 1.0, max: 500.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Inkrement",     get: |s| s.wave_zombie_increment as f32,   set: |s,v| s.wave_zombie_increment = v as u32,   step: 1.0, min: 0.0, max: 100.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Spawn-Intervall", get: |s| s.spawn_interval,                set: |s,v| s.spawn_interval = v,                step: 0.1, min: 0.1, max: 30.0,  display: DisplayMode::Float }),
        Item::Value(Entry { label: "Wellen-Pause",  get: |s| s.wave_pause,                     set: |s,v| s.wave_pause = v,                    step: 0.5, min: 0.5, max: 60.0,  display: DisplayMode::Float }),

        // === Combo ===
        Item::Category("Combo"),
        Item::Value(Entry { label: "Drain-Speed",   get: |s| s.combo_drain_speed,  set: |s,v| s.combo_drain_speed = v,  step: 0.01, min: 0.01, max: 5.0,  display: DisplayMode::Float }),
        Item::Value(Entry { label: "Kill-Boost",    get: |s| s.combo_kill_boost,   set: |s,v| s.combo_kill_boost = v,   step: 0.05, min: 0.05, max: 20.0, display: DisplayMode::Float }),

        // === Gore ===
        Item::Category("Gore"),
        Item::Value(Entry { label: "Blut-Partikel",  get: |s| s.blood_particles as f32, set: |s,v| s.blood_particles = v as u32, step: 1.0,  min: 0.0, max: 30.0,  display: DisplayMode::Float }),
        Item::Value(Entry { label: "Blut-Spread",    get: |s| s.blood_spread_speed,     set: |s,v| s.blood_spread_speed = v,     step: 10.0, min: 0.0, max: 500.0, display: DisplayMode::Float }),
        Item::Value(Entry { label: "Dismember",      get: |s| s.dismember_chance * 100.0, set: |s,v| s.dismember_chance = v / 100.0, step: 5.0, min: 0.0, max: 100.0, display: DisplayMode::Percent }),
        Item::Value(Entry { label: "Gib-Verfall",    get: |s| s.gib_decay_time,         set: |s,v| s.gib_decay_time = v,         step: 0.5,  min: 0.5, max: 30.0,  display: DisplayMode::Float }),
        Item::Value(Entry { label: "Explosion-Radius", get: |s| s.explosion_radius,     set: |s,v| s.explosion_radius = v,       step: 5.0,  min: 20.0, max: 1000.0, display: DisplayMode::Float }),

        // === Pistole ===
        Item::Category("Pistole"),
        w!("Cooldown",    pistol.cooldown,        0.05, 0.02, 10.0),
        w!("Magazin",     pistol.magazine,         1.0,  1.0, 500.0),
        w!("Damage",      pistol.damage,           1.0,  1.0, 2000.0),
        w!("Range",       pistol.range,           25.0, 50.0, 5000.0),
        w!("Proj-Speed",  pistol.bullet_speed,    25.0, 50.0, 5000.0),
        w!("Score",       pistol.score_required,   1.0,  0.0, 500.0),
        w!("Lv2 Score",   pistol.score_level_2,    1.0,  0.0, 500.0),
        w!("Lv3 Score",   pistol.score_level_3,    1.0,  0.0, 500.0),

        // === Shotgun ===
        Item::Category("Shotgun"),
        w!("Cooldown",    shotgun.cooldown,       0.1,  0.1, 10.0),
        w!("Magazin",     shotgun.magazine,        1.0,  1.0, 200.0),
        w!("Damage",      shotgun.damage,          1.0,  1.0, 500.0),
        w!("Pellets",     shotgun.pellet_count,    1.0,  1.0, 100.0),
        w!("Spread",      shotgun.spread_angle,    0.05, 0.1, 3.14),
        w!("Proj-Speed",  shotgun.bullet_speed,   25.0, 50.0, 5000.0),
        w!("Score",       shotgun.score_required,  1.0,  0.0, 500.0),
        w!("Lv2 Score",   shotgun.score_level_2,   1.0,  0.0, 500.0),
        w!("Lv3 Score",   shotgun.score_level_3,   1.0,  0.0, 500.0),

        // === Uzi ===
        Item::Category("Uzi"),
        w!("Cooldown",    uzi.cooldown,           0.01, 0.02, 5.0),
        w!("Magazin",     uzi.magazine,            5.0,  5.0, 1000.0),
        w!("Damage",      uzi.damage,              1.0,  1.0, 500.0),
        w!("Proj-Speed",  uzi.bullet_speed,       25.0, 50.0, 5000.0),
        w!("Score",       uzi.score_required,      1.0,  0.0, 500.0),
        w!("Lv2 Score",   uzi.score_level_2,       1.0,  0.0, 500.0),
        w!("Lv3 Score",   uzi.score_level_3,       1.0,  0.0, 500.0),

        // === Flammenwerfer ===
        Item::Category("Flammenwerfer"),
        w!("Cooldown",    flamethrower.cooldown,       0.01, 0.01, 5.0),
        w!("Magazin",     flamethrower.magazine,        5.0, 10.0, 2000.0),
        w!("Damage",      flamethrower.damage,          0.5,  0.5, 200.0),
        w!("Range",       flamethrower.range,          10.0, 50.0, 2000.0),
        w!("Proj-Speed",  flamethrower.bullet_speed,   10.0, 50.0, 5000.0),
        w!("Score",       flamethrower.score_required,  1.0,  0.0, 500.0),
        w!("Lv2 Score",   flamethrower.score_level_2,   1.0,  0.0, 500.0),
        w!("Lv3 Score",   flamethrower.score_level_3,   1.0,  0.0, 500.0),

        // === Granate ===
        Item::Category("Granate"),
        w!("Cooldown",    grenade.cooldown,        0.1,  0.2, 30.0),
        w!("Magazin",     grenade.magazine,         1.0,  1.0, 200.0),
        w!("Damage",      grenade.damage,           5.0,  5.0, 5000.0),
        w!("Range",       grenade.range,           25.0, 50.0, 5000.0),
        w!("Proj-Speed",  grenade.bullet_speed,    25.0, 50.0, 5000.0),
        Item::Value(Entry { label: "Fuse (berechnet)", get: |s| s.grenade.range / s.grenade.bullet_speed.max(1.0), set: |_,_| {}, step: 0.0, min: 0.0, max: 999.0, display: DisplayMode::ReadOnly }),
        w!("Score",       grenade.score_required,   1.0,  0.0, 500.0),
        w!("Lv2 Score",   grenade.score_level_2,    1.0,  0.0, 500.0),
        w!("Lv3 Score",   grenade.score_level_3,    1.0,  0.0, 500.0),

        // === Railgun ===
        Item::Category("Railgun"),
        w!("Cooldown",    railgun.cooldown,        0.1,  0.1, 30.0),
        w!("Magazin",     railgun.magazine,         1.0,  1.0, 200.0),
        w!("Damage",      railgun.damage,           5.0,  5.0, 5000.0),
        w!("Range",       railgun.range,           50.0,100.0,10000.0),
        w!("Proj-Speed",  railgun.bullet_speed,    50.0,100.0,10000.0),
        w!("Score",       railgun.score_required,   1.0,  0.0, 500.0),
        w!("Lv2 Score",   railgun.score_level_2,    1.0,  0.0, 500.0),
        w!("Lv3 Score",   railgun.score_level_3,    1.0,  0.0, 500.0),

        // === Freeze Gun ===
        Item::Category("Freeze Gun"),
        w!("Cooldown",    freezegun.cooldown,       0.05, 0.05, 10.0),
        w!("Magazin",     freezegun.magazine,        1.0,  1.0, 500.0),
        w!("Damage",      freezegun.damage,          0.5,  0.5, 200.0),
        w!("Slow-Faktor", freezegun.slow_factor,     0.05, 0.05, 0.99),
        w!("Slow-Dauer",  freezegun.slow_duration,   0.5,  0.5, 60.0),
        w!("Proj-Speed",  freezegun.bullet_speed,   25.0, 50.0, 5000.0),
        w!("Score",       freezegun.score_required,  1.0,  0.0, 500.0),
        w!("Lv2 Score",   freezegun.score_level_2,   1.0,  0.0, 500.0),
        w!("Lv3 Score",   freezegun.score_level_3,   1.0,  0.0, 500.0),

        // === Kreissaege ===
        Item::Category("Kreissaege"),
        w!("Cooldown",    buzzsaw.cooldown,        0.1,  0.2, 30.0),
        w!("Magazin",     buzzsaw.magazine,         1.0,  1.0, 200.0),
        w!("Damage",      buzzsaw.damage,           1.0,  1.0, 500.0),
        w!("Proj-Speed",  buzzsaw.bullet_speed,    10.0, 30.0, 2000.0),
        w!("Range",       buzzsaw.range,           50.0,100.0, 5000.0),
        w!("Score",       buzzsaw.score_required,   1.0,  0.0, 500.0),
        w!("Lv2 Score",   buzzsaw.score_level_2,    1.0,  0.0, 500.0),
        w!("Lv3 Score",   buzzsaw.score_level_3,    1.0,  0.0, 500.0),

        // === Tesla ===
        Item::Category("Tesla"),
        w!("Cooldown",    tesla.cooldown,          0.05, 0.1, 10.0),
        w!("Magazin",     tesla.magazine,           1.0,  1.0, 300.0),
        w!("Damage",      tesla.damage,             1.0,  1.0, 1000.0),
        w!("Chains",      tesla.chain_count,        1.0,  0.0, 50.0),
        w!("Chain-Range",  tesla.chain_range,       10.0, 20.0, 2000.0),
        w!("Proj-Speed",  tesla.bullet_speed,      25.0, 50.0, 5000.0),
        w!("Score",       tesla.score_required,     1.0,  0.0, 500.0),
        w!("Lv2 Score",   tesla.score_level_2,      1.0,  0.0, 500.0),
        w!("Lv3 Score",   tesla.score_level_3,      1.0,  0.0, 500.0),

        // === Mine ===
        Item::Category("Mine"),
        w!("Cooldown",    mine.cooldown,           0.1,  0.2, 30.0),
        w!("Magazin",     mine.magazine,            1.0,  1.0, 200.0),
        w!("Damage",      mine.damage,              5.0,  5.0, 5000.0),
        w!("Trigger-R",   mine.trigger_radius,      5.0, 10.0, 500.0),
        w!("Expl-Radius", mine.explosion_radius_override, 5.0, 20.0, 1000.0),
        w!("Score",       mine.score_required,      1.0,  0.0, 500.0),
        w!("Lv2 Score",   mine.score_level_2,       1.0,  0.0, 500.0),
        w!("Lv3 Score",   mine.score_level_3,       1.0,  0.0, 500.0),

        // === Boomerang ===
        Item::Category("Boomerang"),
        w!("Cooldown",    boomerang.cooldown,      0.1,  0.2, 30.0),
        w!("Magazin",     boomerang.magazine,       1.0,  1.0, 100.0),
        w!("Damage",      boomerang.damage,         1.0,  1.0, 1000.0),
        w!("Range",       boomerang.range,         25.0, 50.0, 5000.0),
        w!("Proj-Speed",  boomerang.bullet_speed,  25.0,100.0, 3000.0),
        w!("Score",       boomerang.score_required, 1.0,  0.0, 500.0),
        w!("Lv2 Score",   boomerang.score_level_2,  1.0,  0.0, 500.0),
        w!("Lv3 Score",   boomerang.score_level_3,  1.0,  0.0, 500.0),

        // === Rakete ===
        Item::Category("Rakete"),
        w!("Cooldown",    rocket.cooldown,         0.1,  0.3, 30.0),
        w!("Magazin",     rocket.magazine,          1.0,  1.0, 100.0),
        w!("Damage",      rocket.damage,            5.0,  5.0, 5000.0),
        w!("Range",       rocket.range,            25.0, 50.0, 5000.0),
        w!("Proj-Speed",  rocket.bullet_speed,     25.0, 50.0, 5000.0),
        w!("Expl-Radius", rocket.explosion_radius_override, 5.0, 20.0, 1000.0),
        w!("Score",       rocket.score_required,    1.0,  0.0, 500.0),
        w!("Lv2 Score",   rocket.score_level_2,     1.0,  0.0, 500.0),
        w!("Lv3 Score",   rocket.score_level_3,     1.0,  0.0, 500.0),

        // === Laser ===
        Item::Category("Laser"),
        w!("Cooldown",    laser.cooldown,          0.01, 0.01, 5.0),
        w!("Magazin",     laser.magazine,           5.0, 10.0, 2000.0),
        w!("Damage",      laser.damage,             0.5,  0.5, 500.0),
        w!("Range",       laser.range,             50.0,100.0,10000.0),
        w!("Proj-Speed",  laser.bullet_speed,     100.0,500.0,10000.0),
        w!("Score",       laser.score_required,     1.0,  0.0, 500.0),
        w!("Lv2 Score",   laser.score_level_2,      1.0,  0.0, 500.0),
        w!("Lv3 Score",   laser.score_level_3,      1.0,  0.0, 500.0),
    ]
}

// --- Hilfsfunktionen ---

/// Gibt (Kategorie-Indices, flache Entry-Liste mit globalem Index) zurueck
fn build_categories(items: &[Item]) -> Vec<(usize, &'static str)> {
    items.iter().enumerate()
        .filter_map(|(i, item)| match item {
            Item::Category(name) => Some((i, *name)),
            _ => None,
        })
        .collect()
}

/// Gibt nur die Value-Indices der offenen Kategorie zurueck
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

// --- Systeme ---

pub fn settings_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut ui_state: ResMut<SettingsUiState>,
    mut settings: ResMut<GameSettings>,
) {
    ui_state.repeat_timer.tick(time.delta());
    let items = all_items();
    let visible = visible_indices(&items, ui_state.open_category);
    let count = visible.len();
    if count == 0 { return; }

    // Navigation
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        ui_state.selected = (ui_state.selected + count - 1) % count;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        ui_state.selected = (ui_state.selected + 1) % count;
    }

    // Tab: naechste Kategorie
    if keyboard.just_pressed(KeyCode::Tab) {
        let cats = build_categories(&items);
        if !cats.is_empty() {
            let dir = if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
                cats.len() - 1 // rueckwaerts
            } else {
                1
            };
            ui_state.open_category = (ui_state.open_category + dir) % cats.len();
            ui_state.selected = 0; // Auf Kategorie-Header springen
        }
    }

    // Werte aendern
    if count == 0 { return; }
    let selected_item_idx = visible[ui_state.selected];

    let can_change = keyboard.just_pressed(KeyCode::ArrowLeft)
        || keyboard.just_pressed(KeyCode::ArrowRight)
        || ((keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::ArrowRight))
            && ui_state.repeat_timer.is_finished());

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
                if keyboard.pressed(KeyCode::ArrowRight) {
                    (entry.set)(&mut settings, (val + step).min(entry.max));
                }
                if keyboard.pressed(KeyCode::ArrowLeft) {
                    (entry.set)(&mut settings, (val - step).max(entry.min));
                }
            }
        }
    }

    // F5: Speichern
    if keyboard.just_pressed(KeyCode::F5) {
        settings.save();
    }

    // F6: Defaults
    if keyboard.just_pressed(KeyCode::F6) {
        let show = settings.show_debug;
        *settings = GameSettings::default();
        settings.show_debug = show;
    }
}

pub fn settings_render(
    mut commands: Commands,
    settings: Res<GameSettings>,
    ui_state: Res<SettingsUiState>,
    panel_query: Query<Entity, With<SettingsPanel>>,
) {
    for entity in panel_query.iter() {
        commands.entity(entity).despawn();
    }

    let items = all_items();
    let cats = build_categories(&items);
    let visible = visible_indices(&items, ui_state.open_category);
    let open_cat_name = cats.get(ui_state.open_category).map(|(_, n)| *n).unwrap_or("");

    let mut lines: Vec<String> = Vec::new();

    // Header
    lines.push("=== KLOTZKOEPFE - PAUSE ===".into());
    lines.push("Up/Down: Wert | Shift: 10x | Tab: Kategorie".into());
    lines.push("F5: Speichern | F6: Defaults | ESC: Weiter".into());
    lines.push(String::new());

    // Kategorie-Navigator
    let cat_num = ui_state.open_category + 1;
    let cat_total = cats.len();
    lines.push(format!(
        "<< Tab  [{}/{}] {}  Tab >>",
        cat_num, cat_total, open_cat_name
    ));
    lines.push(String::new());

    // Scrolling: max 16 Eintraege sichtbar
    let max_visible = 16;
    let total = visible.len();
    let sel = ui_state.selected.min(total.saturating_sub(1));
    let half = max_visible / 2;
    let scroll_start = if sel > half {
        (sel - half).min(total.saturating_sub(max_visible))
    } else {
        0
    };
    let scroll_end = (scroll_start + max_visible).min(total);

    if scroll_start > 0 {
        lines.push(format!("   ... {} davor ...", scroll_start));
    }

    for i in scroll_start..scroll_end {
        let item_idx = visible[i];
        if let Item::Value(entry) = &items[item_idx] {
            let is_selected = i == sel;
            let marker = if is_selected { "> " } else { "  " };

            let val = (entry.get)(&settings);
            let val_str = match entry.display {
                DisplayMode::Bool => {
                    if val >= 0.5 { "ON".into() } else { "OFF".into() }
                }
                DisplayMode::Percent => format!("{:.0}%", val),
                DisplayMode::ReadOnly => format!("{:.2}s", val),
                DisplayMode::Float => format!("{:.2}", val),
            };

            lines.push(format!("{}{:<18} {}", marker, entry.label, val_str));
        }
    }

    if scroll_end < total {
        lines.push(format!("   ... {} weitere ...", total - scroll_end));
    }

    let text = lines.join("\n");

    commands.spawn((
        Text2d::new(text),
        TextFont { font_size: 13.0, ..default() },
        TextColor(Color::srgb(0.0, 1.0, 0.0)),
        Transform::from_xyz(0.0, 0.0, 50.0),
        SettingsPanel,
    ));
}

pub fn cleanup_settings_panel(
    mut commands: Commands,
    panel_query: Query<Entity, With<SettingsPanel>>,
) {
    for entity in panel_query.iter() {
        commands.entity(entity).despawn();
    }
}
