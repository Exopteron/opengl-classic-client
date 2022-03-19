use std::path::{Path, PathBuf};

use glam::{Vec2, vec2};

use crate::render::opengl::texture::Texture2D;

use super::world::Facing;

pub struct TerrainAtlas {
    pub texture: Texture2D,
}
impl TerrainAtlas {
    pub fn load_from_file(terrain_png: impl Into<PathBuf>) -> anyhow::Result<Self> {
        unsafe {
            let texture = Texture2D::from_image(image::open(terrain_png.into())?);
            Ok(Self { texture })
        }
    }
    pub fn get_texture(coords: Vec2, face: Facing) -> Vec<Vec2> {
        let div: f32 = 16.;
        let offset = coords;
        let coords;
        match face {
            Facing::Front => {
                coords = vec![
                    (vec2(1., 1.) + offset) / div,
                    (vec2(1., 0.) + offset) / div,
                    (vec2(0., 0.) + offset) / div,
                    (vec2(0., 0.) + offset) / div,
                    (vec2(0., 1.) + offset) / div,
                    (vec2(1., 1.) + offset) / div,
                ]
            },
            Facing::Back => {
                coords = vec![
                    (vec2(0., 1.) + offset) / div,
                    (vec2(1., 0.) + offset) / div,
                    (vec2(0., 0.) + offset) / div,
                    (vec2(1., 0.) + offset) / div,
                    (vec2(0., 1.) + offset) / div,
                    (vec2(1., 1.) + offset) / div,
                ]
            },
            Facing::Bottom => {
                coords = vec![
                    (vec2(1., 0.) + offset) / div,
                    (vec2(0., 1.) + offset) / div,
                    (vec2(1., 1.) + offset) / div,
                    (vec2(0., 1.) + offset) / div,
                    (vec2(1., 0.) + offset) / div,
                    (vec2(0., 0.) + offset) / div,
                ]
            },
            Facing::Top => {
                coords = vec![
                    (vec2(1., 1.) + offset) / div,
                    (vec2(1., 0.) + offset) / div,
                    (vec2(0., 0.) + offset) / div,
                    (vec2(0., 0.) + offset) / div,
                    (vec2(0., 1.) + offset) / div,
                    (vec2(1., 1.) + offset) / div,
                ]
            },
            Facing::Right => {
                coords = vec![
                    (vec2(1., 1.) + offset) / div,
                    (vec2(0., 0.) + offset) / div,
                    (vec2(0., 1.) + offset) / div,
                    (vec2(0., 0.) + offset) / div,
                    (vec2(1., 1.) + offset) / div,
                    (vec2(1., 0.) + offset) / div,
                ]
            },
            Facing::Left => {
                coords = vec![
                    (vec2(0., 1.) + offset) / div,
                    (vec2(1., 1.) + offset) / div,
                    (vec2(1., 0.) + offset) / div,
                    (vec2(1., 0.) + offset) / div,
                    (vec2(0., 0.) + offset) / div,
                    (vec2(0., 1.) + offset) / div,
                ]
            },
        }
        coords
    }
}