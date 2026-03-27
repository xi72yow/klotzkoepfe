use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;

const CRATE_SIZE: f32 = 20.0;
const LIGHT_SIZE: f32 = 3.0;
const AIRDROP_SPEED_MIN: f32 = 120.0;
const AIRDROP_SPEED_MAX: f32 = 280.0;
const AIRDROP_DIST_MIN: f32 = 350.0;
const AIRDROP_DIST_MAX: f32 = 650.0;
const SMOKE_INTERVAL: f32 = 0.08;
const SMOKE_LIFETIME: f32 = 1.0;
const FLARE_SIZE: f32 = 6.0;

/// Generate a random position inside the room with some margin from walls
fn random_room_position() -> Vec2 {
    let mut rng = rand::rng();
    let margin = WALL_THICKNESS + 40.0;
    let half_w = WINDOW_WIDTH / 2.0 - margin;
    let half_h = WINDOW_HEIGHT / 2.0 - margin;
    Vec2::new(
        rng.random_range(-half_w..half_w),
        rng.random_range(-half_h..half_h),
    )
}

/// Spawn base crate spawners (called at startup/restart)
pub fn setup_base_crates(mut commands: Commands, settings: Res<GameSettings>) {
    do_setup_base_crates(&mut commands, &settings);
}

pub fn do_setup_base_crates(commands: &mut Commands, settings: &GameSettings) {
    for _ in 0..4 {
        commands.spawn(BaseCrateSpawner {
            position: Vec2::ZERO,
            respawn_timer: Timer::from_seconds(settings.base_crate_respawn_time, TimerMode::Once),
            active: true,
        });
    }
}

/// Spawn a random red crate at a given position (called from collision on kill)
pub fn spawn_random_crate(commands: &mut Commands, pos: Vec2, despawn_time: f32) {
    spawn_loot_crate(commands, pos, CrateType::Random, despawn_time);
}

fn spawn_loot_crate(commands: &mut Commands, pos: Vec2, crate_type: CrateType, despawn_time: f32) {
    let color = match crate_type {
        CrateType::Random => Color::srgb(0.8, 0.15, 0.1),
        CrateType::Base => Color::srgb(0.6, 0.5, 0.2),
    };
    let light_timer_duration = if crate_type == CrateType::Random {
        despawn_time / 6.0
    } else {
        999.0 // Base crates don't despawn
    };

    commands
        .spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::splat(CRATE_SIZE)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 7.0),
            LootCrate {
                crate_type,
                despawn_timer: Timer::from_seconds(despawn_time, TimerMode::Once),
                lights: 5,
                light_timer: Timer::from_seconds(light_timer_duration, TimerMode::Repeating),
            },
        ))
        .with_children(|parent| {
            // 5 lights on top
            let light_colors = [
                Color::srgb(0.0, 1.0, 0.0),
                Color::srgb(1.0, 1.0, 0.0),
                Color::srgb(1.0, 0.5, 0.0),
                Color::srgb(1.0, 0.0, 0.0),
                Color::srgb(0.5, 0.0, 1.0),
            ];
            for i in 0..5u8 {
                let lx = -6.0 + i as f32 * 3.0;
                parent.spawn((
                    Sprite {
                        color: light_colors[i as usize],
                        custom_size: Some(Vec2::splat(LIGHT_SIZE)),
                        ..default()
                    },
                    Transform::from_xyz(lx, CRATE_SIZE / 2.0 + 2.0, 1.0),
                    CrateLight { index: i },
                ));
            }
        });
}

