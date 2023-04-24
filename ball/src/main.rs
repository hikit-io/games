use bevy::{prelude::*, window::PrimaryWindow};
use rand::random;

pub const PLAYER_SIZE: f32 = 64.0;
pub const PLAYER_SPEED: f32 = 500.0;

pub const NUMBER_OF_ENEMIES: usize = 4;
pub const ENEMY_SPEED: f32 = 200.0;
pub const ENEMY_SIZE: f32 = 64.0;

pub const START_SIZE: f32 = 32.0;
pub const NUMBER_OF_STAR: usize = 10;
pub const STAR_SPAWN_TIME: f32 = 1.0;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Score>()
        .init_resource::<StarSpawnTimer>()
        .init_resource::<EnemySpawnTimer>()
        .add_startup_system(spawn_player)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_enemies)
        .add_startup_system(spawn_star)
        .add_system(player_movement)
        .add_system(confine_player_movement)
        .add_system(enemy_movement)
        .add_system(confine_enemy_movement)
        .add_system(enemy_hit)
        .add_system(player_collect_star)
        .add_system(update_score)
        .add_system(tick_start)
        .add_system(tick_spawn_star)
        .add_system(tick_enemy_timer)
        .add_system(spawn_enemy_over_timer)
        .run();
}

#[derive(Component)]
pub struct Player {}

#[derive(Component)]
pub struct Enemy {
    direction: Vec2,
}

#[derive(Component)]
pub struct Star {}

#[derive(Resource, Default)]
pub struct Score {
    value: u32,
}

#[derive(Resource)]
pub struct StarSpawnTimer {
    timer: Timer,
}

impl Default for StarSpawnTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        }
    }
}

#[derive(Resource)]
pub struct EnemySpawnTimer {
    timer: Timer,
}

impl Default for EnemySpawnTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(5.0, TimerMode::Repeating),
        }
    }
}

pub fn spawn_player(
    mut cmd: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();

    cmd.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
            texture: asset_server.load("sprites/ball_blue_large.png"),
            ..default()
        },
        Player {},
    ));
}

pub fn spawn_camera(mut cmd: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.get_single().unwrap();

    cmd.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..default()
    });
}

pub fn spawn_enemies(
    mut cmd: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();
    for _ in 0..NUMBER_OF_ENEMIES {
        let x = random::<f32>() * window.width();
        let y = random::<f32>() * window.height();
        cmd.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(x, y, 0.0),
                texture: asset_server.load("sprites/ball_red_large.png"),
                ..default()
            },
            Enemy {
                direction: Vec2::new(random(), random()).normalize(),
            },
        ));
    }
}

pub fn spawn_star(
    mut cmd: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();
    for _ in 0..NUMBER_OF_STAR {
        let x = random::<f32>() * window.width();
        let y = random::<f32>() * window.height();

        cmd.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(x, y, 0.0),
                texture: asset_server.load("sprites/star.png"),
                ..default()
            },
            Star {},
        ));
    }
}

