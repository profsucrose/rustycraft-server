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
    Sand
}

pub fn index_to_block(index: usize) -> BlockType {
    match index {
        0 => BlockType::Grass,
        1 => BlockType::Dirt,
        2 => BlockType::Log,
        3 => BlockType::Leaves,
        4 => BlockType::Stone,
        5 => BlockType::Air,
        6 => BlockType::Orange,
        7 => BlockType::Black,
        8 => BlockType::DarkOrange,
        9 => BlockType::Water,
        10 => BlockType::Sand,
        _ => panic!("Attempted to convert index {} to BlockType", index)
    }
}