use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlayerId {
    P1,
    P2,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum WeaponType {
    Pistol,
    Uzi,
    Grenade,
    Railgun,
    Flamethrower,
    Shotgun,
    Laser,
    Mine,
    Boomerang,
    Tesla,
    Buzzsaw,
    Rocket,
    FreezeGun,
}

impl WeaponType {
    pub fn all() -> &'static [WeaponType] {
        &[
            WeaponType::Pistol,
            WeaponType::Shotgun,
            WeaponType::Uzi,
            WeaponType::Flamethrower,
            WeaponType::Grenade,
            WeaponType::Railgun,
            WeaponType::FreezeGun,
            WeaponType::Buzzsaw,
            WeaponType::Tesla,
            WeaponType::Mine,
            WeaponType::Boomerang,
            WeaponType::Rocket,
            WeaponType::Laser,
        ]
    }

    pub fn name(self) -> &'static str {
        self.name_at_level(1)
    }

    pub fn name_at_level(self, level: u32) -> &'static str {
        match (self, level) {
            (WeaponType::Pistol, 1) => "Pistole",
            (WeaponType::Pistol, 2) => "Revolver",
            (WeaponType::Pistol, _) => "Deagle",

            (WeaponType::Uzi, 1) => "Uzi",
            (WeaponType::Uzi, 2) => "MP5",
            (WeaponType::Uzi, _) => "P90",

            (WeaponType::Shotgun, 1) => "Shotgun",
            (WeaponType::Shotgun, 2) => "Pumpgun",
            (WeaponType::Shotgun, _) => "Auto-Shotgun",

            (WeaponType::Grenade, 1) => "Granate",
            (WeaponType::Grenade, 2) => "Splitter",
            (WeaponType::Grenade, _) => "Cluster",

            (WeaponType::Railgun, 1) => "Railgun",
            (WeaponType::Railgun, 2) => "Gaussgewehr",
            (WeaponType::Railgun, _) => "Partikelkanone",

            (WeaponType::Flamethrower, 1) => "Flammenwerfer",
            (WeaponType::Flamethrower, 2) => "Inferno",
            (WeaponType::Flamethrower, _) => "Hoellenfeuer",

            (WeaponType::Laser, 1) => "Laser",
            (WeaponType::Laser, 2) => "Phaser",
            (WeaponType::Laser, _) => "Todesstrahl",

            (WeaponType::Mine, 1) => "Mine",
            (WeaponType::Mine, 2) => "Sprengfalle",
            (WeaponType::Mine, _) => "Nuklearmine",

            (WeaponType::Boomerang, 1) => "Boomerang",
            (WeaponType::Boomerang, 2) => "Doppelrang",
            (WeaponType::Boomerang, _) => "Triplerang",

            (WeaponType::Tesla, 1) => "Tesla",
            (WeaponType::Tesla, 2) => "Blitzgewitter",
            (WeaponType::Tesla, _) => "Donnergott",

            (WeaponType::Buzzsaw, 1) => "Kreissaege",
            (WeaponType::Buzzsaw, 2) => "Todessaege",
            (WeaponType::Buzzsaw, _) => "Phantomsaege",

            (WeaponType::Rocket, 1) => "Rakete",
            (WeaponType::Rocket, 2) => "Panzerfaust",
            (WeaponType::Rocket, _) => "ICBM",

            (WeaponType::FreezeGun, 1) => "Freeze Gun",
            (WeaponType::FreezeGun, 2) => "Frostknarre",
            (WeaponType::FreezeGun, _) => "Eissturm",
        }
    }

    pub fn bullet_size(self) -> Vec2 {
        match self {
            WeaponType::Pistol => Vec2::new(5.0, 1.5),
            WeaponType::Uzi => Vec2::new(4.0, 1.5),
            WeaponType::Grenade => Vec2::new(6.0, 6.0),
            WeaponType::Railgun => Vec2::new(16.0, 1.5),
            WeaponType::Flamethrower => Vec2::new(4.0, 4.0),
            WeaponType::Shotgun => Vec2::new(3.0, 1.5),
            WeaponType::Laser => Vec2::new(24.0, 1.5),
            WeaponType::Mine => Vec2::new(10.0, 10.0),
            WeaponType::Boomerang => Vec2::new(10.0, 10.0),
            WeaponType::Tesla => Vec2::new(6.0, 2.0),
            WeaponType::Buzzsaw => Vec2::new(14.0, 14.0),
            WeaponType::Rocket => Vec2::new(8.0, 3.0),
            WeaponType::FreezeGun => Vec2::new(5.0, 2.0),
        }
    }

    pub fn bullet_color(self) -> Color {
        match self {
            WeaponType::Pistol => Color::srgb(1.0, 0.9, 0.2),
            WeaponType::Uzi => Color::srgb(1.0, 0.7, 0.1),
            WeaponType::Grenade => Color::srgb(0.3, 0.5, 0.2),
            WeaponType::Railgun => Color::srgb(0.3, 0.8, 1.0),
            WeaponType::Flamethrower => Color::srgb(1.0, 0.4, 0.0),
            WeaponType::Shotgun => Color::srgb(0.9, 0.7, 0.3),
            WeaponType::Laser => Color::srgb(1.0, 0.1, 0.1),
            WeaponType::Mine => Color::srgb(0.6, 0.6, 0.1),
            WeaponType::Boomerang => Color::srgb(0.8, 0.4, 0.0),
            WeaponType::Tesla => Color::srgb(0.5, 0.5, 1.0),
            WeaponType::Buzzsaw => Color::srgb(0.7, 0.7, 0.7),
            WeaponType::Rocket => Color::srgb(0.8, 0.3, 0.1),
            WeaponType::FreezeGun => Color::srgb(0.4, 0.9, 1.0),
        }
    }

    /// Muzzle-Flash-Farben (inner, outer) - None = kein Flash
    pub fn muzzle_flash_colors(self) -> Option<(LinearRgba, LinearRgba)> {
        match self {
            WeaponType::Pistol => Some((
                LinearRgba::new(1.0, 0.95, 0.7, 1.0),
                LinearRgba::new(1.0, 0.6, 0.1, 0.8),
            )),
            WeaponType::Uzi => Some((
                LinearRgba::new(1.0, 0.9, 0.5, 1.0),
                LinearRgba::new(1.0, 0.5, 0.05, 0.7),
            )),
            WeaponType::Shotgun => Some((
                LinearRgba::new(1.0, 1.0, 0.8, 1.0),
                LinearRgba::new(1.0, 0.5, 0.1, 0.9),
            )),
            WeaponType::Railgun => Some((
                LinearRgba::new(0.7, 0.95, 1.0, 1.0),
                LinearRgba::new(0.2, 0.6, 1.0, 0.8),
            )),
            WeaponType::Laser => Some((
                LinearRgba::new(1.0, 0.5, 0.5, 1.0),
                LinearRgba::new(1.0, 0.1, 0.05, 0.7),
            )),
            WeaponType::Rocket => Some((
                LinearRgba::new(1.0, 0.9, 0.6, 1.0),
                LinearRgba::new(1.0, 0.4, 0.05, 0.9),
            )),
            // Flamethrower: kein Muzzle Flash, hat Cone-Beam
            WeaponType::Flamethrower => None,
            WeaponType::Tesla => Some((
                LinearRgba::new(0.8, 0.8, 1.0, 1.0),
                LinearRgba::new(0.3, 0.3, 1.0, 0.7),
            )),
            // FreezeGun: kein Muzzle Flash, hat Cone-Beam
            WeaponType::FreezeGun => None,
            WeaponType::Buzzsaw => Some((
                LinearRgba::new(1.0, 0.9, 0.6, 1.0),
                LinearRgba::new(0.8, 0.6, 0.2, 0.6),
            )),
            // Kein Flash fuer Granate, Mine, Boomerang
            WeaponType::Grenade | WeaponType::Mine | WeaponType::Boomerang => None,
        }
    }

    /// Muzzle-Flash-Groesse (Laenge des Kegels)
    pub fn muzzle_flash_size(self) -> f32 {
        match self {
            WeaponType::Pistol => 8.0,
            WeaponType::Uzi => 6.0,
            WeaponType::Shotgun => 14.0,
            WeaponType::Railgun => 12.0,
            WeaponType::Laser => 8.0,
            WeaponType::Rocket => 16.0,
            WeaponType::Flamethrower => 8.0,
            WeaponType::Tesla => 10.0,
            WeaponType::FreezeGun => 8.0,
            WeaponType::Buzzsaw => 6.0,
            _ => 8.0,
        }
    }

    pub fn piercing(self) -> u32 {
        match self {
            WeaponType::Railgun | WeaponType::Laser => 999,
            WeaponType::Buzzsaw => 999,
            _ => 1,
        }
    }

    pub fn sprite_size(self) -> Vec2 {
        match self {
            WeaponType::Pistol => Vec2::new(12.0, 4.0),
            WeaponType::Uzi => Vec2::new(16.0, 5.0),
            WeaponType::Grenade => Vec2::new(8.0, 8.0),
            WeaponType::Railgun => Vec2::new(24.0, 3.0),
            WeaponType::Flamethrower => Vec2::new(14.0, 6.0),
            WeaponType::Shotgun => Vec2::new(14.0, 6.0),
            WeaponType::Laser => Vec2::new(22.0, 3.0),
            WeaponType::Mine => Vec2::new(8.0, 8.0),
            WeaponType::Boomerang => Vec2::new(10.0, 4.0),
            WeaponType::Tesla => Vec2::new(14.0, 5.0),
            WeaponType::Buzzsaw => Vec2::new(10.0, 10.0),
            WeaponType::Rocket => Vec2::new(18.0, 6.0),
            WeaponType::FreezeGun => Vec2::new(16.0, 5.0),
        }
    }

    pub fn sprite_color(self) -> Color {
        match self {
            WeaponType::Pistol => Color::srgb(0.5, 0.5, 0.5),
            WeaponType::Uzi => Color::srgb(0.4, 0.4, 0.4),
            WeaponType::Grenade => Color::srgb(0.3, 0.4, 0.2),
            WeaponType::Railgun => Color::srgb(0.2, 0.5, 0.6),
            WeaponType::Flamethrower => Color::srgb(0.5, 0.3, 0.1),
            WeaponType::Shotgun => Color::srgb(0.5, 0.35, 0.2),
            WeaponType::Laser => Color::srgb(0.6, 0.2, 0.2),
            WeaponType::Mine => Color::srgb(0.4, 0.4, 0.1),
            WeaponType::Boomerang => Color::srgb(0.5, 0.3, 0.1),
            WeaponType::Tesla => Color::srgb(0.3, 0.3, 0.6),
            WeaponType::Buzzsaw => Color::srgb(0.5, 0.5, 0.5),
            WeaponType::Rocket => Color::srgb(0.4, 0.25, 0.1),
            WeaponType::FreezeGun => Color::srgb(0.2, 0.5, 0.6),
        }
    }
}

