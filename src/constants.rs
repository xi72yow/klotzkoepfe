use bevy::prelude::*;

// Fenster
pub const WINDOW_WIDTH: f32 = 1024.0;
pub const WINDOW_HEIGHT: f32 = 768.0;

// Spieler (visuals)
pub const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 40.0);
pub const PLAYER_COLOR_P1: Color = Color::srgb(0.2, 0.8, 0.2);
pub const PLAYER_COLOR_P2: Color = Color::srgb(0.2, 0.4, 0.9);

// Zombie (visuals)
pub const ZOMBIE_SIZE: Vec2 = Vec2::new(28.0, 36.0);
pub const ZOMBIE_COLOR: Color = Color::srgb(0.8, 0.15, 0.15);

// Blut
pub const BLOOD_PARTICLE_SIZE: Vec2 = Vec2::new(4.0, 4.0);
pub const BLOOD_PARTICLES_PER_HIT: u32 = 8;
pub const BLOOD_SPREAD_SPEED: f32 = 150.0;
pub const BLOOD_LIFETIME: f32 = 0.3;
pub const BLOOD_COLOR_MIN: Color = Color::srgb(0.4, 0.0, 0.0);
pub const BLOOD_COLOR_MAX: Color = Color::srgb(0.7, 0.05, 0.05);

// Raum
pub const WALL_THICKNESS: f32 = 20.0;
pub const WALL_COLOR: Color = Color::srgb(0.4, 0.4, 0.4);
pub const FLOOR_COLOR: Color = Color::srgb(0.25, 0.25, 0.3);

// HP-Balken ueber Spieler
pub const HP_BAR_WIDTH: f32 = 30.0;
pub const HP_BAR_HEIGHT: f32 = 4.0;
pub const HP_BAR_OFFSET_Y: f32 = 28.0;

// Combo-Meter (visuals)
pub const COMBO_TRACK_WIDTH: f32 = 200.0;
pub const COMBO_TRACK_HEIGHT: f32 = 12.0;
pub const COMBO_BLOCK_SIZE: f32 = 16.0;

// Explosion (visuals)
pub const EXPLOSION_LIFETIME: f32 = 0.3;
pub const EXPLOSION_COLOR: Color = Color::srgb(1.0, 0.5, 0.0);

// WaveState defaults
pub const SPAWN_INTERVAL: f32 = 0.8;
pub const WAVE_PAUSE: f32 = 2.0;
