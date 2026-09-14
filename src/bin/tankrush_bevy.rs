use bevy::prelude::*;

mod game {
    pub use tankrush::game::*;
}

const SCALE: f32 = 1.0;

#[derive(Component)]
struct TankVisual {
    player_id: game::PlayerId,
}

#[derive(Component)]
struct ArenaWallVisual;

#[derive(Resource)]
struct ArenaState {
    map: game::GameMap,
    simulation: game::GameSimulation,
}

fn main() {
    let config = game::GameConfig::default();
    let map = game::GameMap::generate(game::MapSize::Medium);
    let mut simulation = game::GameSimulation::new(config, map.clone());
    let player = simulation
        .state_mut()
        .add_player("Player 1", None)
        .expect("player slot available");
    simulation.state_mut().add_tank(player, map.spawn_points[0]);

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.05)))
        .insert_resource(ArenaState { map, simulation })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "TankRush — Bevy".into(),
                resolution: (1024.0, 768.0).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (drive_simulation, sync_tanks))
        .run();
}

fn setup(mut commands: Commands, arena: Res<ArenaState>) {
    commands.spawn(Camera2d);

    for wall in &arena.map.walls {
        let size = Vec2::new(wall.max.x - wall.min.x, wall.max.y - wall.min.y) * SCALE;
        let center = Vec2::new(
            (wall.min.x + wall.max.x) * 0.5,
            (wall.min.y + wall.max.y) * 0.5,
        ) * SCALE;
        commands.spawn((
            Sprite::from_color(Color::srgb(0.10, 0.12, 0.16), size),
            Transform::from_translation(center.extend(0.0)),
            ArenaWallVisual,
        ));
    }

    for tank in &arena.simulation.state().tanks {
        commands.spawn((
            Sprite::from_color(Color::srgb(0.12, 0.55, 0.95), Vec2::new(28.0, 28.0)),
            Transform::from_translation(
                Vec3::new(tank.position.x, tank.position.y, 1.0) * SCALE,
            ),
            TankVisual {
                player_id: tank.player_id,
            },
        ));
    }
}

fn drive_simulation(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut arena: ResMut<ArenaState>,
) {
    let mut input = game::TankInput::default();
    input.forward = keyboard.pressed(KeyCode::KeyW);
    input.backward = keyboard.pressed(KeyCode::KeyS);
    input.turn_left = keyboard.pressed(KeyCode::KeyA);
    input.turn_right = keyboard.pressed(KeyCode::KeyD);
    input.fire = keyboard.just_pressed(KeyCode::Space);

    arena.simulation.set_input(game::PlayerId(0), input);
    arena.simulation.update(time.delta_secs());
}

fn sync_tanks(
    arena: Res<ArenaState>,
    mut tanks: Query<(&TankVisual, &mut Transform)>,
) {
    for (visual, mut transform) in &mut tanks {
        if let Some(tank) = arena
            .simulation
            .state()
            .tanks
            .iter()
            .find(|tank| tank.player_id == visual.player_id)
        {
            transform.translation.x = tank.position.x * SCALE;
            transform.translation.y = tank.position.y * SCALE;
            transform.rotation = Quat::from_rotation_z(tank.rotation_radians);
            transform.scale = if tank.alive { Vec3::ONE } else { Vec3::ZERO };
        }
    }
}