#[derive(Component)]
pub struct Player {
    pub id: PlayerId,
    pub facing: Vec2,
    pub weapon: WeaponType,
    pub ammo: u32,
    pub shoot_cooldown: Timer,
    pub reload_timer: Timer,
    pub reloading: bool,
    pub reload_elapsed: f32,
    pub magazines: HashMap<WeaponType, u32>,
    pub weapon_level: u32,
    pub shoot_loop_sound: Option<Entity>,
}



#[derive(Component)]
pub struct Knockback {
    pub velocity: Vec2,
    pub duration: Timer,
}

#[derive(Component)]
pub struct RegenCooldown {
    pub timer: Timer,
}

#[derive(Component)]
pub struct WeaponSprite;

#[derive(Component)]
pub struct WeaponPart;

#[derive(Component)]
pub struct PlayerHead;

#[derive(Component)]
pub struct PlayerBody;

#[derive(Component)]
pub struct PlayerLeg {
    pub side: f32, // -1.0 links, 1.0 rechts
}

#[derive(Component)]
pub struct PlayerArm {
    pub side: f32, // -1.0 links, 1.0 rechts
    pub has_weapon: bool,
}

#[derive(Component)]
pub struct PlayerEye {
    pub side: f32, // -1.0 links, 1.0 rechts
}

