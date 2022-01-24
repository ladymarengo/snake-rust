use std::process::exit;
use bevy::{core::FixedTimestep, prelude::*};
use bevy_kira_audio::{Audio, AudioPlugin, AudioSource};
use rand::{prelude::SliceRandom, Rng};

const BLOCK_SIZE: f32 = 30.0;
const TIMESTEP_1_PER_SECOND: f64 = 15.0 / 60.0;

struct LoadedSounds(Vec<Handle<AudioSource>>);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.9, 0.9, 0.9)))
        .insert_resource(WindowDescriptor {
            width: BLOCK_SIZE * 20.0,
            height: BLOCK_SIZE * 20.0,
            title: "Snake".to_string(),
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        // Sounds
        .insert_resource(LoadedSounds(vec![]))
        .add_plugin(AudioPlugin)
        .add_startup_system(load_sounds)
        // Logic
        .insert_resource(Eaten(true))
        .add_startup_system(start)
        .add_system_to_stage(CoreStage::PreUpdate, spawn_food)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIMESTEP_1_PER_SECOND))
                .with_system(head_move.label("head"))
                .with_system(eat_food.after("head").label("eat"))
                .with_system(body_move.after("eat").label("body")),
        )
        .add_system(change_direction)
        .add_system(check_wall)
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

#[derive(Component)]
struct Food;

fn start(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    let head = commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(BLOCK_SIZE / 2.0, BLOCK_SIZE / 2.0, 0.0),
                scale: Vec3::new(BLOCK_SIZE, BLOCK_SIZE, 0.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.65),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Direction { x: 1.0, y: 0.0 })
        .id();

    let body = commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0 - BLOCK_SIZE / 2.0, BLOCK_SIZE / 2.0, 0.0),
                scale: Vec3::new(BLOCK_SIZE, BLOCK_SIZE, 0.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.65),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Last)
        .id();

    commands.entity(head).insert(SnakeHead { previous: body });
    commands.entity(body).insert(SnakeBody { next: head });
}

fn load_sounds(mut sounds: ResMut<LoadedSounds>, asset_server: Res<AssetServer>) {
    let loaded_sounds = (1..=4).map(|i| asset_server.load(&format!("nom{i}.ogg")));
    sounds.0.extend(loaded_sounds);
}

fn head_move(
    mut commands: Commands,
    mut query: Query<(Entity, &mut SnakeHead, &Direction, &mut Transform)>,
    mut query_body: Query<&mut SnakeBody>,
) {
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
                color: Color::rgb(0.25, 0.25, 0.65),
                ..Default::default()
            },
            ..Default::default()
        })
        .id();
    commands
        .entity(new_body)
        .insert(SnakeBody { next: head_entity });
    query_body.get_mut(head.previous).unwrap().next = new_body;
    head.previous = new_body;
    head_translation.x += BLOCK_SIZE * direction.x;
    head_translation.y += BLOCK_SIZE * direction.y;
}

fn check_wall(
    head: Query<&Transform, With<SnakeHead>>,
    query: Query<&Transform, (Without<Food>, Without<SnakeHead>)>,
) {
    let head_translation = head.single().translation;
    if query
        .iter()
        .any(|e| e.translation.distance(head_translation) < 1.0)
        || head_translation.x > BLOCK_SIZE * 10.0
        || head_translation.x <= BLOCK_SIZE * -10.0
        || head_translation.y > BLOCK_SIZE * 10.0
        || head_translation.y <= BLOCK_SIZE * -10.0
    {
        exit(0);
    }
}

fn body_move(
    mut commands: Commands,
    mut query: Query<(Entity, &SnakeBody), With<Last>>,
    eaten: Res<Eaten>,
) {
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

fn eat_food(
    mut commands: Commands,
    snake_query: Query<&Transform, With<SnakeHead>>,
    food_query: Query<(Entity, &Transform), With<Food>>,
    mut eaten: ResMut<Eaten>,
    audio: Res<Audio>,
    sounds: Res<LoadedSounds>,
) {
    let snake = snake_query.single();
    if !food_query.is_empty() {
        let (food_entity, food) = food_query.single();
        if snake.translation.distance(food.translation) < 1.0 {
            audio.play(sounds.0.choose(&mut rand::thread_rng()).unwrap().clone());
            commands.entity(food_entity).despawn();
            eaten.0 = true;
        }
    }
}

fn spawn_food(mut commands: Commands, query: Query<&Transform>, mut eaten: ResMut<Eaten>) {
    if eaten.0 {
        eaten.0 = false;
        loop {
            let mut rng = ::rand::thread_rng();
            let food_translation = Vec3::new(
                BLOCK_SIZE * rng.gen_range(-9..9) as f32 - BLOCK_SIZE / 2.0,
                BLOCK_SIZE * rng.gen_range(-9..9) as f32 - BLOCK_SIZE / 2.0,
                0.0,
            );
            let food = commands
                .spawn_bundle(SpriteBundle {
                    transform: Transform {
                        translation: food_translation,
                        scale: Vec3::new(BLOCK_SIZE, BLOCK_SIZE, 0.0),
                        ..Default::default()
                    },
                    sprite: Sprite {
                        color: Color::rgb(0.7, 0.5, 0.3),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Food)
                .id();

            if !query
                .iter()
                .any(|e| e.translation.distance(food_translation) < 1.0)
            {
                break;
            }
            commands.entity(food).despawn();
        }
    }
}
