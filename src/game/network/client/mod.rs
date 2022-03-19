use std::fmt::Debug;

use flume::{Receiver, Sender};
use tokio::net::TcpStream;

use super::{Readable, Writeable};

use self::{worker::ClientWorker, packet::{ServerPlayPacket, ClientPlayPacket, PositionAndOrientation}, handshake::ServerDataPackage};
pub mod worker;
pub mod handshake;
pub mod packet;
pub mod handle;
pub struct Client {
    pub reader: Receiver<ServerPlayPacket>,
    pub sender: Sender<ClientPlayPacket>,
    pub username: String,
}
impl Client {
    pub async fn connect(addr: &str, username: String) -> anyhow::Result<(Self, ServerDataPackage)> {
        let worker = ClientWorker::new(TcpStream::connect(addr).await?, username.clone());
        let (sender, reciever, package) = worker.split().await;
        Ok((Self {
            reader: reciever,
            sender,
            username
        }, package))
    }
    pub async fn read(&mut self) -> anyhow::Result<ServerPlayPacket> {
        Ok(self.reader.recv_async().await?)
    }
    pub fn write(&mut self, p: ClientPlayPacket) -> anyhow::Result<()> {
        Ok(self.sender.send(p)?)
    }
    pub fn update_position(&mut self, x: f32, y: f32, z: f32, yaw: u8, pitch: u8) {
        self.write(ClientPlayPacket::PositionAndOrientation(PositionAndOrientation {
            player_id: 255,
            x: ((x * 2.) * 32.) as i16,
            y: ((y * 2.) * 32.) as i16,
            z: ((z * 2.) * 32.) as i16,
            yaw,
            pitch,
        })).unwrap();
    }
}