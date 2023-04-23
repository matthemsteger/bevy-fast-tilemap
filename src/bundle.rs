use crate::map::Map;
use bevy::{
    ecs::system::EntityCommands,
    math::{vec2, ivec2, mat2},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
    sprite::Mesh2dHandle,
};
use std::mem::size_of;

/// Descriptor for creating a fast tilemap bundle
pub struct FastTileMapDescriptor {
    /// Size of the map (in tiles)
    pub map_size: IVec2,
    /// Size of a single tile (in pixels)
    pub tile_size: Vec2,
    /// Images holding the texture atlases, one for each layer of the map.
    /// All atlases must have a tile size of `tile_size` and no padding.
    pub tiles_texture: Handle<Image>,
    pub transform: Transform,

    pub projection: Mat2,
    pub tile_anchor_point: Vec2,
}

impl Default for FastTileMapDescriptor {
    fn default() -> Self {
        Self {
            map_size: ivec2(100, 100),
            tile_size: vec2(16.0, 16.0),
            tiles_texture: default(),
            transform: default(),
            projection: mat2(vec2(1.0, 0.0), vec2(0.0, -1.0)),
            tile_anchor_point: vec2(0.0, 0.0),
        }
    }
}

impl FastTileMapDescriptor {

    pub fn spawn<'a, 'w, 's>(
        self,
        commands: &'a mut Commands<'w, 's>,
        images: &mut ResMut<Assets<Image>>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> EntityCommands<'w, 's, 'a> {

        let mut map_image = Image::new(
            Extent3d {
                width: self.map_size.x as u32,
                height: self.map_size.y as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            vec![0u8; (self.map_size.x * self.map_size.y) as usize * size_of::<u16>()],
            TextureFormat::R16Uint,
        );
        map_image.texture_descriptor.usage =
            TextureUsages::STORAGE_BINDING | TextureUsages::COPY_DST
            | TextureUsages::TEXTURE_BINDING;
        map_image.texture_descriptor.mip_level_count = 1;

        let projection = self.projection;
        let inverse_projection = projection.inverse();

        // In the first step we use a zero offset, it will be corrected later
        let world_offset = Vec2::default();

        let mut map = Map {
            size: self.map_size,
            tile_size: self.tile_size,
            map_texture: images.add(map_image),
            tiles_texture: self.tiles_texture.clone(),
            ready: false,
            projection, inverse_projection,
            world_offset, tile_anchor_point: self.tile_anchor_point
        };

        // Determine the bounding rectangle of the projected map (in order to construct the quad
        // that will hold the texture).
        //
        // There is probably a more elegant way to do this, but this
        // works and is simple enough:
        // 1. save coordinates for all 4 corners
        // 2. take maximum x- and y distances

        let mut low = map.map_to_world(vec2(0.0, 0.0));
        let mut high = low.clone();
        for corner in [
            vec2(map.size.x as f32, 0.0),
            vec2(0.0, map.size.y as f32),
            vec2(map.size.x as f32, map.size.y as f32),
        ] {
            let pos = map.map_to_world(corner);
            low = low.min(pos);
            high = high.max(pos);
        }
        let size = high - low;

        // `map.projection` keeps the map coordinate (0, 0) at the world coordinate (0, 0).
        // However after projection we may want the (0, 0) tile to map to a different position than
        // say the top left corner (eg for an iso projection it might be vertically centered).
        // We use `low` from above to figure out how to correctly translate here.

        map.world_offset = vec2(-0.5, -0.5) * size - low;


        // See bevy_render/src/mesh/shape/mod.rs
        // will generate 3d position, 3d normal, and 2d UVs
        let mesh = Mesh2dHandle(meshes.add(Mesh::from(shape::Quad { size, flip: false, })));

        let bundle = FastTileMapBundle {
            map,
            mesh: mesh.clone(),
            transform: self.transform,
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            computed_visibility: ComputedVisibility::default(),
        };

        commands.spawn(bundle)
    } // fn spawn()
} // impl FastTileMapDescriptor

#[derive(Bundle, Clone)]
pub struct FastTileMapBundle {
    pub mesh: Mesh2dHandle,
    pub map: Map,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}
