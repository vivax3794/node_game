use bevy_prng::WyRand;
use rand::Rng;

use crate::prelude::*;
use crate::{assets, MainState, ZIndex};

const SIZE: f32 = 32.0;
const SCALE: f32 = 2.0;
const TILE_SIZE: f32 = SIZE * SCALE;

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentSize>().add_systems(
            Update,
            (spawn_background, move_background).run_if(in_state(MainState::Playing)),
        );
    }
}

#[derive(Component)]
struct Background;

#[derive(Resource, Default, Debug)]
struct CurrentSize(IVec2);

#[derive(Component)]
struct Tile(IVec2);

fn spawn_background(
    mut commands: Commands,
    misc: Res<assets::Misc>,
    window: Query<&Window, Changed<Window>>,
    backgrounds: Query<Entity, With<Background>>,
    mut size: ResMut<CurrentSize>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
) {
    let Ok(window) = window.get_single() else {
        return;
    };

    let width = window.width();
    let height = window.height();

    let x_amount = (width / TILE_SIZE).ceil() as i32 + 3;
    let y_amount = (height / TILE_SIZE).ceil() as i32 + 3;

    let size_vec = IVec2::new(x_amount, y_amount);
    if size_vec == size.0 {
        return;
    }
    size.0 = size_vec;

    for background in &backgrounds {
        commands.entity(background).despawn_recursive();
    }

    commands
        .spawn((
            Gc(MainState::Playing),
            Background,
            Name::new("Background"),
            SpatialBundle::default(),
        ))
        .with_children(|children| {
            for x_index in 0..x_amount {
                for y_index in 0..y_amount {
                    let x = x_index as f32 * TILE_SIZE - width / 2.;
                    let y = y_index as f32 * TILE_SIZE - height / 2.;
                    children.spawn((
                        SpriteBundle {
                            texture: misc.background.clone(),
                            transform: Transform {
                                translation: Vec3::new(x, y, ZIndex::Background.into()),
                                scale: Vec3::new(SCALE, SCALE, 1.0),
                                rotation: Quat::from_rotation_z(
                                    rng.gen_range(0..4) as f32 * std::f32::consts::FRAC_PI_2,
                                ),
                            },
                            sprite: Sprite {
                                flip_x: rng.gen_bool(0.5),
                                flip_y: rng.gen_bool(0.5),
                                ..default()
                            },
                            ..default()
                        },
                        Name::new("Tile"),
                        Tile(IVec2::new(x_index, y_index)),
                    ));
                }
            }
        });
}

fn move_background(
    camera: Query<&Transform, (With<Camera>, Changed<Transform>, Without<Tile>)>,
    mut tiles: Query<(&mut Tile, &mut Transform)>,
    size: Res<CurrentSize>,
) {
    let Ok(camera_trans) = camera.get_single() else {
        return;
    };

    let x_size = size.0.x as f32 * TILE_SIZE;
    let y_size = size.0.y as f32 * TILE_SIZE;

    let x_bound = x_size / 2.;
    let y_bound = y_size / 2.;

    let camera_pos = camera_trans.translation.truncate();

    for (mut tile, mut trans) in &mut tiles {
        let pos = trans.translation.truncate();
        if pos.x > camera_pos.x + x_bound {
            tile.0.x -= size.0.x;
            trans.translation -= Vec3::new(x_size, 0.0, 0.0);
        } else if pos.x < camera_pos.x - x_bound {
            tile.0.x += size.0.x;
            trans.translation += Vec3::new(x_size, 0.0, 0.0);
        }
        if pos.y > camera_pos.y + y_bound {
            tile.0.y -= size.0.y;
            trans.translation -= Vec3::new(0.0, y_size, 0.0);
        } else if pos.y < camera_pos.y - y_bound {
            tile.0.y += size.0.y;
            trans.translation += Vec3::new(0.0, y_size, 0.0);
        }
    }
}
