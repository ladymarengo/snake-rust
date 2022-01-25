use std::process::exit;
use bevy::{core::FixedTimestep, prelude::*};
use bevy_kira_audio::{Audio, AudioPlugin, AudioSource};
use rand::{prelude::SliceRandom, Rng};

const BLOCK_SIZE: f32 = 30.0;
const TIMESTEP: f64 = 15.0 / 60.0;

struct LoadedSounds(Vec<Handle<AudioSource>>);

/* "App" is a Bevy program. It will put all your game logic together.

All app logic in Bevy uses the Entity Component System paradigm, which
is often shortened to ECS. ECS is a software pattern that involves
breaking your program up into Entities, Components, and Systems.

Entities are unique "things" that are assigned groups of Components,
which are then processed using Systems.

For example, one entity might have a Position and Velocity component,
whereas another entity might have a Position and UI component.

Systems are logic that runs on a specific set of component types.
You might have a movement system that runs on all entities with a
Position and Velocity component.*/

fn main() {
    App::new()
        // Background color
        .insert_resource(ClearColor(Color::rgb(0.9, 0.9, 0.9)))
        // Window params
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
        // This function will be called only once in the beginning
        // and will create a starting position.
        .add_startup_system(start)
        // This function will be called every frame before everything else.
        .add_system_to_stage(CoreStage::PreUpdate, spawn_food)
        // This system of a functions will be called every 0.25 seconds.
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIMESTEP))
                .with_system(head_move.label("head"))
                .with_system(eat_food.after("head").label("eat"))
                .with_system(body_move.after("eat").label("body")),
        )
        // These two functions will also be called every frame.
        .add_system(change_direction)
        .add_system(check_wall)
        .run();
}

// Components and Resources
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

    // Here we create the snake head and one block of the snake body.
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

    // Then we connect them together.
    commands.entity(head).insert(SnakeHead { previous: body });
    commands.entity(body).insert(SnakeBody { next: head });

    // In the end our "head" entity is made of components "SnakeHead",
    // "Direction" and has a pointer to the previous body block.

    // Our "body" entity has "SnakeBody" component and "Last", which means it's a tail.
    // Also it has a pointer to the next block, which is head now.
}

// Loading 4 sounds from the assets folder.
fn load_sounds(mut sounds: ResMut<LoadedSounds>, asset_server: Res<AssetServer>) {
    let loaded_sounds = (1..=4).map(|i| asset_server.load(&format!("nom{i}.ogg")));
    sounds.0.extend(loaded_sounds);
}

// When the snake moves, we don't move all the blocks. Instead we create
// one new block in the beginning and delete the last one. So everything in the middle stays in place.
fn head_move(
    mut commands: Commands,
    mut query: Query<(Entity, &mut SnakeHead, &Direction, &mut Transform)>,
    mut query_body: Query<&mut SnakeBody>,
) {
    
    // Here in arguments we have two queries: first is a query of all Entities that have the components
    // "SnakeHead", "Direction" and "Transform" (this one is the default). Second is a query of all Entities
    // that have a "SnakeBody".
    // We know that the first query has only one member so we use "single".
    let (head_entity, mut head, direction, mut transform) = query.single_mut();
    let head_translation = &mut transform.translation;
    
    // Creating a new SnakeBody block in the same place where the head is.
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
    
    // Change pointers - now both "previous" in the head and "next" in the previous body block point to the new block
    // and "next" in the new block points to the head.
    commands
        .entity(new_body)
        .insert(SnakeBody { next: head_entity });
    query_body.get_mut(head.previous).unwrap().next = new_body;
    head.previous = new_body;

    // Move head in some direction.
    head_translation.x += BLOCK_SIZE * direction.x;
    head_translation.y += BLOCK_SIZE * direction.y;
}

fn check_wall(
    head: Query<&Transform, With<SnakeHead>>,
    query: Query<&Transform, (Without<Food>, Without<SnakeHead>)>,
) {

    // Here we check if the head collides with the walls or with any Entity that isn't a food or the head itself.
    // We exit if it does.
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

    // If the snake ate something, it grows so we don't have to remove the tail.
    // But if it didn't, we remove the last block and pass its "Last" component to the next one.
    if !eaten.0 {
        let (last_entity, last_body) = query.single_mut();
        commands.entity(last_body.next).insert(Last);
        commands.entity(last_entity).despawn();
    }
}

fn change_direction(keys: Res<Input<KeyCode>>, mut query: Query<&mut Direction>) {

    // Change direction based on user's input.
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

    // If we have some food on the board, we check if the head collides with it.
    // If it is, we change the resource "Eaten" to "true" so it will be processed later in the body_move.
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

    // If we don't have any food on the board, we spawn the new one.
    if eaten.0 {
        eaten.0 = false;
        loop {
            // We create the new food block somewhere and check if it collides with the snake.
            // If it is, we delete it and create again until we find an empty spot.
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
