use crate::{packets, packet_enum, game::network::io::ByteArray};

packets! {
    PlayerIdentification {
        protocol_version u8;
        username String;
        verification_key String;
        unused u8;
    }
    PositionAndOrientation {
        player_id u8;
        x i16;
        y i16;
        z i16;
        yaw u8;
        pitch u8;
    }
}

packets! {
    ServerIdentification {
        protocol_version u8;
        server_name String;
        server_motd String;
        user_type u8;
    }
    LevelInitialize {
        
    }
    Ping {

    }
    LevelDataChunk {
        chunk_length i16;
        chunk_data ByteArray;
        percent_complete u8;
    }
    LevelFinalize {
        x_size i16;
        y_size i16;
        z_size i16;
    }
    Message {
        player_id i8;
        message String;
    }
    SpawnPlayer {
        player_id i8;
        player_name String;
        x i16;
        y i16;
        z i16;
        yaw i8;
        pitch i8;
    }
    PlayerTeleport {
        player_id i8;
        x i16;
        y i16;
        z i16;
        yaw i8;
        pitch i8;
    }
    SetBlock {
        x i16;
        y i16;
        z i16;
        block_type u8;
    }
}

packet_enum!(ClientPlayPacket {
    0x00 = PlayerIdentification,
    0x08 = PositionAndOrientation,
});

packet_enum!(ServerPlayPacket {
    0x01 = Ping,
    0x0D = Message,
    0x07 = SpawnPlayer,
    0x08 = PlayerTeleport,
    0x06 = SetBlock
});

packet_enum!(ServerLoginPacket {
    0x00 = ServerIdentification,
});

packet_enum!(ServerWorldPacket {
    0x02 = LevelInitialize,
    0x03 = LevelDataChunk,
    0x04 = LevelFinalize
});