use serde::{Deserialize, Serialize};

use crate::rustycraft::block_type::BlockType;
use super::direction::Direction;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum RustyCraftMessage {
    Movement { direction: Direction },
    SetName { name: String },
    PlayerMouseMove { x_offset: f32, y_offset: f32 },
    SetBlock { block: BlockType, world_x: i32, world_y: i32, world_z: i32 },
    GetChunks { coords: Vec<(i32, i32)> },
    ChatMessage { content: String },

    // serialized chunk_blocks in the form of Vec<(usize, usize, usize, usize)>
    // stored as string so serialized chunk blocks can be memoized
    ChunkData { chunks: Vec<(i32, i32, String)> },
    Disconnect
}