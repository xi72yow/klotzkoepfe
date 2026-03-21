use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlayerId {
    P1,
    P2,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
        match self {
            WeaponType::Pistol => "Pistole",
            WeaponType::Uzi => "Uzi",
            WeaponType::Grenade => "Granate",
            WeaponType::Railgun => "Railgun",
            WeaponType::Flamethrower => "Flammenwerfer",
            WeaponType::Shotgun => "Shotgun",
            WeaponType::Laser => "Laser",
            WeaponType::Mine => "Mine",
            WeaponType::Boomerang => "Boomerang",
            WeaponType::Tesla => "Tesla",
            WeaponType::Buzzsaw => "Kreissaege",
            WeaponType::Rocket => "Rakete",
            WeaponType::FreezeGun => "Freeze Gun",
        }
    }

    pub fn bullet_size(self) -> Vec2 {
        match self {
            WeaponType::Pistol => Vec2::new(8.0, 4.0),
            WeaponType::Uzi => Vec2::new(6.0, 3.0),
            WeaponType::Grenade => Vec2::new(10.0, 10.0),
            WeaponType::Railgun => Vec2::new(20.0, 3.0),
            WeaponType::Flamethrower => Vec2::new(6.0, 6.0),
            WeaponType::Shotgun => Vec2::new(5.0, 3.0),
            WeaponType::Laser => Vec2::new(30.0, 2.0),
            WeaponType::Mine => Vec2::new(12.0, 12.0),
            WeaponType::Boomerang => Vec2::new(12.0, 12.0),
            WeaponType::Tesla => Vec2::new(8.0, 8.0),
            WeaponType::Buzzsaw => Vec2::new(16.0, 16.0),
            WeaponType::Rocket => Vec2::new(12.0, 6.0),
            WeaponType::FreezeGun => Vec2::new(8.0, 8.0),
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
}

#[derive(Component)]
pub struct WeaponSprite;

#[derive(Component)]
pub struct Zombie {
    pub speed: f32,
    pub damage_cooldown: Timer,
    pub speed_modifier: f32,
    pub freeze_timer: Timer,
}

#[derive(Component)]
pub struct Bullet {
    pub damage: f32,
    pub range_remaining: f32,
    pub pierce_remaining: u32,
}

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
}

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
pub struct WeaponUnlockText {
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct WeaponUnlockIcon {
    pub lifetime: Timer,
    pub player_id: PlayerId,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DropType {
    Ammo,
    Health,
}

#[derive(Component)]
pub struct DropItem {
    pub drop_type: DropType,
    pub lifetime: Timer,
}
