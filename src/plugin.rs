use crate::map::{
    apply_map_transforms, configure_loaded_assets, log_map_events, update_loading_maps,
    update_map_vertex_attributes,
};
use bevy::{prelude::*, sprite::Material2dPlugin};

use crate::{
    map::Map,
    shader::{SHADER_CODE, SHADER_HANDLE},
};

/// Plugin for fast tilemap.
/// Add this to you app and then spawn one or multiple maps use [`crate::map_builder::MapBuilder`].
#[derive(Default)]
pub struct FastTileMapPlugin;

impl Plugin for FastTileMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<Map>::default());
        let mut shaders = app.world.resource_mut::<Assets<Shader>>();
        shaders.insert(SHADER_HANDLE, Shader::from_wgsl(SHADER_CODE, file!()));
        app.add_systems(
            Update,
            (
                (configure_loaded_assets, update_loading_maps, log_map_events).chain(),
                update_map_vertex_attributes,
            ),
        );
        app.add_systems(Update, apply_map_transforms);
    }
}
