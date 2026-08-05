use bevy::log::LogPlugin;
use bevy::prelude::*;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Movement
{
    target: Vec3
}

const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;
const MAP_WIDTH: u32 = 80;
const MAP_HEIGHT: u32 = 40;
const FIELD_SIZE: f32 = 24.0;

fn map_to_screen_coordinates(map_x: u32, map_y: u32) -> Vec3 {
    let screen_x = -(WINDOW_WIDTH as f32 / 2.0) + (map_x as f32 * FIELD_SIZE) + (FIELD_SIZE / 2.0);
    let screen_y = WINDOW_HEIGHT as f32 / 2.0 - (map_y as f32 * FIELD_SIZE) - (FIELD_SIZE / 2.0);
    Vec3::new(screen_x, screen_y, 0.0)
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
    .add_systems(Update, move_player)
	.run();
}

fn setup(mut commands: Commands) {
	commands.spawn(Camera2d);

	commands.spawn((
        Text2d::new("@"),
        TextFont {
            font_size: FontSize::Px(FIELD_SIZE),	
            font: default(),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(map_to_screen_coordinates(2, 3)),
        Player,
    ));

    info!("Player spawned");

    for y in 0..MAP_HEIGHT {
        commands.spawn((
            Text2d::new("#"), 
            TextFont { 
                font_size: FontSize::Px(FIELD_SIZE), 
                font: default(),
                ..default()
                },
                TextColor(Color::WHITE), 
                Transform::from_translation(map_to_screen_coordinates(0, y)),
        ));
        commands.spawn((
            Text2d::new("#"), 
            TextFont { 
                font_size: FontSize::Px(FIELD_SIZE), 
                font: default(),
                ..default()
                },
                TextColor(Color::WHITE), 
                Transform::from_translation(map_to_screen_coordinates(MAP_WIDTH - 1, y)),
        ));
    }
    for x in 1..MAP_WIDTH - 1 {
        commands.spawn((
            Text2d::new("#"), 
            TextFont { 
                font_size: FontSize::Px(FIELD_SIZE), 
                font: default(),
                ..default()
                },
                TextColor(Color::WHITE), 
                Transform::from_translation(map_to_screen_coordinates(x, 0)),
        ));
        commands.spawn((
            Text2d::new("#"), 
            TextFont { 
                font_size: FontSize::Px(FIELD_SIZE), 
                font: default(),
                ..default()
                },
                TextColor(Color::WHITE), 
                Transform::from_translation(map_to_screen_coordinates(x, MAP_HEIGHT - 1)),
        ));
    }
    
}

fn keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Query<Entity, (With<Player>, Without<Movement>)>,
    mut commands: Commands
) {
    for player in query.iter() {
        let mut direction = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }
        if direction != Vec3::ZERO {
            direction *= FIELD_SIZE;
            debug!("Player {player} moving toward {direction:?}");
            let movement = Movement { target: direction };
            commands.entity(player).insert(movement);
        }
    }
}

fn move_player(time: Res<Time>, mut query: Query<(&mut Transform, &mut Movement, Entity), With<Player>>, mut commands: Commands) {
    for (mut transform, mut movement, entity) in query.iter_mut() {
        let speed = 100.0;
        let delta = time.delta_secs();
        let direction = movement.target;
        let displacement = if direction.length_squared()>1.0 {
            direction.normalize_or_zero() * speed * delta
        } else {
            direction
        };
        transform.translation += displacement;
        movement.target -= displacement;
        if movement.target.length_squared() < 0.1 {
            debug!("Player {entity} reached target");
            commands.entity(entity).remove::<Movement>();
        }
    }
}