use bevy::prelude::*;

use crate::resources::*;

#[derive(Component)]
pub struct SettingsPanel;

#[derive(Resource)]
pub struct SettingsUiState {
    pub selected: usize,
    pub repeat_timer: Timer,
}

impl Default for SettingsUiState {
    fn default() -> Self {
        Self {
            selected: 0,
            repeat_timer: Timer::from_seconds(0.08, TimerMode::Once),
        }
    }
}

struct Entry {
    label: &'static str,
    get: fn(&GameSettings) -> f32,
    set: fn(&mut GameSettings, f32),
    step: f32,
    min: f32,
    max: f32,
}

macro_rules! w {
    ($label:expr, $field:ident . $prop:ident, $step:expr, $min:expr, $max:expr) => {
        Entry { label: $label, get: |s| s.$field.$prop as f32, set: |s,v| s.$field.$prop = v as _, step: $step, min: $min, max: $max }
    };
}

fn entries() -> Vec<Entry> {
    vec![
        Entry { label: "Player Speed",     get: |s| s.player_speed,           set: |s,v| s.player_speed = v,           step: 10.0,  min: 50.0,  max: 500.0 },
        Entry { label: "Player HP",        get: |s| s.player_hp,              set: |s,v| s.player_hp = v,              step: 10.0,  min: 10.0,  max: 500.0 },
        Entry { label: "Zombie Speed",     get: |s| s.zombie_speed,           set: |s,v| s.zombie_speed = v,           step: 5.0,   min: 10.0,  max: 300.0 },
        Entry { label: "Zombie HP",        get: |s| s.zombie_hp,              set: |s,v| s.zombie_hp = v,              step: 5.0,   min: 5.0,   max: 200.0 },
        Entry { label: "Zombie Damage",    get: |s| s.zombie_damage,          set: |s,v| s.zombie_damage = v,          step: 1.0,   min: 1.0,   max: 50.0 },
        Entry { label: "Zombie Dmg CD",    get: |s| s.zombie_damage_cooldown, set: |s,v| s.zombie_damage_cooldown = v, step: 0.1,   min: 0.1,   max: 3.0 },
        Entry { label: "Combo Drain",      get: |s| s.combo_drain_speed,      set: |s,v| s.combo_drain_speed = v,      step: 0.01,  min: 0.01,  max: 0.5 },
        Entry { label: "Combo Kill Boost", get: |s| s.combo_kill_boost,       set: |s,v| s.combo_kill_boost = v,       step: 0.05,  min: 0.05,  max: 2.0 },
        Entry { label: "Wave Base",        get: |s| s.wave_base_zombies as f32, set: |s,v| s.wave_base_zombies = v as u32, step: 1.0, min: 1.0, max: 50.0 },
        Entry { label: "Wave Increment",   get: |s| s.wave_zombie_increment as f32, set: |s,v| s.wave_zombie_increment = v as u32, step: 1.0, min: 0.0, max: 20.0 },
        Entry { label: "Spawn Interval",   get: |s| s.spawn_interval,         set: |s,v| s.spawn_interval = v,         step: 0.1,   min: 0.1,   max: 5.0 },
        Entry { label: "Wave Pause",       get: |s| s.wave_pause,             set: |s,v| s.wave_pause = v,             step: 0.5,   min: 0.5,   max: 10.0 },
        Entry { label: "Explosion Radius", get: |s| s.explosion_radius,       set: |s,v| s.explosion_radius = v,       step: 5.0,   min: 20.0,  max: 200.0 },
        // Pistole
        w!("Pistol CD",     pistol.cooldown,     0.05, 0.02, 2.0),
        w!("Pistol Mag",    pistol.magazine,      1.0,  1.0, 50.0),
        w!("Pistol Dmg",    pistol.damage,        1.0,  1.0, 200.0),
        w!("Pistol Range",  pistol.range,        25.0, 50.0, 1000.0),
        w!("Pistol Score",  pistol.score_required,1.0,  0.0, 50.0),
        // Shotgun
        w!("Shotgun CD",    shotgun.cooldown,    0.1,  0.1, 3.0),
        w!("Shotgun Mag",   shotgun.magazine,     1.0,  1.0, 20.0),
        w!("Shotgun Dmg",   shotgun.damage,       1.0,  1.0, 50.0),
        w!("Shotgun Pellets",shotgun.pellet_count,1.0,  1.0, 20.0),
        w!("Shotgun Spread",shotgun.spread_angle, 0.05, 0.1, 1.0),
        w!("Shotgun Score", shotgun.score_required,1.0, 0.0, 50.0),
        // Uzi
        w!("Uzi CD",       uzi.cooldown,        0.01, 0.02, 0.5),
        w!("Uzi Mag",      uzi.magazine,         5.0,  5.0, 100.0),
        w!("Uzi Dmg",      uzi.damage,           1.0,  1.0, 50.0),
        w!("Uzi Score",    uzi.score_required,    1.0,  0.0, 50.0),
        // Flammenwerfer
        w!("Flame CD",     flamethrower.cooldown,    0.01, 0.01, 0.5),
        w!("Flame Mag",    flamethrower.magazine,     5.0, 10.0, 200.0),
        w!("Flame Dmg",    flamethrower.damage,       0.5,  0.5, 20.0),
        w!("Flame Range",  flamethrower.range,       10.0, 50.0, 300.0),
        w!("Flame Score",  flamethrower.score_required,1.0, 0.0, 50.0),
        // Granate
        w!("Grenade CD",   grenade.cooldown,      0.1,  0.2, 5.0),
        w!("Grenade Mag",  grenade.magazine,       1.0,  1.0, 20.0),
        w!("Grenade Dmg",  grenade.damage,         5.0,  5.0, 300.0),
        w!("Grenade Score",grenade.score_required, 1.0,  0.0, 50.0),
        // Railgun
        w!("Rail CD",      railgun.cooldown,      0.1,  0.1, 5.0),
        w!("Rail Mag",     railgun.magazine,       1.0,  1.0, 20.0),
        w!("Rail Dmg",     railgun.damage,         5.0,  5.0, 500.0),
        w!("Rail Range",   railgun.range,         50.0,100.0,2000.0),
        w!("Rail Score",   railgun.score_required, 1.0,  0.0, 50.0),
        // Freeze Gun
        w!("Freeze CD",    freezegun.cooldown,    0.05, 0.05, 2.0),
        w!("Freeze Mag",   freezegun.magazine,     1.0,  1.0, 50.0),
        w!("Freeze Dmg",   freezegun.damage,       0.5,  0.5, 20.0),
        w!("Freeze Slow",  freezegun.slow_factor,  0.05, 0.05, 0.9),
        w!("Freeze Dur",   freezegun.slow_duration, 0.5,  0.5, 10.0),
        w!("Freeze Score", freezegun.score_required,1.0,  0.0, 50.0),
        // Kreissaege
        w!("Saw CD",       buzzsaw.cooldown,      0.1,  0.2, 5.0),
        w!("Saw Mag",      buzzsaw.magazine,       1.0,  1.0, 20.0),
        w!("Saw Dmg",      buzzsaw.damage,         1.0,  1.0, 50.0),
        w!("Saw Speed",    buzzsaw.bullet_speed,  10.0, 30.0, 300.0),
        w!("Saw Range",    buzzsaw.range,         50.0, 100.0,1000.0),
        w!("Saw Score",    buzzsaw.score_required, 1.0,  0.0, 50.0),
        // Tesla
        w!("Tesla CD",     tesla.cooldown,        0.05, 0.1, 3.0),
        w!("Tesla Mag",    tesla.magazine,         1.0,  1.0, 30.0),
        w!("Tesla Dmg",    tesla.damage,           1.0,  1.0, 100.0),
        w!("Tesla Chains", tesla.chain_count,      1.0,  0.0, 10.0),
        w!("Tesla ChainR", tesla.chain_range,     10.0, 20.0, 200.0),
        w!("Tesla Score",  tesla.score_required,   1.0,  0.0, 50.0),
        // Mine
        w!("Mine CD",      mine.cooldown,         0.1,  0.2, 5.0),
        w!("Mine Mag",     mine.magazine,          1.0,  1.0, 20.0),
        w!("Mine Dmg",     mine.damage,            5.0,  5.0, 300.0),
        w!("Mine Trigger",  mine.trigger_radius,   5.0, 10.0, 100.0),
        w!("Mine ExplR",   mine.explosion_radius_override,5.0,20.0,200.0),
        w!("Mine Score",   mine.score_required,    1.0,  0.0, 50.0),
        // Boomerang
        w!("Boom CD",      boomerang.cooldown,    0.1,  0.2, 5.0),
        w!("Boom Mag",     boomerang.magazine,     1.0,  1.0, 10.0),
        w!("Boom Dmg",     boomerang.damage,       1.0,  1.0, 100.0),
        w!("Boom Range",   boomerang.range,       25.0, 50.0, 500.0),
        w!("Boom Speed",   boomerang.bullet_speed,25.0,100.0, 600.0),
        w!("Boom Score",   boomerang.score_required,1.0, 0.0, 50.0),
        // Rakete
        w!("Rocket CD",    rocket.cooldown,       0.1,  0.3, 5.0),
        w!("Rocket Mag",   rocket.magazine,        1.0,  1.0, 10.0),
        w!("Rocket Dmg",   rocket.damage,          5.0,  5.0, 500.0),
        w!("Rocket ExplR", rocket.explosion_radius_override,5.0,20.0,300.0),
        w!("Rocket Score", rocket.score_required,  1.0,  0.0, 50.0),
        // Laser
        w!("Laser CD",     laser.cooldown,        0.01, 0.01, 0.5),
        w!("Laser Mag",    laser.magazine,         5.0, 10.0, 200.0),
        w!("Laser Dmg",    laser.damage,           0.5,  0.5, 50.0),
        w!("Laser Range",  laser.range,           50.0,100.0,1000.0),
        w!("Laser Speed",  laser.bullet_speed,   100.0,500.0,3000.0),
        w!("Laser Score",  laser.score_required,   1.0,  0.0, 50.0),
    ]
}

