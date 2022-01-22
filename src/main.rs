use bevy::{prelude::*, core::FixedTimestep};

const BLOCK_SIZE: f32 = 20.0;
const TIMESTEP_1_PER_SECOND: f64 = 60.0 / 60.0;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.9, 0.9, 0.9)))
        .insert_resource(WindowDescriptor {
            width: BLOCK_SIZE * 20.0,
            height: BLOCK_SIZE * 20.0,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(start)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIMESTEP_1_PER_SECOND))
                .with_system(head_move)
        )
        .run();
}

#[derive(Component)]
struct SnakeHead {
    previous: Entity,
}

#[derive(Component)]
struct SnakeBody {
    next: Entity,
}

#[derive(Component)]
struct Last;

fn start(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    let head = commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: Vec3::new(BLOCK_SIZE, BLOCK_SIZE, 0.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 1.0),
                ..Default::default()
            },
            ..Default::default()
        }).id();

    let body = commands
    .spawn_bundle(SpriteBundle {
        transform: Transform {
            translation: Vec3::new(0.0 - BLOCK_SIZE, 0.0, 0.0),
            scale: Vec3::new(BLOCK_SIZE, BLOCK_SIZE, 0.0),
            ..Default::default()
        },
        sprite: Sprite {
            color: Color::rgb(1.0, 0.5, 0.5),
            ..Default::default()
        },
        ..Default::default()
    })
        .insert(Last).id();

    commands.entity(head).insert(SnakeHead {previous: body});
    commands.entity(body).insert(SnakeBody {next: head});
}

fn head_move(mut commands: Commands, mut query: Query<(Entity, &mut SnakeHead, &mut Transform)>, mut query_body: Query<&mut SnakeBody>) {
    let (head_entity, mut head, mut transform) = query.single_mut();
    let head_translation = &mut transform.translation;
    let new_body = commands
    .spawn_bundle(SpriteBundle {
        transform: Transform {
            translation: Vec3::new(head_translation.x, head_translation.y, 0.0),
            scale: Vec3::new(BLOCK_SIZE, BLOCK_SIZE, 0.0),
            ..Default::default()
        },
        sprite: Sprite {
            color: Color::rgb(1.0, 0.5, 0.5),
            ..Default::default()
        },
        ..Default::default()
    }).id();
    commands.entity(new_body).insert(SnakeBody {next: head_entity});
    query_body.get_mut(head.previous).unwrap().next = new_body;
    head.previous = new_body;
    head_translation.x += BLOCK_SIZE;
}