/// Update crates: despawn animation and pickup
pub fn crate_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<GameSettings>,
    score: Res<Score>,
    mut player_query: Query<(&mut Player, &mut Health, &Transform)>,
    mut crate_query: Query<(Entity, &mut LootCrate, &Transform, &Children)>,
    mut light_query: Query<(&CrateLight, &mut Visibility)>,
    mut sound_events: ResMut<super::audio::SoundQueue>,
) {
    for (crate_entity, mut loot_crate, crate_transform, children) in crate_query.iter_mut() {
        let crate_pos = crate_transform.translation.truncate();

        // Pickup check
        let mut picked_up = false;
        for (mut player, mut health, player_transform) in player_query.iter_mut() {
            let player_pos = player_transform.translation.truncate();
            if crate_pos.distance(player_pos) < 25.0 {
                // Give loot: ammo + healing if not full HP
                let lvl = settings.weapon_level(player.weapon, score.points);
                let ws = settings.weapon_at_level(player.weapon, lvl);
                player.ammo = ws.magazine;
                player.reloading = false;
                player.reload_elapsed = 0.0;

                // Healing: 25 HP if not full
                if health.current < health.max {
                    health.current = (health.current + 25.0).min(health.max);
                }

                match loot_crate.crate_type {
                    CrateType::Random => {
                        // Refill all magazines
                        for w in WeaponType::all() {
                            let wset = settings.weapon(*w);
                            let max_mags = if wset.max_magazines > 0 { wset.max_magazines } else { 999 };
                            player.magazines.insert(*w, max_mags);
                        }
                    }
                    CrateType::Base => {
                        // Refill one magazine for current weapon
                        let current_weapon = player.weapon;
                        let max_mags = if ws.max_magazines > 0 { ws.max_magazines } else { 999 };
                        let mags = player.magazines.entry(current_weapon).or_insert(0);
                        *mags = (*mags + 1).min(max_mags);
                    }
                }
                picked_up = true;
                sound_events.0.push(super::audio::SoundEvent::CratePickup);
                break;
            }
        }

        if picked_up {
            commands.entity(crate_entity).try_despawn();

            // If base crate, spawn respawn timer entity
            if loot_crate.crate_type == CrateType::Base {
                commands.spawn(BaseCrateSpawner {
                    position: Vec2::ZERO,
                    respawn_timer: Timer::from_seconds(settings.base_crate_respawn_time, TimerMode::Once),
                    active: true,
                });
            }
            continue;
        }

        // Despawn timer (only for random crates)
        if loot_crate.crate_type == CrateType::Random {
            loot_crate.despawn_timer.tick(time.delta());
            loot_crate.light_timer.tick(time.delta());

            // Turn off lights one by one
            if loot_crate.light_timer.just_finished() && loot_crate.lights > 0 {
                loot_crate.lights -= 1;
                let off_index = loot_crate.lights;
                for child in children.iter() {
                    if let Ok((light, mut vis)) = light_query.get_mut(child) {
                        if light.index == off_index {
                            *vis = Visibility::Hidden;
                        }
                    }
                }
            }

            if loot_crate.despawn_timer.is_finished() {
                commands.entity(crate_entity).try_despawn();
            }
        }
    }
}

/// Respawn base crates: timer -> spawn flare instead of direct crate
pub fn base_crate_respawn(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<GameSettings>,
    mut spawner_query: Query<(Entity, &mut BaseCrateSpawner)>,
) {
    for (entity, mut spawner) in spawner_query.iter_mut() {
        spawner.respawn_timer.tick(time.delta());
        if spawner.respawn_timer.is_finished() {
            let pos = random_room_position();
            spawn_flare(&mut commands, pos, &settings);
            commands.entity(entity).try_despawn();
        }
    }
}

fn spawn_flare(commands: &mut Commands, pos: Vec2, settings: &GameSettings) {
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 0.2, 0.0),
            custom_size: Some(Vec2::splat(FLARE_SIZE)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 6.0),
        Flare {
            burn_timer: Timer::from_seconds(settings.flare_duration, TimerMode::Once),
            smoke_timer: Timer::from_seconds(SMOKE_INTERVAL, TimerMode::Repeating),
        },
    ));
}

