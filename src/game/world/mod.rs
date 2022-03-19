use std::{io::Write, sync::{RwLock, Arc, RwLockReadGuard}};

use enum_iterator::IntoEnumIterator;
use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPosition {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}
impl ChunkPosition {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BlockPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
impl BlockPosition {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
    pub fn offset(&self, face: Facing) -> Self {
        let mut clone = *self;
        match face {
            Facing::Front => clone.z -= 1,
            Facing::Back => clone.z += 1,
            Facing::Bottom => clone.y -= 1,
            Facing::Top => clone.y += 1,
            Facing::Right => clone.x -= 1,
            Facing::Left => clone.x += 1,
        }
        clone
    }
    pub fn to_chunk(&self) -> ChunkPosition {
        ChunkPosition::new((self.x >> 4) as usize, (self.y >> 4) as usize, (self.z >> 4) as usize)
    }
}
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash, FromPrimitive, ToPrimitive, IntoEnumIterator)]
pub enum Facing {
    Front,
    Back,
    Bottom,
    Top,
    Right,
    Left,
}
#[derive(Clone, Copy)]
pub struct Block {
    pub id: u8,
    pub position: BlockPosition,
}
impl Block {
    pub fn new(id: u8, position: BlockPosition) -> Self {
        Self {
            id,
            position
        }
    }
}
#[derive(Clone)]
pub struct World {
    pub data: Arc<RwLock<Box<[u8]>>>, // XZY
    width: usize,
    height: usize,
    length: usize,
}

impl World {
    pub fn from_data(data: Vec<u8>, width: i16, height: i16, length: i16) -> Self {
        Self {
            data: Arc::new(RwLock::new(data.into_boxed_slice())),
            width: width as usize,
            height: height as usize,
            length: length as usize,
        }
    }
    pub fn from_file(file_path: &str) -> Option<World> {
        use flate2::read::GzDecoder;
        use nbt::decode::read_compound_tag;
        let cursor = std::fs::File::open(file_path).ok()?;
        let mut cursor = GzDecoder::new(cursor);
        let root_tag = read_compound_tag(&mut cursor).ok()?;
        let mut world = root_tag.get_i8_vec("BlockArray").ok()?.to_owned();
        let width = root_tag.get_i16("X").ok()? as usize;
        let height = root_tag.get_i16("Y").ok()? as usize;
        let length = root_tag.get_i16("Z").ok()? as usize;
        let spawn = root_tag.get_compound_tag("Spawn").ok()?;
        let spawn_x = spawn.get_i16("X").ok()?;
        let spawn_y = spawn.get_i16("Y").ok()?;
        let spawn_z = spawn.get_i16("Z").ok()?;
        let mut newworld: Vec<u8> = vec![];
        let mut world = std::mem::ManuallyDrop::new(world);
        unsafe {
            let ptr = world.as_mut_ptr();
            let len = world.len();
            let cap = world.capacity();
            newworld = Vec::from_raw_parts(ptr as *mut u8, len, cap);
        }
        /*     for i in 0..world.len() {
          newworld.push(world[i] as u8);
        } */
        let size = width * height * length;
        let mut data = vec![0; 4];
        data[0] = (size >> 24) as u8;
        data[1] = (size >> 16) as u8;
        data[2] = (size >> 8) as u8;
        data[3] = size as u8;
        data.append(&mut newworld);
        //let data = data.into_boxed_slice();
        Some(Self {
            data: Arc::new(RwLock::new(data.into_boxed_slice())),
            width,
            height,
            length,
        })
    }

    pub fn new(generator: impl WorldGenerator, width: usize, height: usize, length: usize) -> Self {
        let size = width * height * length;
        let mut data = vec![0; size + 4].into_boxed_slice();
        // Big-endian length of blocks array number
        // TODO use bytes to simplify into .put_be_i32() or something
        data[0] = (size >> 24) as u8;
        data[1] = (size >> 16) as u8;
        data[2] = (size >> 8) as u8;
        data[3] = size as u8;
        generator.generate(&mut data[4..], width, height, length);
        Self {
            data: Arc::new(RwLock::new(data)),
            width,
            height,
            length,
        }
    }

    pub fn pos_to_index(&self, x: usize, y: usize, z: usize) -> usize {
        if x > self.width || y > self.height || z > self.length {
            // TODO bad
            return 0;
        }
        (z + y * self.length) * self.width + x + 4
    }

    // TODO position struct type stuff
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> u8 {
        if x > self.width || y > self.height || z > self.length {
            return 0;
        }
        self.data.read().unwrap().get(self.pos_to_index(x, y, z)).copied().unwrap_or(0)
    }

    pub fn set_block(&mut self, block: Block) {
        let mut data = self.data.write().unwrap();
        data[self.pos_to_index(
            block.position.x as usize,
            block.position.y as usize,
            block.position.z as usize,
        )] = block.id;
    }

    // pub fn data(&self) -> &[u8] {
    //     &self.data
    // }

    // pub fn data_mut(&mut self) -> &mut [u8] {
    //     &mut self.data
    // }

    pub fn width(&self) -> usize {
        self.width
    }
    
    pub fn height(&self) -> usize {
        self.height
    }

    pub fn length(&self) -> usize {
        self.length
    }
}

pub trait WorldGenerator {
    fn generate(&self, data: &mut [u8], width: usize, height: usize, length: usize);
}

pub struct FlatWorldGenerator {
    height: usize,
    below: u8,
    surface: u8,
    above: u8,
}

impl FlatWorldGenerator {
    pub fn new(height: usize, below: u8, surface: u8, above: u8) -> Self {
        Self {
            height,
            below,
            surface,
            above,
        }
    }
}

impl WorldGenerator for FlatWorldGenerator {
    fn generate(&self, data: &mut [u8], width: usize, height: usize, length: usize) {
        let area = width * length;
        for y in 0..height {
            let yi = area * y;
            if y < self.height - 1 {
                for i in 0..area {
                    data[yi + i] = self.below;
                }
            } else if y < self.height {
                for i in 0..area {
                    data[yi + i] = self.surface;
                }
            } else {
                for i in 0..area {
                    data[yi + i] = self.above;
                }
            }
        }
    }
}
