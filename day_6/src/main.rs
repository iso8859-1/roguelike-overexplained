use bevy::log::LogPlugin;
use bevy::prelude::*;
use std::collections::HashMap;

const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;
const FIELD_SIZE_X: f32 = 16.0;
const FIELD_SIZE_Y: f32 = 24.0;
const MAP_WIDTH: u32 = 120;
const MAP_HEIGHT: u32 = 40;

const TERRAIN_Z: u32 = 0;
const ACTORS_Z: u32 = 1;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Movement;

#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Hash)]
struct MapPosition {
    x: u32,
    y: u32,
}

#[derive(Component)]
struct Terrain;

#[derive(Component)]
struct Being;

#[derive(Resource)]
struct Map {
    width: u32,
    height: u32,
    entities: HashMap<MapPosition, Entity>,
}

impl Map {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            entities: HashMap::new(),
        }
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn add_entity(&mut self, position: MapPosition, entity: Entity) {
        self.entities.insert(position, entity);
    }

    fn remove_entity(&mut self, position: &MapPosition) {
        self.entities.remove(position);
    }

    fn get_entity(&self, position: &MapPosition) -> Option<&Entity> {
        self.entities.get(position)
    }

    fn update_entity_position(&mut self, old_position: &MapPosition, new_position: MapPosition) {
        if let Some(entity) = self.entities.remove(old_position) {
            self.entities.insert(new_position, entity);
        }
    }

    fn spawn_wall(&mut self, commands: &mut Commands, x: u32, y: u32) {
        let e = commands.spawn((
            Text2d::new("#"), 
            TextFont { 
                font_size: FontSize::Px(FIELD_SIZE_Y), 
                font: default(),
                ..default()
                },
                TextColor(Color::WHITE), 
                Transform::from_translation(map_to_screen_coordinates(x, y, TERRAIN_Z)),
                Terrain,
        ));
        self.add_entity(MapPosition { x, y }, e.id());
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::new(MAP_WIDTH, MAP_HEIGHT)
    }
}

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
    .insert_resource(Map::default())
	.run();
}

fn setup(mut map: ResMut<Map>, mut commands: Commands) {
	commands.spawn(Camera2d);

	let e = commands.spawn((
        Text2d::new("@"),
        TextFont {
            font_size: FontSize::Px(FIELD_SIZE_Y),	
            font: default(),
            ..default()
        },
        TextColor(Color::linear_rgb(1.0,0.0, 0.0)),
        Transform::from_translation(map_to_screen_coordinates(2, 3, ACTORS_Z)),
        MapPosition { x: 2, y: 3 },
        Player,
        Being,
    ));
    map.add_entity(MapPosition { x: 2, y: 3 }, e.id());

    info!("Player spawned");

    for y in 0..map.height() {
        let width = map.width();
        map.spawn_wall(&mut commands, 0, y);
        map.spawn_wall(&mut commands, width - 1, y);
    }
    for x in 1..map.width() - 1 {
        let height = map.height();
        map.spawn_wall(&mut commands, x, 0);
        map.spawn_wall(&mut commands, x, height - 1);
    }
}

fn keyboard_input(
    mut map: ResMut<Map>,
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
            if map.get_entity(&map_position).is_some() {
                debug!("Player {entity} collided with an entity at ({}, {})", map_position.x, map_position.y);
                *map_position = original_position;
            } else {
                debug!("Player {entity} moved to ({}, {})", map_position.x, map_position.y);
                commands.entity(entity).insert(Movement{});
                map.update_entity_position(&original_position, map_position.clone());
            }
        }
    }
}

fn move_entity(time: Res<Time>, mut query: Query<(&mut Transform, &MapPosition, Entity), With<Movement>>, mut commands: Commands) {
    for (mut transform, map_position, entity) in query.iter_mut() {
        let speed = 100.0;
        let delta = time.delta_secs();
        let target_position = map_to_screen_coordinates(map_position.x, map_position.y, ACTORS_Z);
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
                let screen_coords = map_to_screen_coordinates(x, y, ACTORS_Z);
                let (map_x, map_y) = screen_to_map_coordinates(screen_coords.x, screen_coords.y);
                assert_eq!((map_x, map_y), (x, y));
            }
        }
    }

    #[test]
    fn test_screen_to_map_coordinates() {
        let (reference_map_x, reference_map_y) = (5, 7);
        let center = map_to_screen_coordinates(reference_map_x, reference_map_y, ACTORS_Z);
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