use bevy::{
    prelude::*,
    sprite::{
        MaterialMesh2dBundle,
        collide_aabb::{collide, Collision}
    },
    time::FixedTimestep
};

const WINDOW_WIDTH: f32 = 750.0;
const WINDOW_HEIGHT: f32 = 450.0;
const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const TIME_STEP: f32 = 1.0 / 60.0;
const GAP: f32 = 30.0;

const WALL_COLOR: Color = Color::rgb(0.5, 0.5, 0.5);
const WALL_THINKNESS: f32 = 10.0;
const LEFT_WALL: f32   = -((WINDOW_WIDTH / 2.0) - GAP);
const RIGHT_WALL: f32  = (WINDOW_WIDTH / 2.0) - GAP;
const TOP_WALL: f32    = (WINDOW_HEIGHT / 2.0) - GAP;
const BOTTOM_WALL: f32 = -((WINDOW_HEIGHT / 2.0) - GAP);

const PADDLE_SIZE: Vec3 = Vec3::new(10.0, 100.0, 0.0);

const PADDLE_LEFT: f32 = LEFT_WALL + GAP;
const PADDLE_LEFT_COLOR: Color = Color::rgb(0.4, 0.4, 0.4);

const PADDLE_RIGHT: f32 = RIGHT_WALL - GAP;
const PADDLE_RIGHT_COLOR: Color = Color::rgb(0.3, 0.3, 0.3);

const BALL_SIZE: Vec3 = Vec3::new(10.0, 10.0, 0.0);
const BALL_COLOR: Color = Color::rgb(0.3, 0.3, 0.3);

const SPEED: f32 = 200.0;

#[derive(Default)]
struct CollisionEvent;

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Ball;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct LeftPaddle;

#[derive(Component)]
struct RightPaddle;

#[derive(Component)]
enum WallLocation {
    Left,
    Right,
    Top,
    Bottom
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.0),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.0),
            WallLocation::Top => Vec2::new(0.0, TOP_WALL),
            WallLocation::Bottom => Vec2::new(0.0, BOTTOM_WALL),
        }
    }

    fn size(&self) -> Vec2 {
        let arena_width = RIGHT_WALL - LEFT_WALL;
        let arena_height = TOP_WALL - BOTTOM_WALL;

        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THINKNESS, arena_height + WALL_THINKNESS)
            },
            WallLocation::Top | WallLocation::Bottom => {
                Vec2::new(arena_width + WALL_THINKNESS, WALL_THINKNESS)
            }
        }
    }

    fn side(self) -> Self { self }
}

#[derive(Bundle)]
struct WallBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
    side: WallLocation
}

impl WallBundle {
    fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                transform: Transform {
                    translation: location.position().extend(0.0),
                    scale: location.size().extend(1.0),
                    ..default()
                },
                ..default()
            },
            collider: Collider,
            side: location.side()
        }
    }
}

fn main() {
    App::new()
    .add_plugins(DefaultPlugins
        .set(
            WindowPlugin {
                window: WindowDescriptor {
                    width: WINDOW_WIDTH,
                    height: WINDOW_HEIGHT,
                    title: String::from("MyGame"),
                    ..default()
                },
                ..default()
            }
        )
    )
    .insert_resource(ClearColor(BACKGROUND_COLOR))
    .add_startup_system(setup)
    .add_event::<CollisionEvent>()
    .add_system_set(
        SystemSet::new()
        .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
        .with_system(check_for_collisions)
        .with_system(move_ball.before(check_for_collisions))
        .with_system(move_left_paddle.before(check_for_collisions))
        .with_system(move_right_paddle.before(check_for_collisions))
    )
    .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Walls
    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Right));
    commands.spawn(WallBundle::new(WallLocation::Top));
    commands.spawn(WallBundle::new(WallLocation::Bottom));

    // Left paddle
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: PADDLE_LEFT_COLOR,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(PADDLE_LEFT, 0.0, 0.0),
                scale: PADDLE_SIZE,
                ..default()
            },
            ..default()
        },
        LeftPaddle,
        Collider
    ));

    // Right paddle
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: PADDLE_RIGHT_COLOR,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(PADDLE_RIGHT, 0.0, 0.0),
                scale: PADDLE_SIZE,
                ..default()
            },
            ..default()
        },
        RightPaddle,
        Collider
    ));

    // Ball
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(BALL_COLOR)),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)).with_scale(BALL_SIZE),
            ..default()
        },
        Ball,
        Velocity(Vec2::new(1.0, 1.0)),
        Collider
    ));
}

fn check_for_collisions(
    mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
    collider_query: Query<(&Transform, Option<&WallLocation>), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>
) {
    let (mut ball_velocity, ball_transform) = ball_query.single_mut();

    for (collider_transform, wall_location) in &collider_query {
        let collision = collide(
            ball_transform.translation, ball_transform.scale.truncate(),
            collider_transform.translation, collider_transform.scale.truncate()
        );

        if let Some(collision) = collision {
            collision_events.send_default();

            if wall_location.is_some() {
                match wall_location {
                    Some(WallLocation::Left) => println!("true"),
                    Some(WallLocation::Right) => println!("true"),
                    _ => println!("false"),
                }
            }

            let mut reflect_x = false;
            let mut reflect_y = false;

            match collision {
                Collision::Left => reflect_x = ball_velocity.x > 0.0,
                Collision::Right => reflect_x = ball_velocity.x < 0.0,
                Collision::Top => reflect_y = ball_velocity.y < 0.0,
                Collision::Bottom => reflect_y = ball_velocity.y > 0.0,
                Collision::Inside => {},
            }

            if reflect_x {
                ball_velocity.x *= -1.0;
            }
            if reflect_y {
                ball_velocity.y *= -1.0;
            }
        }
    }
}

fn move_ball(
    mut query: Query<(&mut Transform, &Velocity)>
) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * TIME_STEP * SPEED;
        transform.translation.y += velocity.y * TIME_STEP * SPEED;
    }
}

fn paddle_movement(
    keyboard_input: Res<Input<KeyCode>>,
    transform: &mut Transform,
    up: KeyCode,
    down: KeyCode
) {
    let mut position = 0.0;

    if keyboard_input.pressed(up) { position += 1.0; }
    if keyboard_input.pressed(down) { position -= 1.0; }

    let new_translation = transform.translation.y + position * SPEED * TIME_STEP;

    let top_bound = TOP_WALL - WALL_THINKNESS / 2.0 - PADDLE_SIZE.y / 2.0;
    let bottom_bound = BOTTOM_WALL + WALL_THINKNESS / 2.0 + PADDLE_SIZE.y / 2.0;

    transform.translation.y = new_translation.clamp(bottom_bound, top_bound);    
}

fn move_left_paddle(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<LeftPaddle>>
) {
    let mut transform = query.single_mut();
    paddle_movement(keyboard_input, &mut transform, KeyCode::W, KeyCode::S);
}

fn move_right_paddle(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<RightPaddle>>
) {
    let mut transform = query.single_mut();
    paddle_movement(keyboard_input, &mut transform, KeyCode::Up, KeyCode::Down);
}