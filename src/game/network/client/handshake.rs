use std::io::{Cursor, Write, Read};

use anyhow::bail;
use flate2::read::GzDecoder;

use crate::game::{network::client::packet::ServerWorldPacket, world::World};

use super::{worker::ClientWorker, packet::{ClientPlayPacket, PlayerIdentification, ServerLoginPacket}};

pub struct ServerDataPackage {
    pub world: World,
}
impl ServerDataPackage {
    pub fn new(world: World) -> Self {
        Self { world }
    }
}

pub async fn do_handshake(worker: &mut ClientWorker) -> anyhow::Result<ServerDataPackage> {
    worker.write(ClientPlayPacket::PlayerIdentification(PlayerIdentification {
        protocol_version: 23,
        username: worker.username.clone(),
        verification_key: String::from(""),
        unused: 0,
    })).await?;
    let p = worker.read::<ServerLoginPacket>().await?;
    log::info!("P {:?}", p);

    let mut world_stage_buf = Vec::new();
    let mut world_size_x: i16;
    let mut world_size_y: i16;
    let mut world_size_z: i16;
    loop  {
        if let Ok(p) = worker.read::<ServerWorldPacket>().await {
            match p {
                ServerWorldPacket::LevelInitialize(_) => log::info!("Recieving world"),
                ServerWorldPacket::LevelDataChunk(data) => {
                    log::info!("Recieved world chunk, percent {}", data.percent_complete);
                    world_stage_buf.extend_from_slice(&data.chunk_data.0[..data.chunk_length as usize]);
                },
                ServerWorldPacket::LevelFinalize(data) => {
                    world_size_x = data.x_size;
                    world_size_y = data.y_size;
                    world_size_z = data.z_size;
                    log::info!("Complete. {:?}", data);
                    break;
                },
            }
        } else {
            bail!("Bad world")
        }
    }
    let cursor = Cursor::new(world_stage_buf);
    let mut cursor = GzDecoder::new(cursor);
    let mut output = Vec::new();
    cursor.read_to_end(&mut output)?;
    let world = World::from_data(output, world_size_x, world_size_y, world_size_z);
    Ok(ServerDataPackage::new(world))
}