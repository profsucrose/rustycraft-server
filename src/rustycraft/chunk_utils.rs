use std::collections::{HashMap, HashSet};

use crate::rustycraft::{block_map::BlockMap, block_type::index_to_block};

type BlocksInMesh = Vec<(usize, usize, usize)>;

fn run_length_encode(to_serialize: &String) -> String {
    let mut result = String::new();
    if to_serialize.len() == 0 {
        return result;
    }

    let bytes = to_serialize.as_bytes();
    let mut current_ch = bytes[0];
    let mut acc = 1;
    for i in 1..bytes.len() {
        let ch = bytes[i];
        let is_not_last_char = i < bytes.len() - 1;
        if ch == current_ch && is_not_last_char && acc < 64 {
            acc += 1;
        } else {
            if !is_not_last_char {
                acc += 1;
            }

            // 128 64 32 16 8 4 2 1
  {
                // add 5th bit to mark byte as count
                let num = (acc as u8) | (1 << 6);
                result.push(num as char);
            }
            result.push(current_ch as char);
            acc = 1;
            current_ch = ch;
        }
    }
    result
}

fn run_length_decode(serialized: &String) -> String {
    let mut result = String::new();
    let bytes = serialized.as_bytes();
    let mut i = 0;
    while i < serialized.len() {
        let byte = bytes[i];
        // use left-most bit as flag of consecutive byte count
        if (byte >> 6) == 1 {
            let count = byte & (1 << 6);
            for _ in 0..count {
                result.push(bytes[i + 1] as char);
            }
            i += 1;
        }
        i += 1;
    }
    result
}

pub fn from_serialized(serialized: &String) -> (BlocksInMesh, BlockMap) {
    // format (127 as delimiter between layers)
    // 127 <y_greater_than_1> <y mod 127> 16x16 layer grid ...
    let mut blocks_in_mesh = Vec::new();
    let mut blocks = BlockMap::new();
    let bytes = serialized.as_bytes();
    let mut i = 0;
    let mut y = 0;
    let mut iter_in_layer = 0;
    while i < bytes.len() {
        let byte = bytes[i];
        if byte == 127 {
            y = bytes[i + 1] * 127 + bytes[i + 2];
            iter_in_layer = 0;
            i += 2;
        } else {
            let x = iter_in_layer / 16;
            let z = iter_in_layer % 16;
            blocks_in_mesh.push((x, y as usize, z));
            let block = index_to_block(byte as usize);
            blocks.set(x, y as usize, z, block);
            iter_in_layer += 1;
        }
        i += 1;
    }
    (blocks_in_mesh, blocks)
}

// for calculating compression ratio
pub fn original_serialize(blocks_in_mesh: &BlocksInMesh, blocks: &BlockMap) -> String {
    let mut result = String::new();
    for (x, y, z) in blocks_in_mesh.iter() {
        result.push(*x as u8 as char);
        result.push(*y as u8 as char);
        result.push(*z as u8 as char);
        result.push(blocks.get(*x, *y, *z) as u8 as char);
    }
    result
}

pub fn to_serialized(blocks_in_mesh: &BlocksInMesh, blocks: &BlockMap) -> String {
    // map y to list of x, z and block tuples
    let mut layer_ys = HashSet::new();
    for (_, y, _) in blocks_in_mesh.iter() {
        layer_ys.insert(*y);
    }

    let mut serialized = String::new();
    for y in layer_ys.iter() {
        // use 255 as delimiter, ignored in RLE compression
        serialized.push(127 as u8 as char);
        let y = *y as u8;
        // need two chars to represent 0-255
        let has_127 = if y > 127 { 1u8 } else { 0u8 };
        serialized.push(has_127 as char);
        serialized.push((y % 127) as char);
        for x in 0..16 {
            for z in 0..16 {
                let block = blocks.get(x, y as usize, z);
                serialized.push(block as u8 as char);
            }
        }
    }
    serialized
}