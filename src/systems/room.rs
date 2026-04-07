use bevy::prelude::*;

use crate::components::{Wall, WallSide, FloorSprite};
use crate::constants::*;
use crate::resources::GameField;

pub fn setup_room(mut commands: Commands) {
    do_setup_room(&mut commands);
}

pub fn do_setup_room(commands: &mut Commands) {
    let w = WINDOW_WIDTH;
    let h = WINDOW_HEIGHT;

    // Boden
    commands.spawn((
        Sprite {
            color: FLOOR_COLOR,
            custom_size: Some(Vec2::new(w, h)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        FloorSprite,
    ));

    // Wand oben
    commands.spawn((
        Sprite { color: WALL_COLOR, custom_size: Some(Vec2::new(w, WALL_THICKNESS)), ..default() },
        Transform::from_xyz(0.0, h / 2.0 - WALL_THICKNESS / 2.0, 5.0),
        Wall, WallSide::Top,
    ));
    // Wand unten
    commands.spawn((
        Sprite { color: WALL_COLOR, custom_size: Some(Vec2::new(w, WALL_THICKNESS)), ..default() },
        Transform::from_xyz(0.0, -h / 2.0 + WALL_THICKNESS / 2.0, 5.0),
        Wall, WallSide::Bottom,
    ));
    // Wand links
    commands.spawn((
        Sprite { color: WALL_COLOR, custom_size: Some(Vec2::new(WALL_THICKNESS, h)), ..default() },
        Transform::from_xyz(-w / 2.0 + WALL_THICKNESS / 2.0, 0.0, 5.0),
        Wall, WallSide::Left,
    ));
    // Wand rechts
    commands.spawn((
        Sprite { color: WALL_COLOR, custom_size: Some(Vec2::new(WALL_THICKNESS, h)), ..default() },
        Transform::from_xyz(w / 2.0 - WALL_THICKNESS / 2.0, 0.0, 5.0),
        Wall, WallSide::Right,
    ));
}

/// Synchronisiert Waende und Boden mit der aktuellen GameField-Groesse
pub fn sync_room(
    field: Res<GameField>,
    mut wall_query: Query<(&WallSide, &mut Transform, &mut Sprite), With<Wall>>,
    mut floor_query: Query<&mut Sprite, (With<FloorSprite>, Without<Wall>)>,
) {
    let w = field.width;
    let h = field.height;

    for (side, mut transform, mut sprite) in wall_query.iter_mut() {
        match side {
            WallSide::Top => {
                sprite.custom_size = Some(Vec2::new(w, WALL_THICKNESS));
                transform.translation = Vec3::new(0.0, h / 2.0 - WALL_THICKNESS / 2.0, 5.0);
            }
            WallSide::Bottom => {
                sprite.custom_size = Some(Vec2::new(w, WALL_THICKNESS));
                transform.translation = Vec3::new(0.0, -h / 2.0 + WALL_THICKNESS / 2.0, 5.0);
            }
            WallSide::Left => {
                sprite.custom_size = Some(Vec2::new(WALL_THICKNESS, h));
                transform.translation = Vec3::new(-w / 2.0 + WALL_THICKNESS / 2.0, 0.0, 5.0);
            }
            WallSide::Right => {
                sprite.custom_size = Some(Vec2::new(WALL_THICKNESS, h));
                transform.translation = Vec3::new(w / 2.0 - WALL_THICKNESS / 2.0, 0.0, 5.0);
            }
        }
    }

    for mut sprite in floor_query.iter_mut() {
        sprite.custom_size = Some(Vec2::new(w, h));
    }
}
