use bevy::prelude::*;

const ARENA_SIZE: Vec2 = Vec2::new(960.0, 640.0);
const TANK_SIZE: Vec2 = Vec2::new(34.0, 26.0);
const TANK_SPEED: f32 = 220.0;
const TANK_TURN_SPEED: f32 = 2.8;

#[derive(Component)]
struct PlayerTank;

#[derive(Component)]
struct ArenaWall;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.05)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "TankRush — Bevy Prototype".into(),
                resolution: (ARENA_SIZE.x, ARENA_SIZE.y).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, player_movement)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    spawn_wall(&mut commands, Vec2::new(0.0, ARENA_SIZE.y * 0.5), Vec2::new(ARENA_SIZE.x, 16.0));
    spawn_wall(&mut commands, Vec2::new(0.0, -ARENA_SIZE.y * 0.5), Vec2::new(ARENA_SIZE.x, 16.0));
    spawn_wall(&mut commands, Vec2::new(-ARENA_SIZE.x * 0.5, 0.0), Vec2::new(16.0, ARENA_SIZE.y));
    spawn_wall(&mut commands, Vec2::new(ARENA_SIZE.x * 0.5, 0.0), Vec2::new(16.0, ARENA_SIZE.y));

    commands.spawn((
        Sprite::from_color(Color::srgb(0.12, 0.55, 0.95), TANK_SIZE),
        Transform::from_xyz(-220.0, 0.0, 1.0),
        PlayerTank,
    ));
}

fn spawn_wall(commands: &mut Commands, position: Vec2, size: Vec2) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.10, 0.12, 0.16), size),
        Transform::from_translation(position.extend(0.0)),
        ArenaWall,
    ));
}

fn player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut tanks: Query<&mut Transform, With<PlayerTank>>,
) {
    for mut transform in &mut tanks {
        let turn = axis(&keyboard, KeyCode::KeyD, KeyCode::KeyA);
        let throttle = axis(&keyboard, KeyCode::KeyW, KeyCode::KeyS);

        transform.rotate_z(turn * TANK_TURN_SPEED * time.delta_secs());
        let forward = transform.rotation * Vec3::Y;
        transform.translation += forward * throttle * TANK_SPEED * time.delta_secs();

        let half = ARENA_SIZE * 0.5 - TANK_SIZE * 0.5 - Vec2::splat(8.0);
        transform.translation.x = transform.translation.x.clamp(-half.x, half.x);
        transform.translation.y = transform.translation.y.clamp(-half.y, half.y);
    }
}

fn axis(keyboard: &ButtonInput<KeyCode>, positive: KeyCode, negative: KeyCode) -> f32 {
    f32::from(keyboard.pressed(positive)) - f32::from(keyboard.pressed(negative))
}
