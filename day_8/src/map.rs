use bevy::prelude::*;
use std::collections::HashMap;

use super::{map_to_screen_coordinates, FIELD_SIZE_Y};

pub const DEFAULT_MAP_WIDTH: u32 = 120;
pub const DEFAULT_MAP_HEIGHT: u32 = 40;

pub const TERRAIN_Z: u32 = 0;
pub const ACTORS_Z: u32 = 1;

#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct MapPosition {
    pub x: u32,
    pub y: u32,
}

#[derive(Resource)]
pub struct Map {
    width: u32,
    height: u32,
    entities: HashMap<MapPosition, Entity>,
}

#[derive(Component)]
pub struct Terrain;

#[derive(Component)]
pub struct Being;

impl Map {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            entities: HashMap::new(),
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    fn add_entity(&mut self, position: MapPosition, entity: Entity) {
        self.entities.insert(position, entity);
    }

    fn remove_entity(&mut self, position: &MapPosition) {
        self.entities.remove(position);
    }

    pub fn get_entity(&self, position: &MapPosition) -> Option<&Entity> {
        self.entities.get(position)
    }

    pub fn update_entity_position(&mut self, old_position: &MapPosition, new_position: MapPosition) {
        if let Some(entity) = self.entities.remove(old_position) {
            self.entities.insert(new_position, entity);
        }
    }

    pub fn spawn_wall(&mut self, commands: &mut Commands, x: u32, y: u32) {
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

    pub fn spawn_player(&mut self, commands: &mut Commands, x: u32, y: u32) {
        let e = commands.spawn((
            Text2d::new("@"),
            TextFont {
                font_size: FontSize::Px(FIELD_SIZE_Y),	
                font: default(),
                ..default()
            },
            TextColor(Color::linear_rgb(1.0,0.0, 0.0)),
            Transform::from_translation(map_to_screen_coordinates(x, y, ACTORS_Z)),
            MapPosition { x: 2, y: 3 },
            super::Player,
        ));
        self.add_entity(MapPosition { x, y }, e.id());
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::new(DEFAULT_MAP_WIDTH, DEFAULT_MAP_HEIGHT)
    }
}