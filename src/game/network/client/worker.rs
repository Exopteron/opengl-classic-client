use std::{fmt::{Debug, Write}, io, net::SocketAddr, sync::Arc, time::Duration};

use flume::{Receiver, Sender};
use io::ErrorKind;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    time::timeout,
};

use super::{Writeable, Readable, super::Codec, packet::{ServerPlayPacket, ClientPlayPacket}, handshake::ServerDataPackage};

/// Tokio task which handles a connection and processes
/// packets.
///
/// # Lifecycle
/// * A connection is made, and the `Listener` spawns a `Worker`.
/// * Connection goes through initial handling, i.e., the handshake process.
/// * If the connection was not a status ping, then the main server thread
/// is notified of the new connection via a channel.
pub struct ClientWorker {
    reader: Reader<ServerPlayPacket>,
    writer: Writer<ClientPlayPacket>,
    packets_to_send_tx: Sender<ClientPlayPacket>,
    received_packets_rx: Receiver<ServerPlayPacket>,
    pub username: String,
}

impl ClientWorker {
    pub fn new(
        stream: TcpStream,
        username: String,
    ) -> Self {
        let (reader, writer) = stream.into_split();

        let (received_packets_tx, received_packets_rx) = flume::bounded(32);
        let (packets_to_send_tx, packets_to_send_rx) = flume::unbounded();
        let reader = Reader::new(reader, received_packets_tx);
        let writer = Writer::new(writer, packets_to_send_rx);

        Self {
            username,
            reader,
            writer,
            packets_to_send_tx,
            received_packets_rx,
        }
    }

    pub async fn read<P: Readable>(&mut self) -> anyhow::Result<P> {
        self.reader.read().await
    }

    pub async fn write(&mut self, packet: impl Writeable + Debug) -> anyhow::Result<()> {
        self.writer.write(packet).await
    }

    pub async fn split(mut self) -> (Sender<ClientPlayPacket>, Receiver<ServerPlayPacket>, ServerDataPackage) {
        let p = super::handshake::do_handshake(&mut self).await.unwrap();
        let Self {
            reader,
            writer,
            ..
        } = self;
        let reader = tokio::task::spawn(async move { reader.run().await });
        let writer = tokio::task::spawn(async move { writer.run().await });

        tokio::task::spawn(async move {
            let result = tokio::select! {
                a = reader => a,
                b = writer => b,
            };
            if let Err(e) = result {
                //log::debug!("{} lost connection: {}", username, message);
            }
        });
        (self.packets_to_send_tx, self.received_packets_rx, p)
    }

    pub fn packets_to_send(&self) -> Sender<ClientPlayPacket> {
        self.packets_to_send_tx.clone()
    }

    pub fn received_packets(&self) -> Receiver<ServerPlayPacket> {
        self.received_packets_rx.clone()
    }
}

struct Reader<T: Writeable + Readable + Send + 'static> {
    stream: OwnedReadHalf,
    codec: Codec,
    buffer: [u8; 512],
    received_packets: Sender<T>,
}

impl<T: Writeable + Readable + Send + 'static> Reader<T> {
    pub fn new(stream: OwnedReadHalf, received_packets: Sender<T>) -> Self {
        Self {
            stream,
            codec: Codec::new(),
            buffer: [0; 512],
            received_packets,
        }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        loop {
            let packet = self.read::<T>().await?;
            let result = self.received_packets.send_async(packet).await;
            if result.is_err() {
                // server dropped connection
                return Ok(());
            }
        }
    }

    pub async fn read<P: Readable>(&mut self) -> anyhow::Result<P> {
        // Keep reading bytes and trying to get the packet.
        loop {
            if let Some(packet) = self.codec.next_packet::<P>()? {
                return Ok(packet);
            }

            let duration = Duration::from_secs(10);
            let read_bytes = timeout(duration, self.stream.read(&mut self.buffer)).await??;
            if read_bytes == 0 {
                return Err(io::Error::new(ErrorKind::UnexpectedEof, "read 0 bytes").into());
            }

            let bytes = &self.buffer[..read_bytes];
            self.codec.accept(bytes);
        }
    }
}

struct Writer<T: Writeable + Readable + Send + 'static> {
    stream: OwnedWriteHalf,
    codec: Codec,
    packets_to_send: Receiver<T>,
    buffer: Vec<u8>,
}

impl<T: Writeable + Readable + Send + 'static> Writer<T> {
    pub fn new(stream: OwnedWriteHalf, packets_to_send: Receiver<T>) -> Self {
        Self {
            stream,
            codec: Codec::new(),
            packets_to_send,
            buffer: Vec::new(),
        }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        while let Ok(packet) = self.packets_to_send.recv_async().await {
            self.write(packet).await?;
        }
        Ok(())
    }

    pub async fn write(&mut self, packet: impl Writeable) -> anyhow::Result<()> {
        self.codec.encode(&packet, &mut self.buffer)?;
        self.stream.write_all(&self.buffer).await?;
        self.buffer.clear();
        Ok(())
    }
}

fn disconnected_message(e: anyhow::Error) -> String {
    if let Some(io_error) = e.downcast_ref::<io::Error>() {
        if io_error.kind() == ErrorKind::UnexpectedEof {
            return "disconnected".to_owned();
        }
    }
    format!("{:?}", e)
}
