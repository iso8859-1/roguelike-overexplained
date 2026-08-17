use bevy::prelude::*;
use std::collections::HashMap;
use std::ops::{Add, Sub};

use super::{FIELD_SIZE_Y, map_to_screen_coordinates};

pub const DEFAULT_MAP_WIDTH: u32 = 120;
pub const DEFAULT_MAP_HEIGHT: u32 = 40;

pub const TERRAIN_Z: i32 = 0;
pub const ACTORS_Z: i32 = 1;

#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Hash, Copy)]
pub struct MapPosition {
    pub x: i32,
    pub y: i32,
}

impl From<(i32, i32)> for MapPosition {
    fn from(pos: (i32, i32)) -> Self {
        MapPosition { x: pos.0, y: pos.1 }
    }
}

impl Add for MapPosition {
    type Output = MapPosition;

    fn add(self, other: MapPosition) -> MapPosition {
        MapPosition {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for MapPosition {
    type Output = MapPosition;

    fn sub(self, other: MapPosition) -> MapPosition {
        MapPosition {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl MapPosition {
    /// Grid distance for 8-directional movement (Chebyshev distance).
    pub fn distance_to(self, other: MapPosition) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

const NEIGHBORS: [MapPosition; 8] = [
    MapPosition { x: 1, y: 1 },
    MapPosition { x: 1, y: 0 },
    MapPosition { x: 1, y: -1 },
    MapPosition { x: 0, y: -1 },
    MapPosition { x: -1, y: -1 },
    MapPosition { x: -1, y: 0 },
    MapPosition { x: -1, y: 1 },
    MapPosition { x: 0, y: 1 },
];

pub fn neighbors_of(center: MapPosition) -> [MapPosition; 8] {
    NEIGHBORS.map(|offset| center + offset)
}

#[derive(Resource)]
pub struct Map {
    width: u32,
    height: u32,
    entities: HashMap<MapPosition, Entity>,
    positions: HashMap<Entity, MapPosition>,
    player: Option<Entity>,
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Terrain;

#[derive(Component)]
pub struct Npc;

impl Map {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            entities: HashMap::new(),
            positions: HashMap::new(),
            player: None,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn player(&self) -> Option<Entity> {
        self.player
    }

    pub fn position(&self, e: Entity) -> Option<MapPosition> {
        self.positions.get(&e).cloned()
    }

    pub fn player_position(&self) -> Option<MapPosition> {
        match self.player() {
            Some(player) => self.position(player),
            None => None,
        }
    }

    pub fn check_collision(&self, pos: MapPosition) -> bool {
        self.entities.contains_key(&pos)
    }

    fn add_entity(&mut self, position: MapPosition, entity: Entity) {
        self.entities.insert(position, entity);
        self.positions.insert(entity, position);
    }

    fn remove_entity(&mut self, position: &MapPosition) {
        let v = self.entities.remove(position);
        if let Some(entity) = v {
            self.positions.remove(&entity);
        }
    }

    pub fn get_entity(&self, position: &MapPosition) -> Option<&Entity> {
        self.entities.get(position)
    }

    pub fn update_entity_position(
        &mut self,
        old_position: &MapPosition,
        new_position: MapPosition,
    ) {
        if let Some(entity) = self.entities.remove(old_position) {
            self.entities.insert(new_position, entity);
            self.positions.remove(&entity);
            self.positions.insert(entity, new_position);
        }
    }

    pub fn spawn_wall(&mut self, commands: &mut Commands, x: i32, y: i32) {
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

    pub fn spawn_player(&mut self, commands: &mut Commands, x: i32, y: i32) {
        let e = commands.spawn((
            Text2d::new("@"),
            TextFont {
                font_size: FontSize::Px(FIELD_SIZE_Y),
                font: default(),
                ..default()
            },
            TextColor(Color::linear_rgb(1.0, 0.0, 0.0)),
            Transform::from_translation(map_to_screen_coordinates(x, y, ACTORS_Z)),
            MapPosition { x, y },
            Player,
        ));
        self.add_entity(MapPosition { x, y }, e.id());
        self.player = Some(e.id());
    }

    pub fn spawn_npc(&mut self, commands: &mut Commands, x: i32, y: i32, symbol: &str) {
        let e = commands.spawn((
            Text2d::new(symbol),
            TextFont {
                font_size: FontSize::Px(FIELD_SIZE_Y),
                font: default(),
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_translation(map_to_screen_coordinates(x, y, ACTORS_Z)),
            MapPosition { x, y },
            Npc,
        ));
        self.add_entity(MapPosition { x, y }, e.id());
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::new(DEFAULT_MAP_WIDTH, DEFAULT_MAP_HEIGHT)
    }
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Map::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance() {
        let pos1 = MapPosition { x: 0, y: 0 };
        let pos2 = MapPosition { x: 3, y: 4 };
        assert_eq!(pos1.distance_to(pos2), 7);
    }

    #[test]
    fn test_neighbors() {
        let center = MapPosition { x: 5, y: 5 };
        let neighbors = neighbors_of(center);
        let expected_neighbors = [
            MapPosition { x: 6, y: 6 },
            MapPosition { x: 6, y: 5 },
            MapPosition { x: 6, y: 4 },
            MapPosition { x: 5, y: 4 },
            MapPosition { x: 4, y: 4 },
            MapPosition { x: 4, y: 5 },
            MapPosition { x: 4, y: 6 },
            MapPosition { x: 5, y: 6 },
        ];
        assert_eq!(neighbors, expected_neighbors);
    }

    #[test]
    fn test_addition() {
        let pos1 = MapPosition { x: 2, y: 3 };
        let pos2 = MapPosition { x: 4, y: 5 };
        let result = pos1 + pos2;
        assert_eq!(result, MapPosition { x: 6, y: 8 });
    }

    #[test]
    fn test_subtraction() {
        let pos1 = MapPosition { x: 5, y: 7 };
        let pos2 = MapPosition { x: 2, y: 3 };
        let result = pos1 - pos2;
        assert_eq!(result, MapPosition { x: 3, y: 4 });
    }

    #[test]
    fn test_from_tuple() {
        let pos: MapPosition = (10, 20).into();
        assert_eq!(pos, MapPosition { x: 10, y: 20 });
    }
}