/// Flare system: smoke particles, trigger airdrop
pub fn flare_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<GameSettings>,
    mut flare_query: Query<(Entity, &mut Flare, &Transform)>,
) {
    let mut rng = rand::rng();

    for (entity, mut flare, transform) in flare_query.iter_mut() {
        flare.burn_timer.tick(time.delta());
        flare.smoke_timer.tick(time.delta());

        // Spawn smoke particles
        if flare.smoke_timer.just_finished() {
            let pos = transform.translation.truncate();
            let x_offset = rng.random_range(-3.0..3.0);
            commands.spawn((
                Sprite {
                    color: Color::srgba(0.5, 0.5, 0.5, 0.5),
                    custom_size: Some(Vec2::splat(4.0)),
                    ..default()
                },
                Transform::from_xyz(pos.x + x_offset, pos.y, 5.5),
                SmokeParticle {
                    lifetime: Timer::from_seconds(SMOKE_LIFETIME, TimerMode::Once),
                },
                Velocity(Vec2::new(
                    rng.random_range(-5.0..5.0),
                    rng.random_range(25.0..45.0),
                )),
            ));
        }

        // When burn timer finishes, spawn airdrop (flare stays until landing)
        if flare.burn_timer.is_finished() {
            let pos = transform.translation.truncate();
            spawn_airdrop(&mut commands, pos, entity);
            // Stop smoke by removing the Flare component (entity stays as visual marker)
            commands.entity(entity).remove::<Flare>();
        }
    }
}

/// Smoke particle system: rise, fade, despawn
pub fn smoke_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut SmokeParticle, &mut Transform, &mut Sprite, &Velocity)>,
) {
    let dt = time.delta_secs();
    for (entity, mut smoke, mut transform, mut sprite, velocity) in query.iter_mut() {
        smoke.lifetime.tick(time.delta());

        transform.translation.x += velocity.0.x * dt;
        transform.translation.y += velocity.0.y * dt;

        // Fade out based on lifetime progress
        let progress = smoke.lifetime.fraction();
        let alpha = 0.5 * (1.0 - progress);
        let scale = 1.0 + progress * 0.5; // Grow slightly
        sprite.color = Color::srgba(0.5, 0.5, 0.5, alpha);
        transform.scale = Vec3::splat(scale);

        if smoke.lifetime.is_finished() {
            commands.entity(entity).try_despawn();
        }
    }
}

fn spawn_airdrop(commands: &mut Commands, target_pos: Vec2, flare_entity: Entity) {
    let mut rng = rand::rng();

    // Random approach: angle, distance, speed, curve
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let distance = rng.random_range(AIRDROP_DIST_MIN..AIRDROP_DIST_MAX);
    let speed = rng.random_range(AIRDROP_SPEED_MIN..AIRDROP_SPEED_MAX);
    let curve = rng.random_range(-80.0..80.0); // Seitliche Kurve
    let start_pos = Vec2::new(
        target_pos.x + angle.cos() * distance,
        target_pos.y + angle.sin() * distance,
    );

    // Shadow on the ground - starts at crate start position
    let shadow_entity = commands.spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.4),
            custom_size: Some(Vec2::splat(CRATE_SIZE + 10.0)),
            ..default()
        },
        Transform::from_xyz(start_pos.x, start_pos.y, 1.5)
            .with_scale(Vec3::splat(0.3)),
        AirdropShadow,
    )).id();

    // Falling crate from diagonal direction
    let color = Color::srgb(0.6, 0.5, 0.2);
    commands
        .spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::splat(CRATE_SIZE)),
                ..default()
            },
            Transform::from_xyz(start_pos.x, start_pos.y, 9.0),
            AirdropCrate {
                target_pos,
                start_pos,
                fall_speed: speed,
                shadow: shadow_entity,
                flare: flare_entity,
                elapsed: 0.0,
                curve_offset: curve,
            },
        ))
        .with_children(|parent| {
            let light_colors = [
                Color::srgb(0.0, 1.0, 0.0),
                Color::srgb(1.0, 1.0, 0.0),
                Color::srgb(1.0, 0.5, 0.0),
                Color::srgb(1.0, 0.0, 0.0),
                Color::srgb(0.5, 0.0, 1.0),
            ];
            for i in 0..5u8 {
                let lx = -6.0 + i as f32 * 3.0;
                parent.spawn((
                    Sprite {
                        color: light_colors[i as usize],
                        custom_size: Some(Vec2::splat(LIGHT_SIZE)),
                        ..default()
                    },
                    Transform::from_xyz(lx, CRATE_SIZE / 2.0 + 2.0, 1.0),
                    CrateLight { index: i },
                ));
            }
        });
}

