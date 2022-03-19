use std::any;

use crate::game::{CubeGame, world::{Block, BlockPosition}};

use super::packet::ServerPlayPacket;

pub fn handle_packet(game: &mut CubeGame, packet: ServerPlayPacket) -> anyhow::Result<()> {
    match packet {
        ServerPlayPacket::Ping(_) => (),
        ServerPlayPacket::Message(p) => {
            log::info!("Message {}", p.message);
        },
        ServerPlayPacket::SpawnPlayer(_) => (),
        ServerPlayPacket::PlayerTeleport(packet) => {
            if packet.player_id == -1 {
                let cam = &mut game.camera;
                cam.position.x = (packet.x as f32) / 32.; 
                cam.position.y = (packet.y as f32) / 32.; 
                cam.position.z = (packet.z as f32) / 32.; 
            }
        },
        ServerPlayPacket::SetBlock(packet) => {
            game.set_block(Block::new(packet.block_type, BlockPosition::new(packet.x as i32, packet.y as i32, packet.z as i32)));
        },
    }
    Ok(())
}