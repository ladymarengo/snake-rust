use bevy::{prelude::*, core::FixedTimestep};

const BLOCK_SIZE: f32 = 20.0;
const TIMESTEP_1_PER_SECOND: f64 = 30.0 / 60.0;

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
        .insert_resource(Eaten(false))
        .add_startup_system(start)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIMESTEP_1_PER_SECOND))
                .with_system(head_move.label("head"))
                .with_system(body_move.after("head"))
        )
        .add_system(change_direction)
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

#[derive(Component)]
struct Direction {
    x: f32,
    y: f32,
}

#[derive(Default)]
struct Eaten(bool);

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
        }).insert(Direction{x: 1.0, y: 0.0})
        .id();

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

fn head_move(mut commands: Commands, mut query: Query<(Entity, &mut SnakeHead, &Direction, &mut Transform)>, mut query_body: Query<&mut SnakeBody>) {
    let (head_entity, mut head, direction, mut transform) = query.single_mut();
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
    head_translation.x += BLOCK_SIZE * direction.x;
    head_translation.y += BLOCK_SIZE * direction.y;
}

fn body_move(mut commands: Commands, mut query: Query<(Entity, &SnakeBody), With<Last>>, eaten: Res<Eaten>) {
    if !eaten.0 {
        let (last_entity, last_body) = query.single_mut();
        commands.entity(last_body.next).insert(Last);
        commands.entity(last_entity).despawn();
    }
}

fn change_direction(keys: Res<Input<KeyCode>>, mut query: Query<&mut Direction>) {
    let mut direction = query.single_mut();
    if keys.just_pressed(KeyCode::Left) && direction.x != 1.0 {
        direction.x = -1.0;
        direction.y = 0.0;
    }
    if keys.just_pressed(KeyCode::Right) && direction.x != -1.0 {
        direction.x = 1.0;
        direction.y = 0.0;
    }
    if keys.just_pressed(KeyCode::Up) && direction.y != -1.0 {
        direction.x = 0.0;
        direction.y = 1.0;
    }
    if keys.just_pressed(KeyCode::Down) && direction.y != 1.0 {
        direction.x = 0.0;
        direction.y = -1.0;
    }
}