/// Airdrop system: shadow moves on ground, crate hovers above with height offset + wobble
pub fn airdrop_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<GameSettings>,
    mut crate_query: Query<(Entity, &mut AirdropCrate, &mut Transform)>,
    mut shadow_query: Query<&mut Transform, (With<AirdropShadow>, Without<AirdropCrate>)>,
) {
    let dt = time.delta_secs();

    for (entity, mut airdrop, mut transform) in crate_query.iter_mut() {
        airdrop.elapsed += dt;

        // Progress: 0 = start, 1 = landed
        let total_distance = airdrop.start_pos.distance(airdrop.target_pos);
        let traveled = airdrop.elapsed * airdrop.fall_speed;
        let progress = (traveled / total_distance).clamp(0.0, 1.0);

        // Curved ground path: quadratic bezier with perpendicular control point
        let dir = (airdrop.target_pos - airdrop.start_pos).normalize_or_zero();
        let perp = Vec2::new(-dir.y, dir.x);
        let mid = airdrop.start_pos.lerp(airdrop.target_pos, 0.5) + perp * airdrop.curve_offset;
        let inv = 1.0 - progress;
        let ground_pos = airdrop.start_pos * inv * inv
            + mid * 2.0 * inv * progress
            + airdrop.target_pos * progress * progress;

        // Subtle wobble
        let t = airdrop.elapsed;
        let wobble_x = (t * 5.1).sin() * 3.0 + (t * 11.7).cos() * 2.0;
        let wobble_y = (t * 7.9).cos() * 2.5 + (t * 4.1).sin() * 1.5;

        // Height: crate above shadow, decreases to 0 at landing
        let height = 1.0 - progress;
        let height_offset = height * 60.0;
        let wobble_strength = height;

        // Crate position = ground + height + wobble
        transform.translation.x = ground_pos.x + wobble_x * wobble_strength;
        transform.translation.y = ground_pos.y + height_offset + wobble_y * wobble_strength;

        // Crate scale: large when high (1.8), normal when landed (1.0)
        let crate_scale = 1.0 + 0.8 * height;
        transform.scale = Vec3::splat(crate_scale);

        // Shadow on the ground: follows ground_pos, grows as crate descends
        if let Ok(mut shadow_transform) = shadow_query.get_mut(airdrop.shadow) {
            shadow_transform.translation.x = ground_pos.x;
            shadow_transform.translation.y = ground_pos.y;
            let shadow_scale = 0.3 + 0.7 * progress;
            shadow_transform.scale = Vec3::splat(shadow_scale);
        }

        // Landing check
        if progress >= 1.0 {
            let pos = airdrop.target_pos;
            commands.entity(entity).try_despawn();
            commands.entity(airdrop.shadow).try_despawn();
            commands.entity(airdrop.flare).try_despawn();

            // Spawn actual loot crate
            spawn_loot_crate(&mut commands, pos, CrateType::Base, settings.crate_despawn_time);

            // Impact dust particles
            let mut rng = rand::rng();
            for _ in 0..6 {
                let angle = rng.random_range(0.0..std::f32::consts::TAU);
                let speed = rng.random_range(30.0..80.0);
                commands.spawn((
                    Sprite {
                        color: Color::srgba(0.6, 0.55, 0.4, 0.6),
                        custom_size: Some(Vec2::splat(3.0)),
                        ..default()
                    },
                    Transform::from_xyz(pos.x, pos.y, 5.0),
                    SmokeParticle {
                        lifetime: Timer::from_seconds(0.5, TimerMode::Once),
                    },
                    Velocity(Vec2::new(angle.cos() * speed, angle.sin() * speed)),
                ));
            }
        }
    }
}
