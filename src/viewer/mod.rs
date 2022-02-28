use crate::MapTiles;
use bevy::prelude::*;
use bevy_polyline::{Polyline, PolylineBundle, PolylineMaterial, PolylinePlugin};

mod camera;

use camera::{spawn_camera, update_camera};

#[derive(Component)]
struct DrawAxes(bool);

fn draw_axes(
    mut commands: Commands,
    draw_axes: Res<DrawAxes>,
    mut polylines: ResMut<Assets<Polyline>>,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
) {
    let axes = vec![
        (
            Vec3::new(1000.0, 0.0, 0.0),
            Color::Rgba {
                red: 1.0,
                green: 0.0,
                blue: 0.0,
                alpha: 0.5,
            },
        ),
        (
            Vec3::new(0.0, 1000.0, 0.0),
            Color::Rgba {
                red: 0.0,
                green: 1.0,
                blue: 0.0,
                alpha: 0.5,
            },
        ),
        (
            Vec3::new(0.0, 0.0, 1000.0),
            Color::Rgba {
                red: 0.0,
                green: 0.0,
                blue: 1.0,
                alpha: 0.5,
            },
        ),
    ];

    if draw_axes.0 {
        for (point, color) in axes {
            commands.spawn_bundle(PolylineBundle {
                polyline: polylines.add(Polyline {
                    vertices: vec![Vec3::new(0.0, 0.0, 0.0), point],
                    ..Default::default()
                }),
                material: polyline_materials.add(PolylineMaterial {
                    width: 25.0,
                    color,
                    perspective: true,
                    ..Default::default()
                }),
                ..Default::default()
            });
        }
    }
}

fn draw_map_tiles(
    map_tiles: Res<MapTiles>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let zoom = map_tiles.zoom;

    for (i, x) in (map_tiles.x_tile_min..=map_tiles.x_tile_max).enumerate() {
        for (j, y) in (map_tiles.y_tile_min..=map_tiles.y_tile_max).enumerate() {
            let tile_file = format!("tiles/tile_{zoom}_{x}_{y}.png");

            let texture_handle = asset_server.load(&tile_file);

            let quad_width = 2.560;
            let quad_height = quad_width;
            let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
                quad_width,
                quad_height,
            ))));

            let material_handle = materials.add(StandardMaterial {
                base_color_texture: Some(texture_handle.clone()),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..Default::default()
            });

            let nx = map_tiles.as_ref().get_nx() as f32;
            let ny = map_tiles.as_ref().get_ny() as f32;

            let i = i as f32;
            let j = j as f32;

            let x = (-0.5 * nx + 0.5 + i) * quad_width;
            let z = (0.5 * ny - 0.5 - j) * quad_height;
            let y = -0.1;

            commands.spawn_bundle(PbrBundle {
                mesh: quad_handle.clone(),
                material: material_handle,
                transform: Transform {
                    translation: Vec3::new(x, y, -z),
                    rotation: Quat::from_rotation_x(-std::f32::consts::PI / 2.0),
                    ..Default::default()
                },
                ..Default::default()
            });
        }
    }
}

fn draw_polylines(
    mut commands: Commands,
    mut polylines: ResMut<Assets<Polyline>>,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
    activities: Res<Vec<Vec<Vec3>>>,
) {
    for activitie in activities.iter() {
        commands.spawn_bundle(PolylineBundle {
            polyline: polylines.add(Polyline {
                vertices: activitie.to_vec(),
                ..Default::default()
            }),
            material: polyline_materials.add(PolylineMaterial {
                width: 16.0,
                color: Color::Rgba {
                    red: 1.0,
                    green: 0.26,
                    blue: 0.0,
                    alpha: 0.5,
                },
                perspective: true,
                ..Default::default()
            }),
            ..Default::default()
        });
    }
}

pub fn run(map_tiles: MapTiles, activities: Vec<Vec<Vec3>>) {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "gpx-heatmap".to_string(),
            width: 1280.,
            height: 720.,
            ..Default::default()
        })
        .insert_resource(map_tiles)
        .insert_resource(activities)
        .insert_resource(DrawAxes(false))
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_plugins(DefaultPlugins)
        .add_plugin(PolylinePlugin)
        .add_startup_system(setup)
        .add_startup_system(draw_axes)
        .add_startup_system(draw_map_tiles)
        .add_startup_system(draw_polylines)
        .add_system(update_camera)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    spawn_camera(commands);
}
