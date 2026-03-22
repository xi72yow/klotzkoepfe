use bevy::prelude::*;

use crate::components::Wall;
use crate::constants::*;

pub fn setup_room(mut commands: Commands) {
    do_setup_room(&mut commands);
}

pub fn do_setup_room(commands: &mut Commands) {
    // Boden
    commands.spawn((
        Sprite {
            color: FLOOR_COLOR,
            custom_size: Some(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Wand oben
    commands.spawn((
        Sprite {
            color: WALL_COLOR,
            custom_size: Some(Vec2::new(WINDOW_WIDTH, WALL_THICKNESS)),
            ..default()
        },
        Transform::from_xyz(0.0, WINDOW_HEIGHT / 2.0 - WALL_THICKNESS / 2.0, 5.0),
        Wall,
    ));

    // Wand unten
    commands.spawn((
        Sprite {
            color: WALL_COLOR,
            custom_size: Some(Vec2::new(WINDOW_WIDTH, WALL_THICKNESS)),
            ..default()
        },
        Transform::from_xyz(0.0, -WINDOW_HEIGHT / 2.0 + WALL_THICKNESS / 2.0, 5.0),
        Wall,
    ));

    // Wand links
    commands.spawn((
        Sprite {
            color: WALL_COLOR,
            custom_size: Some(Vec2::new(WALL_THICKNESS, WINDOW_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(-WINDOW_WIDTH / 2.0 + WALL_THICKNESS / 2.0, 0.0, 5.0),
        Wall,
    ));

    // Wand rechts
    commands.spawn((
        Sprite {
            color: WALL_COLOR,
            custom_size: Some(Vec2::new(WALL_THICKNESS, WINDOW_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(WINDOW_WIDTH / 2.0 - WALL_THICKNESS / 2.0, 0.0, 5.0),
        Wall,
    ));
}
