use std::time::Duration;

use bevy::input::common_conditions::input_just_pressed;
use bevy_prng::WyRand;
use rand::Rng;

use crate::node_editor::{NodeEventData, NodeOutputTrigger, SnarlContainer, WorldEvent};
use crate::prelude::*;
use crate::{assets, MainState, PlayingState, ZIndex};

const BULLET_SPEED: f32 = 500.0;
const PLAYER_SPEED: f32 = 300.0;
const CAMERA_DISTANCE: f32 = 200.0;
const CAMERA_MAX_SPEED: f32 = 500.0;
const CAMERA_ACCELERATION: f32 = 2000.0;

pub struct GamePlayPlugin;

impl Plugin for GamePlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MainState::Playing), (spawn_player, spawn_camera))
            .init_resource::<CursorLocation>()
            .add_systems(OnEnter(PlayingState::ShootyTime), set_cursor_visibility)
            .add_systems(OnEnter(PlayingState::Editor), set_cursor_visibility)
            .add_systems(
                Update,
                (
                    spawn_bullet,
                    move_bullets,
                    do_timer_despawning,
                    do_animation,
                    (move_player, set_camera_speed, move_camera).chain(),
                    set_player_animation,
                    update_cursor_location,
                    (
                        shoot_action.run_if(input_just_pressed(MouseButton::Left)),
                        move_custom_cursor,
                    )
                        .after(update_cursor_location),
                )
                    .run_if(in_state(PlayingState::ShootyTime)),
            );
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct MovingDirection(Vec2);

#[derive(Component)]
struct Animation {
    row: usize,
    col_size: usize,
    anim_lenth: usize,
    timer: Timer,
}

fn spawn_player(mut commands: Commands, assets: Res<assets::Player>) {
    commands.spawn((
        Gc(MainState::Playing),
        Player,
        SpriteBundle {
            texture: assets.sprite.clone(),
            transform: Transform::from_scale(Vec3::new(4.0, 4.0, 1.0)).with_translation(Vec3::new(
                0.0,
                0.0,
                ZIndex::Player.into(),
            )),
            ..default()
        },
        TextureAtlas {
            layout: assets.layout.clone(),
            ..default()
        },
        Animation {
            row: 0,
            col_size: 8,
            anim_lenth: 8,
            timer: Timer::new(Duration::from_millis(150), TimerMode::Repeating),
        },
        MovingDirection(Vec2::ZERO),
        Name::new("Player"),
    ));
    commands.spawn((
        Gc(MainState::Playing),
        SpriteBundle {
            texture: assets.cursor.clone(),
            ..default()
        },
        CustomCursor,
    ));
}

#[derive(Component)]
struct CustomCursor;

fn move_custom_cursor(
    mut cursor: Query<&mut Transform, With<CustomCursor>>,
    cursor_location: Res<CursorLocation>,
) {
    let Ok(mut trans) = cursor.get_single_mut() else {
        return;
    };

    trans.translation = cursor_location.0.extend(ZIndex::Cursor.into());
}

fn do_animation(mut query: Query<(&mut Animation, &mut TextureAtlas)>, time: Res<Time>) {
    for (mut animation, mut atlas) in &mut query {
        let mut index = atlas
            .index
            // when the row is changed this could overlfow
            .saturating_sub(animation.row * animation.col_size);
        if animation.timer.tick(time.delta()).finished() {
            index += 1;
        }
        index %= animation.anim_lenth;
        atlas.index = index + animation.row * animation.col_size;
    }
}

fn set_cursor_visibility(mut query: Query<&mut Window>, state: Res<State<PlayingState>>) {
    let Ok(mut window) = query.get_single_mut() else {
        return;
    };

    match **state {
        PlayingState::None => {}
        PlayingState::Editor => {
            window.cursor.visible = true;
        }
        PlayingState::ShootyTime => {
            window.cursor.visible = false;
        }
    }
}

fn move_player(
    mut query: Query<(&mut Transform, &mut MovingDirection), With<Player>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let Ok((mut trans, mut moving)) = query.get_single_mut() else {
        return;
    };

    let dir = Vec2::new(
        Into::<f32>::into(keyboard.pressed(KeyCode::KeyD))
            - Into::<f32>::into(keyboard.pressed(KeyCode::KeyA)),
        Into::<f32>::into(keyboard.pressed(KeyCode::KeyW))
            - Into::<f32>::into(keyboard.pressed(KeyCode::KeyS)),
    )
    .normalize_or_zero();

    let delta = dir * PLAYER_SPEED * time.delta_seconds();
    trans.translation += delta.extend(0.0);

    moving.0 = dir;
}

fn set_player_animation(
    mut query: Query<(&mut Animation, &mut Sprite, &MovingDirection), Changed<MovingDirection>>,
) {
    let Ok((mut animation, mut sprite, moving)) = query.get_single_mut() else {
        return;
    };

    if moving.0 == Vec2::ZERO {
        animation.row = 0;
        animation.anim_lenth = 8;
    } else {
        animation.row = 1;
        animation.anim_lenth = 4;

        if moving.0.x > 0. {
            sprite.flip_x = true;
        } else if moving.0.x < 0. {
            sprite.flip_x = false;
        }
    }
}

#[derive(Resource, Default)]
struct CursorLocation(Vec2);

fn update_cursor_location(
    mut cursor_location: ResMut<CursorLocation>,
    query_window: Query<&Window>,
    query_camera: Query<(&Camera, &GlobalTransform)>,
) {
    let Ok(window) = query_window.get_single() else {
        return;
    };
    let Ok((camera, camera_loc)) = query_camera.get_single() else {
        return;
    };

    if let Some(window_space) = window.cursor_position() {
        let Some(world_space) = camera.viewport_to_world_2d(camera_loc, window_space) else {
            return;
        };

        cursor_location.0 = world_space;
    }
}

