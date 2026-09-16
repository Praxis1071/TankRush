use bevy::prelude::*;

#[path = "game/mod.rs"]
mod game;

const TANK_SIZE: f32 = 30.0;
const PROJECTILE_SIZE: f32 = 8.0;

#[derive(Component)]
struct TankVisual {
    player_id: game::PlayerId,
}

#[derive(Component)]
struct TankBarrel {
    player_id: game::PlayerId,
}

#[derive(Component)]
struct ProjectileVisual {
    projectile_id: game::ProjectileId,
}

#[derive(Resource)]
struct ArenaState {
    simulation: game::GameSimulation,
}

fn main() {
    let config = game::GameConfig::default();
    let map = game::GameMap::generate(game::MapSize::Medium);
    let mut simulation = game::GameSimulation::new(map.clone(), config);
    let player = simulation
        .state
        .add_player("Player 1", None)
        .expect("player slot available");
    simulation.state.add_tank(player, map.spawn_points[0]);

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.018, 0.024, 0.035)))
        .insert_resource(ArenaState { simulation })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "TankRush".into(),
                resolution: (1280u32, 800u32).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (drive_simulation, sync_tanks, sync_projectiles))
        .run();
}

fn setup(mut commands: Commands, arena: Res<ArenaState>) {
    let map = &arena.simulation.map;

    // Game coordinates start at (0, 0), so center the camera on the arena.
    // This fixes the previous prototype where half of the map could be off-screen.
    commands.spawn((
        Camera2d,
        Transform::from_xyz(map.width * 0.5, map.height * 0.5, 1000.0),
    ));

    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.055, 0.065, 0.08),
            Vec2::new(map.width, map.height),
        ),
        Transform::from_xyz(map.width * 0.5, map.height * 0.5, -10.0),
    ));

    for wall in &map.walls {
        let size = Vec2::new(wall.max.x - wall.min.x, wall.max.y - wall.min.y);
        let center = Vec2::new(
            (wall.min.x + wall.max.x) * 0.5,
            (wall.min.y + wall.max.y) * 0.5,
        );
        commands.spawn((
            Sprite::from_color(Color::srgb(0.12, 0.15, 0.19), size),
            Transform::from_translation(center.extend(0.0)),
        ));
    }

    for tank in &arena.simulation.state.tanks {
        spawn_tank_visual(&mut commands, tank.player_id, tank.position, tank.rotation_radians);
    }
}

fn spawn_tank_visual(
    commands: &mut Commands,
    player_id: game::PlayerId,
    position: game::Vec2,
    rotation: f32,
) {
    commands.spawn((
        Sprite::from_color(player_color(player_id), Vec2::splat(TANK_SIZE)),
        Transform::from_xyz(position.x, position.y, 10.0)
            .with_rotation(Quat::from_rotation_z(rotation)),
        TankVisual { player_id },
    ));
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.78, 0.82, 0.88),
            Vec2::new(TANK_SIZE * 0.85, 6.0),
        ),
        Transform::from_xyz(
            position.x + rotation.cos() * 13.0,
            position.y + rotation.sin() * 13.0,
            11.0,
        )
        .with_rotation(Quat::from_rotation_z(rotation)),
        TankBarrel { player_id },
    ));
}

fn player_color(player_id: game::PlayerId) -> Color {
    match player_id.0 {
        0 => Color::srgb(0.12, 0.58, 1.0),
        1 => Color::srgb(1.0, 0.28, 0.20),
        2 => Color::srgb(0.25, 0.85, 0.40),
        _ => Color::srgb(0.82, 0.35, 1.0),
    }
}

fn drive_simulation(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut arena: ResMut<ArenaState>,
) {
    let input = game::TankInput {
        forward: keyboard.pressed(KeyCode::KeyW),
        backward: keyboard.pressed(KeyCode::KeyS),
        left: keyboard.pressed(KeyCode::KeyA),
        right: keyboard.pressed(KeyCode::KeyD),
        fire: keyboard.just_pressed(KeyCode::Space),
    };

    arena
        .simulation
        .advance(time.delta_secs(), &[(game::PlayerId(0), input)]);
}

fn sync_tanks(
    arena: Res<ArenaState>,
    mut tanks: Query<(&TankVisual, &mut Transform)>,
    mut barrels: Query<(&TankBarrel, &mut Transform), Without<TankVisual>>,
) {
    for (visual, mut transform) in &mut tanks {
        if let Some(tank) = arena
            .simulation
            .state
            .tanks
            .iter()
            .find(|tank| tank.player_id == visual.player_id)
        {
            transform.translation.x = tank.position.x;
            transform.translation.y = tank.position.y;
            transform.rotation = Quat::from_rotation_z(tank.rotation_radians);
            transform.scale = if tank.alive { Vec3::ONE } else { Vec3::ZERO };
        }
    }

    for (barrel, mut transform) in &mut barrels {
        if let Some(tank) = arena
            .simulation
            .state
            .tanks
            .iter()
            .find(|tank| tank.player_id == barrel.player_id)
        {
            let direction = tank.direction();
            transform.translation.x = tank.position.x + direction.x * 13.0;
            transform.translation.y = tank.position.y + direction.y * 13.0;
            transform.rotation = Quat::from_rotation_z(tank.rotation_radians);
            transform.scale = if tank.alive { Vec3::ONE } else { Vec3::ZERO };
        }
    }
}

fn sync_projectiles(
    mut commands: Commands,
    arena: Res<ArenaState>,
    mut projectiles: Query<(Entity, &ProjectileVisual, &mut Transform)>,
) {
    let active_ids = arena
        .simulation
        .state
        .projectiles
        .iter()
        .map(|projectile| projectile.id)
        .collect::<std::collections::HashSet<_>>();

    let mut visual_ids = std::collections::HashSet::new();
    for (entity, visual, mut transform) in &mut projectiles {
        visual_ids.insert(visual.projectile_id);
        if let Some(projectile) = arena
            .simulation
            .state
            .projectiles
            .iter()
            .find(|projectile| projectile.id == visual.projectile_id)
        {
            transform.translation.x = projectile.position.x;
            transform.translation.y = projectile.position.y;
        } else if !active_ids.contains(&visual.projectile_id) {
            commands.entity(entity).despawn();
        }
    }

    for projectile in &arena.simulation.state.projectiles {
        if !visual_ids.contains(&projectile.id) {
            commands.spawn((
                Sprite::from_color(
                    Color::srgb(1.0, 0.82, 0.25),
                    Vec2::splat(PROJECTILE_SIZE),
                ),
                Transform::from_xyz(projectile.position.x, projectile.position.y, 20.0),
                ProjectileVisual {
                    projectile_id: projectile.id,
                },
            ));
        }
    }
}
