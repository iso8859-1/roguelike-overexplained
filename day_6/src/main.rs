use bevy::log::LogPlugin;
use bevy::prelude::*;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Movement;

#[derive(Component, Debug, Clone, PartialEq, Eq, Default)]
struct MapPosition {
    x: u32,
    y: u32,
}

#[derive(Component)]
struct MapTile;

const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;
const FIELD_SIZE_X: f32 = 16.0;
const FIELD_SIZE_Y: f32 = 24.0;
const MAP_WIDTH: u32 = 120;
const MAP_HEIGHT: u32 = 40;

const TERRAIN_Z: u32 = 0;
const ENTITIES_Z: u32 = 1;


fn map_to_screen_coordinates(map_x: u32, map_y: u32, z_level: u32) -> Vec3 {
    let screen_x = -(WINDOW_WIDTH as f32 / 2.0) + (map_x as f32 * FIELD_SIZE_X) + (FIELD_SIZE_X / 2.0);
    let screen_y = WINDOW_HEIGHT as f32 / 2.0 - (map_y as f32 * FIELD_SIZE_Y) - (FIELD_SIZE_Y / 2.0);
    Vec3::new(screen_x, screen_y, z_level as f32)
}

fn screen_to_map_coordinates(screen_x: f32, screen_y: f32) -> (u32, u32) {
    let map_x = ((screen_x + (WINDOW_WIDTH as f32 / 2.0)) / FIELD_SIZE_X).floor() as u32;
    let map_y = (((WINDOW_HEIGHT as f32 / 2.0 - screen_y) / FIELD_SIZE_Y).ceil() - 1.0) as u32;
    (map_x, map_y)
}

fn main() {
    let mut default_plugins = DefaultPlugins.build();
    default_plugins = default_plugins.set(LogPlugin {
        level: bevy::log::Level::DEBUG,
        ..default()
    });
    default_plugins = default_plugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Roguelike Overexplained".to_string(),
            resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
            resizable: false,
            ..default()
        }),
        ..default()
    });
	App::new()
	.add_plugins(default_plugins)
	.add_systems(Startup, setup)
    .add_systems(Update, keyboard_input)
    .add_systems(Update, move_entity)
	.run();
}

fn spawn_wall(commands: &mut Commands, x: u32, y: u32) {
    commands.spawn((
        Text2d::new("#"), 
        TextFont { 
            font_size: FontSize::Px(FIELD_SIZE_Y), 
            font: default(),
            ..default()
            },
            TextColor(Color::WHITE), 
            Transform::from_translation(map_to_screen_coordinates(x, y, TERRAIN_Z)),
            MapTile,
    ));
}

fn setup(mut commands: Commands) {
	commands.spawn(Camera2d);

	commands.spawn((
        Text2d::new("@"),
        TextFont {
            font_size: FontSize::Px(FIELD_SIZE_Y),	
            font: default(),
            ..default()
        },
        TextColor(Color::linear_rgb(1.0,0.0, 0.0)),
        Transform::from_translation(map_to_screen_coordinates(2, 3, ENTITIES_Z)),
        MapPosition { x: 2, y: 3 },
        Player,
    ));

    info!("Player spawned");

    for y in 0..MAP_HEIGHT {
        spawn_wall(&mut commands, 0, y);
        spawn_wall(&mut commands, MAP_WIDTH - 1, y);
    }
    for x in 1..MAP_WIDTH - 1 {
        spawn_wall(&mut commands, x, 0);
        spawn_wall(&mut commands, x, MAP_HEIGHT - 1);
    }
}

fn keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, &mut MapPosition), (With<Player>, Without<Movement>)>,
    mut commands: Commands
) {
    for (entity, mut map_position) in query.iter_mut() {
        let original_position = map_position.clone();
        if keyboard_input.pressed(KeyCode::KeyW) {
            map_position.y = map_position.y-1;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            map_position.y = map_position.y+1;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            map_position.x = map_position.x-1;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            map_position.x = map_position.x+1;
        }
        if original_position != *map_position {
            debug!("Player {entity} moved to ({}, {})", map_position.x, map_position.y);
            commands.entity(entity).insert(Movement{});
        }
    }
}

fn move_entity(time: Res<Time>, mut query: Query<(&mut Transform, &MapPosition, Entity), With<Movement>>, mut commands: Commands) {
    for (mut transform, map_position, entity) in query.iter_mut() {
        let speed = 100.0;
        let delta = time.delta_secs();
        let target_position = map_to_screen_coordinates(map_position.x, map_position.y, ENTITIES_Z);
        let direction = (target_position - transform.translation).normalize_or_zero();
        let distance = (target_position - transform.translation).length();
        let movement_distance = speed * delta;
        if distance <= movement_distance {
            transform.translation = target_position;
            commands.entity(entity).remove::<Movement>();
        } else {    
            transform.translation += direction * movement_distance;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_to_screen_coordinates() {
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                let screen_coords = map_to_screen_coordinates(x, y, ENTITIES_Z);
                let (map_x, map_y) = screen_to_map_coordinates(screen_coords.x, screen_coords.y);
                assert_eq!((map_x, map_y), (x, y));
            }
        }
    }

    #[test]
    fn test_screen_to_map_coordinates() {
        let (reference_map_x, reference_map_y) = (5, 7);
        let center = map_to_screen_coordinates(reference_map_x, reference_map_y, ENTITIES_Z);
        let left_edge_x = center.x - FIELD_SIZE_X / 2.0;
        let bottom_edge_y = center.y - FIELD_SIZE_Y / 2.0;
        for x in 0..(FIELD_SIZE_X as i32) {
            for y in 0..(FIELD_SIZE_Y as i32) {
                let screen_x = left_edge_x + x as f32;
                let screen_y = bottom_edge_y + y as f32;
                assert_eq!(
                    screen_to_map_coordinates(screen_x, screen_y),
                    (reference_map_x, reference_map_y),
                    "mismatch at x={x}, y={y}"
                );
            }
        }
    }
}