pub fn settings_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut ui_state: ResMut<SettingsUiState>,
    mut settings: ResMut<GameSettings>,
) {
    ui_state.repeat_timer.tick(time.delta());
    let all = entries();
    let count = all.len();

    // Navigation
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        ui_state.selected = (ui_state.selected + count - 1) % count;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        ui_state.selected = (ui_state.selected + 1) % count;
    }

    // Werte aendern
    let can_change = keyboard.just_pressed(KeyCode::ArrowLeft)
        || keyboard.just_pressed(KeyCode::ArrowRight)
        || ((keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::ArrowRight))
            && ui_state.repeat_timer.finished());

    if can_change {
        ui_state.repeat_timer.reset();
        let entry = &all[ui_state.selected];
        let val = (entry.get)(&settings);
        if keyboard.pressed(KeyCode::ArrowRight) {
            (entry.set)(&mut settings, (val + entry.step).min(entry.max));
        }
        if keyboard.pressed(KeyCode::ArrowLeft) {
            (entry.set)(&mut settings, (val - entry.step).max(entry.min));
        }
    }

    // Speichern
    if keyboard.just_pressed(KeyCode::F5) {
        settings.save();
    }

    // Defaults wiederherstellen
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
    // Altes Panel entfernen
    for entity in panel_query.iter() {
        commands.entity(entity).despawn();
    }

    let all = entries();
    let visible_range = 20;
    let half = visible_range / 2;
    let visible_start = if ui_state.selected > half {
        (ui_state.selected - half).min(all.len().saturating_sub(visible_range))
    } else {
        0
    };
    let visible_end = (visible_start + visible_range).min(all.len());

    let mut lines = vec![
        "=== KLOTZKOEPFE - PAUSE ===".to_string(),
        String::new(),
        "--- Steuerung ---".to_string(),
        "P1: WASD + Space (schiessen) + Q (Waffe wechseln)".to_string(),
        "P2: Pfeiltasten + Enter (schiessen) + RShift (Waffe wechseln)".to_string(),
        String::new(),
        "--- Pause-Menue ---".to_string(),
        "Up/Down: Setting waehlen | Left/Right: Wert aendern".to_string(),
        "F5: Settings speichern | F6: Defaults | ESC: weiter".to_string(),
        "---".to_string(),
    ];

    for i in visible_start..visible_end {
        let entry = &all[i];
        let val = (entry.get)(&settings);
        let marker = if i == ui_state.selected { ">> " } else { "   " };
        lines.push(format!("{}{}: {:.2}", marker, entry.label, val));
    }

    if visible_end < all.len() {
        lines.push(format!("   ... ({} more)", all.len() - visible_end));
    }

    let text = lines.join("\n");

    commands.spawn((
        Text2d::new(text),
        TextFont { font_size: 14.0, ..default() },
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

