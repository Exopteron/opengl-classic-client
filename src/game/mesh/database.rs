use ahash::AHashMap;
use enum_iterator::IntoEnumIterator;
use fnv::FnvHashMap;
use glam::{vec4, Vec4, Vec2, vec2, const_vec2};

use crate::game::{world::Facing, texture::TerrainAtlas};

use super::BlockMesh;

pub struct BlockMeshDatabase {
    map: FnvHashMap<u8, BlockMesh>,
}
impl Default for BlockMeshDatabase {
    fn default() -> Self {
        let mut map = FnvHashMap::default();
        map.insert(1, Self::cube_mesh(&assemble_cube_tex(vec2(1., 0.))));
        map.insert(2, Self::cube_mesh(&[
            (Facing::Top, vec2(0., 0.)),
            (Facing::Bottom, vec2(2., 0.)),

            (Facing::Left, vec2(3., 0.)),
            (Facing::Right, vec2(3., 0.)),
            (Facing::Front, vec2(3., 0.)),
            (Facing::Back, vec2(3., 0.)),
        ]));
        map.insert(17, Self::cube_mesh(&[
            (Facing::Top, vec2(5., 1.)),
            (Facing::Bottom, vec2(5., 1.)),

            (Facing::Left, vec2(4., 1.)),
            (Facing::Right, vec2(4., 1.)),
            (Facing::Front, vec2(4., 1.)),
            (Facing::Back, vec2(4., 1.)),
        ]));
        map.insert(18, Self::cube_mesh(&assemble_cube_tex(vec2(6., 1.))));
        map.insert(3, Self::cube_mesh(&assemble_cube_tex(vec2(2., 0.))));
        map.insert(12, Self::cube_mesh(&assemble_cube_tex(vec2(2., 1.))));

        map.insert(13, Self::cube_mesh(&assemble_cube_tex(vec2(3., 1.))));

        map.insert(8, Self::cube_mesh(&assemble_cube_tex(vec2(14., 0.))));
        map.insert(9, Self::cube_mesh(&assemble_cube_tex(vec2(14., 0.))));

        map.insert(7, Self::cube_mesh(&assemble_cube_tex(vec2(1., 1.))));
        Self { map }
    }
}
impl BlockMeshDatabase {
    pub fn get(&self, idx: u8) -> BlockMesh {
        if let Some(v) = self.map.get(&idx) {
            return v.clone();
        }
        Self::cube_mesh(&assemble_cube_tex(vec2(0., 0.)))
    }
}

impl BlockMeshDatabase {
    pub fn cube_mesh(tex_positions: &[(Facing, Vec2)]) -> BlockMesh {
        const SIZE: f32 = 0.25;
        let mut tex_map = AHashMap::new();
        for (facing, vec) in tex_positions {
            tex_map.insert(*facing, TerrainAtlas::get_texture(*vec, *facing));
        }
        let mut vertices = Vec::new();
        let mut texcoords_out = Vec::new();

        let mut set_coords = Vec::new();
        let faces = [Facing::Front, Facing::Back, Facing::Bottom, Facing::Top, Facing::Right, Facing::Left];
        for f in faces {
            set_coords.append(&mut tex_map.get(&f).unwrap_or(&TerrainAtlas::get_texture(vec2(0., 0.), f)).to_owned());
        }
        // front
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(-SIZE, -SIZE, -SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(-SIZE, SIZE, -SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(SIZE, SIZE, -SIZE, 1.0));

        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(SIZE, SIZE, -SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(SIZE, -SIZE, -SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(-SIZE, -SIZE, -SIZE, 1.0));

        // back
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(-SIZE, -SIZE, SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(SIZE, SIZE, SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(-SIZE, SIZE, SIZE, 1.0));

        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(SIZE, SIZE, SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(-SIZE, -SIZE, SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(SIZE, -SIZE, SIZE, 1.0));

        // bottom
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(-SIZE, -SIZE, -SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(SIZE, -SIZE, SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(-SIZE, -SIZE, SIZE, 1.0));

        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(SIZE, -SIZE, SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(-SIZE, -SIZE, -SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(SIZE, -SIZE, -SIZE, 1.0));

        // top
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(-SIZE, SIZE, -SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(-SIZE, SIZE, SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(SIZE, SIZE, SIZE, 1.0));

        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(SIZE, SIZE, SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(SIZE, SIZE, -SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(-SIZE, SIZE, -SIZE, 1.0));

        // right
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(SIZE, -SIZE, -SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(SIZE, SIZE, SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(SIZE, -SIZE, SIZE, 1.0));

        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(SIZE, SIZE, SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(SIZE, -SIZE, -SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(SIZE, SIZE, -SIZE, 1.0));

        // left
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(-SIZE, -SIZE, -SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(-SIZE, -SIZE, SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(-SIZE, SIZE, SIZE, 1.0));

        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(-SIZE, SIZE, SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(-SIZE, SIZE, -SIZE, 1.0));
        texcoords_out.push(set_coords.pop().unwrap());
        vertices.push(vec4(-SIZE, -SIZE, -SIZE, 1.0));
        vertices.reverse();
        BlockMesh::new(vertices, texcoords_out.to_vec())
    }
}


fn assemble_cube_tex(coords: Vec2) -> Vec<(Facing, Vec2)> {
    let mut v = Vec::new();
    for ele in Facing::into_enum_iter() {
        v.push((ele, coords));
    }
    v
}