fn shoot_action(
    cursor_location: Res<CursorLocation>,
    query_player: Query<&GlobalTransform, With<Player>>,
    mut node_trigger: EventWriter<NodeOutputTrigger>,
    snarl: Res<SnarlContainer>,
) {
    let Ok(player_trans) = query_player.get_single() else {
        return;
    };
    let loc = player_trans.translation().truncate();
    let dir = (cursor_location.0 - loc).try_normalize().unwrap_or(Vec2::X);

    let data = NodeEventData {
        loc: Some(loc),
        dir: Some(dir),
        ..default()
    };
    let event = NodeOutputTrigger {
        data,
        node: snarl.shoot_trigger,
        output_index: 0,
    };
    node_trigger.send(event);
}

#[derive(Component)]
struct SourceNode(egui_snarl::NodeId);

#[derive(Component)]
struct Bullet {
    dir: Vec2,
    lifetime: Timer,
}

fn spawn_bullet(
    mut commands: Commands,
    mut events: EventReader<WorldEvent>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
    assets: Res<assets::Bullet>,
) {
    for event in events.read() {
        if let WorldEvent::SpawnBullet {
            loc: Some(loc),
            dir,
            id,
        } = event
        {
            let dir = dir.unwrap_or_else(|| {
                Vec2::from_angle(rng.gen_range(0.0..(std::f32::consts::PI * 2.)))
            });

            commands.spawn((
                Gc(MainState::Playing),
                Bullet {
                    dir,
                    lifetime: Timer::new(Duration::from_secs(1), TimerMode::Once),
                },
                SourceNode(*id),
                SpriteBundle {
                    texture: assets.sprite.clone_weak(),
                    transform: Transform {
                        translation: loc.extend(ZIndex::Bullet.into()),
                        scale: Vec3::new(2.0, 2.0, 1.0),
                        rotation: Quat::from_rotation_z(
                            dir.to_angle() - std::f32::consts::FRAC_PI_4,
                        ),
                    },
                    sprite: Sprite {
                        flip_x: true,
                        ..default()
                    },
                    ..default()
                },
                TextureAtlas {
                    layout: assets.layout.clone(),
                    index: 0,
                },
                Animation {
                    row: 0,
                    col_size: 4,
                    anim_lenth: 4,
                    timer: Timer::new(Duration::from_millis(100), TimerMode::Repeating),
                },
                Fill::color(Color::YELLOW),
                Stroke::new(Color::YELLOW_GREEN, 1.5),
                Name::new("Bullet"),
            ));
        }
    }
}

fn move_bullets(mut query: Query<(&Bullet, &mut Transform)>, time: Res<Time>) {
    for (bullet, mut trans) in &mut query {
        trans.translation += bullet.dir.extend(0.0) * BULLET_SPEED * time.delta_seconds();
    }
}

fn do_timer_despawning(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Bullet, &SourceNode, &GlobalTransform)>,
    time: Res<Time>,
    mut events: EventWriter<NodeOutputTrigger>,
) {
    for (entity_id, mut bullet, node, trans) in &mut query {
        if bullet.lifetime.tick(time.delta()).finished() {
            let data = NodeEventData {
                loc: Some(trans.translation().truncate()),
                ..default()
            };
            events.send(NodeOutputTrigger {
                data,
                node: node.0,
                output_index: 1,
            });
            commands.entity(entity_id).despawn_recursive();
        }
    }
}

#[derive(Component)]
struct CameraSpeed(Vec2);

#[derive(Component)]
struct CameraRelative(Vec2);

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle::default())
        .insert(CameraSpeed(Vec2::ZERO))
        .insert(CameraRelative(Vec2::ZERO));
}

fn move_camera(
    mut camera: Query<(&mut Transform, &mut CameraRelative, &CameraSpeed), Without<Player>>,
    player: Query<&Transform, With<Player>>,
    time: Res<Time>,
) {
    let Ok(player_trans) = player.get_single() else {
        return;
    };

    for (mut trans, mut relative, speed) in &mut camera {
        relative.0 += speed.0 * time.delta_seconds();
        trans.translation =
            (player_trans.translation.truncate() + relative.0).extend(trans.translation.z);
    }
}

fn set_camera_speed(
    mut camera: Query<(&CameraRelative, &mut CameraSpeed)>,
    player: Query<&MovingDirection, With<Player>>,
    time: Res<Time>,
) {
    let Ok(player_moving) = player.get_single() else {
        return;
    };
    let Ok((camera_trans, mut camera_speed)) = camera.get_single_mut() else {
        return;
    };

    let target = player_moving.0 * CAMERA_DISTANCE;
    let delta = target - camera_trans.0;

    if delta.length() < 50.0 {
        let ideal_speed = delta.length() / 2.0;
        let factor = camera_speed.0.length() / ideal_speed;

        if factor.is_normal() {
            camera_speed.0 /= 1. + factor * time.delta_seconds();
            return;
        }
    }

    let delta = delta.normalize_or_zero();

    camera_speed.0 =
        Vec2::from_angle(delta.to_angle()).normalize_or_zero() * camera_speed.0.length();
    camera_speed.0 += delta * CAMERA_ACCELERATION * time.delta_seconds();
    camera_speed.0 = camera_speed.0.clamp_length_max(CAMERA_MAX_SPEED);
}