pub fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    if let Ok(mut ts) = player_query.get_single_mut() {
        let mut direction = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::W) {
            direction += Vec3::new(0.0, 1.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::S) {
            direction += Vec3::new(0.0, -1.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::A) {
            direction += Vec3::new(-1.0, 0.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::D) {
            direction += Vec3::new(1.0, 0.0, 0.0);
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();
        }

        ts.translation += direction * PLAYER_SPEED * time.delta_seconds();
    }
}

pub fn confine_player_movement(
    mut player_query: Query<&mut Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(mut ts) = player_query.get_single_mut() {
        let window = window_query.get_single().unwrap();
        let half_player_size = PLAYER_SIZE / 2.0;

        let x_min = 0.0 + half_player_size;
        let x_max = window.width() - half_player_size;
        let y_min = 0.0 + half_player_size;
        let y_max = window.height() - half_player_size;

        let mut translation = ts.translation;

        if translation.x < x_min {
            translation.x = x_min
        }
        if translation.x > x_max {
            translation.x = x_max
        }

        if translation.y < y_min {
            translation.y = y_min
        }
        if translation.y > y_max {
            translation.y = y_max
        }

        ts.translation = translation;
    }
}

pub fn enemy_movement(mut enemy_query: Query<(&mut Transform, &Enemy)>, time: Res<Time>) {
    for (mut ts, enemy) in enemy_query.iter_mut() {
        let direction = Vec3::new(enemy.direction.x, enemy.direction.y, 0.0);
        ts.translation += direction * ENEMY_SPEED * time.delta_seconds();
    }
}

pub fn confine_enemy_movement(
    mut enemy_query: Query<(&mut Transform, &mut Enemy)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    let window = window_query.get_single().unwrap();
    let half_player_size = ENEMY_SIZE / 2.0;

    let x_min = 0.0 + half_player_size;
    let x_max = window.width() - half_player_size;
    let y_min = 0.0 + half_player_size;
    let y_max = window.height() - half_player_size;

    for (mut ts, mut enemy) in enemy_query.iter_mut() {
        let mut translation = ts.translation;

        let mut rebound = false;

        if translation.x < x_min {
            translation.x = x_min;
            enemy.direction.x *= -1.0;
            rebound = true;
        }

        if translation.x > x_max {
            translation.x = x_max;
            enemy.direction.x *= -1.0;
            rebound = true;
        }

        if translation.y < y_min {
            translation.y = y_min;
            enemy.direction.y *= -1.0;
            rebound = true;
        }

        if translation.y > y_max {
            translation.y = y_max;
            enemy.direction.y *= -1.0;
            rebound = true;
        }

        if rebound {
            let sound = asset_server.load("audio/pluck_001.ogg");
            audio.play(sound);
        }

        ts.translation = translation;
    }
}

pub fn enemy_hit(
    mut cmd: Commands,
    mut play_query: Query<(Entity, &Transform), With<Player>>,
    enemy_query: Query<&Transform, With<Enemy>>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    if let Ok((player, ts)) = play_query.get_single_mut() {
        for enemy in enemy_query.iter() {
            let distance = ts.translation.distance(enemy.translation);
            if distance <= ENEMY_SIZE / 2.0 + PLAYER_SIZE / 2.0 {
                let sound = asset_server.load("audio/explosionCrunch_000.ogg");
                audio.play(sound);
                cmd.entity(player).despawn();
            }
        }
    }
}

pub fn player_collect_star(
    mut cmd: Commands,
    mut play_query: Query<&Transform, With<Player>>,
    star_query: Query<(Entity, &Transform), With<Star>>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    mut score: ResMut<Score>,
) {
    if let Ok(player) = play_query.get_single_mut() {
        for (star, start_ts) in star_query.iter() {
            let distance = player.translation.distance(start_ts.translation);
            if distance <= START_SIZE / 2.0 + PLAYER_SIZE / 2.0 {
                let sound = asset_server.load("audio/laserLarge_000.ogg");
                audio.play(sound);
                cmd.entity(star).despawn();
                score.value += 1;
            }
        }
    }
}

pub fn update_score(score: Res<Score>) {
    if score.is_changed() {
        println!("Score:{}", score.value);
    }
}

pub fn tick_start(mut timer: ResMut<StarSpawnTimer>, time: Res<Time>) {
    timer.timer.tick(time.delta());
}

pub fn tick_spawn_star(
    timer: Res<StarSpawnTimer>,
    mut cmd: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    if timer.timer.finished() {
        let window = window_query.get_single().unwrap();
        let x = random::<f32>() * window.width();
        let y = random::<f32>() * window.height();

        cmd.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(x, y, 0.0),
                texture: asset_server.load("sprites/star.png"),
                ..default()
            },
            Star {},
        ));
    }
}

pub fn tick_enemy_timer(mut timer: ResMut<EnemySpawnTimer>, time: Res<Time>) {
    timer.timer.tick(time.delta());
}

pub fn spawn_enemy_over_timer(
    mut cmd: Commands,
    timer: Res<EnemySpawnTimer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    if timer.timer.finished() {
        let window = window_query.get_single().unwrap();

        let x = random::<f32>() * window.width();
        let y = random::<f32>() * window.height();
        cmd.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(x, y, 0.0),
                texture: asset_server.load("sprites/ball_red_large.png"),
                ..default()
            },
            Enemy {
                direction: Vec2::new(random(), random()).normalize(),
            },
        ));
    }
}
