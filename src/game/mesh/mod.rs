use std::sync::Arc;

use ahash::AHashMap;
use flume::{Sender, Receiver};
use glam::{vec3, Mat4, Vec3, Vec4, Vec2};

use crate::render::opengl::buffer::VertexBuffer;

use self::database::BlockMeshDatabase;

use super::world::{BlockPosition, ChunkPosition, Facing, World};

pub mod database;

#[derive(Debug, Clone)]
pub struct BlockMesh {
    vertices: Vec<Vec4>,
    texcoords: Vec<Vec2>,
}
impl BlockMesh {
    pub fn new(vertices: Vec<Vec4>, texcoords: Vec<Vec2>) -> Self {
        Self { vertices, texcoords }
    }
    pub fn facing(a: Vec4, b: Vec4, c: Vec4) -> Facing {
        let x = -Self::surface_normal(a, b, c).normalize();
        Self::vec_facing(x)
    }
    pub fn matrix(&mut self, matrix: Mat4) {
        for v in self.vertices.iter_mut() {
            *v = matrix * *v;
        }
    }
    pub fn triangle_retain(&mut self, mut f: impl FnMut(Vec4, Vec4, Vec4) -> bool) {
        let mut i = 0;
        let mut output = Vec::new();
        let mut output_t = Vec::new();
        while i < self.vertices.len() {
            if f(self.vertices[i], self.vertices[i + 1], self.vertices[i + 2]) {
                output.push(self.vertices[i]);
                output.push(self.vertices[i + 1]);
                output.push(self.vertices[i + 2]);

                output_t.push(self.texcoords[i]);
                output_t.push(self.texcoords[i + 1]);
                output_t.push(self.texcoords[i + 2]);
            }
            i += 3;
        }
        self.vertices = output;
        self.texcoords = output_t;
    }
    pub fn vec_facing(v: Vec3) -> Facing {
        if v.x > 0. {
            return Facing::Left;
        }
        if v.x < 0. {
            return Facing::Right;
        }
        if v.y > 0. {
            return Facing::Top;
        }
        if v.y < 0. {
            return Facing::Bottom;
        }
        if v.z > 0. {
            return Facing::Back;
        }
        if v.z < 0. {
            return Facing::Front;
        }
        panic!("AA")
    }
    pub fn surface_normal(a: Vec4, b: Vec4, c: Vec4) -> Vec3 {
        let u = b - a;
        let v = c - a;

        let x = (u.y * v.z) - (u.z * v.y);
        let y = (u.z * v.x) - (u.x * v.z);
        let z = (u.x * v.y) - (u.y * v.x);
        Vec3::new(x, y, z)
    }
}

#[derive(Debug, Default)]
pub struct ChunkMesh {
    pub blocks: [[[Option<BlockMesh>; 16]; 16]; 16],
}
pub enum MeshBuilderTask {
    BuildChunk { world: World, position: ChunkPosition, sender: Sender<(ChunkPosition, ChunkMesh)>  }
}

pub struct MeshBuilder {
    database: Arc<BlockMeshDatabase>,
    meshes: AHashMap<ChunkPosition, ChunkMesh>,
    task_sender: Sender<MeshBuilderTask>,
    tasks: Vec<Receiver<(ChunkPosition, ChunkMesh)>>,
    recieved: usize,
}
impl Default for MeshBuilder {
    fn default() -> Self {
        let (send, recv) = flume::unbounded();
        let db = Arc::new(BlockMeshDatabase::default());
        for _ in 0..16 {
            let recv = recv.clone();
            let db = db.clone();
            rayon::spawn(move || {
                while let Ok(m) = recv.recv() {
                    if let MeshBuilderTask::BuildChunk { world, position, sender } = m {
                        let world_data = world.data.read().unwrap();
                        let mut chunk_mesh = ChunkMesh::default();
                        let x = position.x << 4;
                        let y = position.y << 4;
                        let z = position.z << 4;
                        for x in x..x + 16 {
                            for y in y..y + 16 {
                                for z in z..z + 16 {
                                    let pos = BlockPosition::new(x as i32, y as i32, z as i32);
                                    let block = world_data.get(world.pos_to_index(pos.x as usize, pos.y as usize, pos.z as usize)).copied().unwrap_or(0);
                                    if block != 0 {
                                        let mut mesh = db.get(block);
                                        mesh.matrix(Mat4::from_translation(vec3(x as f32, y as f32, z as f32) * 0.5));
                                        mesh.triangle_retain(|a, b, c| {
                                            let f = BlockMesh::facing(a, b, c);
                                            let p = pos.offset(f);
                                            //panic!("Offset {:?}", p);
                                            let block = world.get_block(p.x as usize, p.y as usize, p.z as usize);
                                            let f = block == 0;
                                            f
                                        });
                                        chunk_mesh.blocks[(x % 16) as usize][(y % 16) as usize][(z % 16) as usize] = Some(mesh);
                                    }
                                }
                            }
                        }
                        sender.send((position, chunk_mesh)).unwrap();
                    }
                }
            });
        }
        Self { database: db, meshes: Default::default(), task_sender: send, tasks: Vec::new(), recieved: 0 }
    }
}
impl MeshBuilder {
    pub fn poll(&mut self) -> bool {
        let len = self.tasks.len();
        if len > 0 {
            let d = if len >= 16 {
                self.tasks.drain(..16)
            } else {
                self.tasks.drain(..)
            };
            let mut tasks = d.collect::<Vec<Receiver<(ChunkPosition, ChunkMesh)>>>();
            tasks.retain(|v| {
                if let Ok((pos, mesh)) = v.try_recv() {
                    self.meshes.insert(pos, mesh);        
                    self.recieved += 1;
                    false
                } else {
                    true
                }
            });
            self.tasks.append(&mut tasks);
            let len = len - self.tasks.len();
            let x = if self.recieved > 32 {
                self.recieved = 0;
                true
            } else {
                false
            };
            x || self.tasks.is_empty()
        } else {
            false
        }
    }
    pub fn build_chunk(&mut self, world: World, position: ChunkPosition) {
        let (send, recv) = flume::unbounded();
        self.task_sender.send(MeshBuilderTask::BuildChunk { world, position, sender: send }).unwrap();
        self.tasks.push(recv);
    }
    pub fn set_data(&self, vertex_buffer: &mut VertexBuffer<Vec4>, texture_buffer: &mut VertexBuffer<Vec2>) {
        let mut mesh_data = Vec::new();
        let mut tex_data = Vec::new();
        for (_, data) in self.meshes.iter() {
            for r in data.blocks.iter() {
                for w in r.iter() {
                    for c in w.iter().flatten() {
                        let mut c = c.clone();
                        mesh_data.append(&mut c.vertices);
                        tex_data.append(&mut c.texcoords);
                    }
                }
            }
        }
        unsafe {
            vertex_buffer.set_data(&mesh_data);
            texture_buffer.set_data(&tex_data);
        }
    }
}