#[derive(Component)]
pub struct ZombieLeg {
    pub side: f32,
}

#[derive(Component)]
pub struct ZombieArm {
    pub side: f32,
}

#[derive(Component)]
pub struct ZombieEye {
    pub side: f32,
}

#[derive(Component)]
pub struct ZombieHead;

#[derive(Component)]
pub struct ZombieBody;

/// Zombie-Variante (0, 1, 2) fuer verschiedene Looks
#[derive(Component, Clone, Copy)]
pub struct ZombieVariant(pub u8);

/// Abgetrenntes Koerperteil das wegfliegt, verrottet und verschwindet
#[derive(Component)]
pub struct Gib {
    pub lifetime: Timer,
    pub on_ground: bool,
    pub decay_timer: Timer,
    pub original_size: Vec2,
}

#[derive(Component)]
pub struct AshCrumble {
    pub timer: Timer,
    pub particle_timer: Timer,
    pub killed: bool, // Kill schon registriert?
}

#[derive(Component)]
pub struct AshParticle {
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct BigZombie;

#[derive(Component)]
pub struct Burning {
    pub damage_per_second: f32,
    pub timer: Timer,
    pub tick_timer: Timer,
}

#[derive(Component)]
pub struct Stunned {
    pub timer: Timer,
}

#[derive(Component)]
pub struct FreezeStacks {
    pub hits: u32,
    pub frozen: bool,
    pub frozen_timer: Timer,
}

#[derive(Component)]
pub struct LightningArc {
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct Zombie {
    pub speed: f32,
    pub damage_cooldown: Timer,
    pub speed_modifier: f32,
    pub freeze_timer: Timer,
    pub groan_timer: Timer,
    pub legs_remaining: u8,
    pub arms_remaining: u8,
    pub crawl_transition: f32, // 0.0 = stehend, 1.0 = liegend
    pub fire_visual: f32, // Akkumulierter Flammenwerfer-Schaden fuer Verkohlungs-Optik
    pub freeze_visual: f32, // Akkumulierter Frostkanone-Schaden fuer Einfrieren-Optik
    pub permanently_frozen: bool,
}

#[derive(Component)]
pub struct Bullet {
    pub damage: f32,
    pub range_remaining: f32,
    pub pierce_remaining: u32,
}

#[derive(Component)]
pub struct BulletOwner(pub PlayerId);

#[derive(Component)]
pub struct FlameBullet;

#[derive(Component)]
pub struct FreezeBullet {
    pub slow_factor: f32,
    pub slow_duration: f32,
}

#[derive(Component)]
pub struct TeslaBullet {
    pub chain_count: u32,
    pub chain_range: f32,
    pub chain_damage: f32,
}

#[derive(Component)]
pub struct GrenadeProjectile {
    pub damage: f32,
    pub fuse: Timer,
    pub explosion_radius: f32,
    pub level: u32,
}

#[derive(Component)]
pub struct RocketProjectile {
    pub damage: f32,
    pub explosion_radius: f32,
    pub range_remaining: f32,
    pub level: u32,
}

#[derive(Component)]
pub struct MineEntity {
    pub damage: f32,
    pub radius: f32,
    pub trigger_radius: f32,
    pub arm_timer: Timer,
}

#[derive(Component)]
pub struct BoomerangProjectile {
    pub damage: f32,
    pub owner_id: PlayerId,
    pub returning: bool,
    pub max_dist: f32,
    pub traveled: f32,
    pub direction: Vec2,
}

#[derive(Component)]
pub struct Spinning {
    pub speed: f32,
}

#[derive(Component)]
pub struct Explosion {
    pub lifetime: Timer,
    pub damage: f32,
    pub radius: f32,
    pub damaged: bool,
    pub level: u32,
}

#[derive(Component)]
pub struct ShockwaveRing;

#[derive(Component)]
pub struct BloodParticle {
    pub lifetime: Timer,
    pub on_ground: bool,
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct PlayerHpBar;

#[derive(Component)]
pub struct PlayerHpBarBg;

#[derive(Component)]
pub struct WaveText;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct ComboTrack;

#[derive(Component)]
pub struct ComboBlock;

#[derive(Component)]
pub struct AmmoIndicator {
    pub player_id: PlayerId,
    pub index: u32,
}

#[derive(Component)]
pub struct WeaponNameText(pub PlayerId);

#[derive(Component)]
pub struct GameOverUi;

#[derive(Component)]
pub struct GameOverUiRoot;

#[derive(Component)]
pub struct RestartButton;

#[derive(Component)]
pub struct WeaponUnlockText {
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct WeaponUnlockIcon {
    pub lifetime: Timer,
    pub player_id: PlayerId,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CrateType {
    Random,
    Base,
}

#[derive(Component)]
pub struct LootCrate {
    pub crate_type: CrateType,
    pub despawn_timer: Timer,
    pub lights: u8,
    pub light_timer: Timer,
}

#[derive(Component)]
pub struct CrateLight {
    pub index: u8,
}

#[derive(Component)]
pub struct GroundDecalLayer;

#[derive(Component)]
pub struct ShaderExplosion {
    pub lifetime: Timer,
    pub damage: f32,
    pub radius: f32,
    pub damaged: bool,
    pub level: u32,
}

/// Speichert die urspruengliche Farbe eines Zombie-Sprites fuer elementare Effekte
#[derive(Component, Clone, Copy)]
pub struct OriginalColor(pub Color);

#[derive(Component)]
pub struct BaseCrateSpawner {
    pub position: Vec2,
    pub respawn_timer: Timer,
    pub active: bool,
}

#[derive(Component)]
pub struct Flare {
    pub burn_timer: Timer,
    pub smoke_timer: Timer,
}

#[derive(Component)]
pub struct SmokeParticle {
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct AirdropCrate {
    pub target_pos: Vec2,
    pub start_pos: Vec2,
    pub fall_speed: f32,
    pub shadow: Entity,
    pub flare: Entity,
    pub elapsed: f32,
    pub curve_offset: f32,
}

#[derive(Component)]
pub struct AirdropShadow;

