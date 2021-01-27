use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize)]
pub enum BlockType {
    Grass = 0,
    Dirt,
    Log,
    Leaves,
    Stone,
    Air,
    Orange,
    Black,
    DarkOrange,
    Water,
    Sand,
    Snow,
    Cactus
}

pub fn index_to_block(index: usize) -> Option<BlockType> {
    match index {
        0 => Some(BlockType::Grass),
        1 => Some(BlockType::Dirt),
        2 => Some(BlockType::Log),
        3 => Some(BlockType::Leaves),
        4 => Some(BlockType::Stone),
        5 => Some(BlockType::Air),
        6 => Some(BlockType::Orange),
        7 => Some(BlockType::Black),
        8 => Some(BlockType::DarkOrange),
        9 => Some(BlockType::Water),
        10 => Some(BlockType::Sand),
        11 => Some(BlockType::Snow),
        12 => Some(BlockType::Cactus),
        _ => None
    }
}