//! Simple example for illustrating axonometrically projected tilemaps.
//! To keep the math simple instead of strictly isometric, we stick to a projection
//! where each tile ends up a diamond shape that is twice as wide as high.

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::{uvec2, vec2},
    prelude::*,
    window::PresentMode,
};
use bevy_fast_tilemap::{
    bundle::MapBundle, map::MapIndexer, FastTileMapPlugin, Map, MapAttributes, AXONOMETRIC,
};
use rand::Rng;

#[path = "common/mouse_controls_camera.rs"]
mod mouse_controls_camera;
use mouse_controls_camera::MouseControlsCameraPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: String::from("Fast Tilemap example"),
                    resolution: (1820., 920.).into(),
                    // disable vsync so we can see the raw FPS speed
                    present_mode: PresentMode::Immediate,
                    ..default()
                }),
                ..default()
            }),
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
            MouseControlsCameraPlugin::default(),
            FastTileMapPlugin::default(),
        ))
        .add_systems(Startup, startup)
        .add_systems(Update, show_coordinate)
        .run();
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<Map>>,
) {
    commands.spawn(Camera2dBundle::default());

    let map = Map::builder(
        // Map size
        uvec2(100, 100),
        // Tile atlas
        asset_server.load("iso_256x128.png"),
        // Tile size
        vec2(256.0, 128.0),
    )
    .with_padding(vec2(256.0, 128.0), vec2(256.0, 128.0), vec2(256.0, 128.0))
    // "Perspective" overhang draws the overlap of tiles depending on their "depth" that is the
    // y-axis of their world position (tiles higher up are considered further away).
    .with_projection(AXONOMETRIC)
    .with_perspective_overhang()
    .build_and_initialize(init_map);

    commands.spawn(MapBundle {
        material: materials.add(map),
        // Optional: apply a color gradient.
        // MapAttributes define attributes per vertex so they can be changed without
        // triggering re-upload of the map data to the GPU which can conserve performance
        // for large maps
        attributes: MapAttributes {
            mix_color: vec![
                Vec4::new(1.0, 0.0, 0.0, 1.0),
                Vec4::new(0.0, 1.0, 0.0, 1.0),
                Vec4::new(1.0, 1.0, 1.0, 1.0), // color gets multiplied, so this means no change
                Vec4::new(0.0, 0.0, 1.0, 1.0),
            ],
        },
        ..Default::default()
    });
} // startup

/// Fill the map with a random pattern
fn init_map(m: &mut MapIndexer) {
    let mut rng = rand::thread_rng();
    for y in 0..m.size().y {
        for x in 0..m.size().x {
            m.set(x, y, rng.gen_range(1..4));
        }
    }
}

/// Highlight the currently hovered tile red, reset all other tiles
fn show_coordinate(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut camera_query: Query<(&GlobalTransform, &Camera), With<OrthographicProjection>>,
    maps: Query<&Handle<Map>>,
    mut materials: ResMut<Assets<Map>>,
) {
    for event in cursor_moved_events.read() {
        for map_handle in maps.iter() {
            let map = materials.get_mut(map_handle).unwrap();
            for (global, camera) in camera_query.iter_mut() {
                // Translate viewport coordinates to world coordinates
                if let Some(world) = camera
                    .viewport_to_world(global, event.position)
                    .map(|ray| ray.origin.truncate())
                {
                    // The map can convert between world coordinates and map coordinates
                    let coord = map.world_to_map(world);

                    // Convert back to world coordinate to obtain a logical z index ("depth") of
                    // the tile
                    let world2 = map.map_to_world_3d(coord.extend(0.0));
                    println!("Map coordinate: {:?} World-Z: {:?}", coord, world2.z);
                } // if Some(world)
            } // for (global, camera)
        } // for map
    } // for event
} // highlight_hovered
