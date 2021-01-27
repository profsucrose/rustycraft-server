use std::fs;
use noise::{NoiseFn, OpenSimplex};
use rand::prelude::*;
use crate::{rustycraft::{block_map::BlockMap, block_type::BlockType}};
use super::chunk_utils::{from_serialized, to_serialized};

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_HEIGHT: usize = 256;

#[derive(Clone)]
pub struct Chunk {
    pub blocks: BlockMap,
    pub blocks_in_mesh: Vec<(usize, usize, usize)>,
    x: i32,
    z: i32,
    save_path: String
    //pub serialized_blocks: String
}

impl Chunk {
    pub fn from(save_path: String, contents: String, x: i32, z: i32) -> Chunk {
        // follows format (single line)
        // [amount if > 1][num][block][amount if > 1][num][block]...
        let (blocks_in_mesh, blocks) = from_serialized(&contents);
        let chunk = Chunk { blocks, blocks_in_mesh, x: x * 16, z: z * 16, save_path };
        chunk
    }

    pub fn new(x_offset: i32, z_offset: i32, simplex: OpenSimplex, chunk_dir: String) -> Chunk {
        let save_path = format!("{}/{}_{}", chunk_dir, x_offset, z_offset);
        let contents = fs::read_to_string(save_path.clone());
        if let Ok(contents) = contents {
            return Chunk::from(save_path, contents, x_offset, z_offset)
        }

        let amplitude = 15.0;
        let mut blocks = BlockMap::new();
        let mut blocks_in_mesh = Vec::new();
        let x_offset = x_offset * 16;
        let z_offset = z_offset * 16;
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let simplex_x = (x as i32 + x_offset) as f32;
                let simplex_z = (z as i32 + z_offset) as f32;
                let noise = gen_heightmap(simplex_x, simplex_z, simplex);
                let height = ((amplitude * noise) as usize) + 1;
                if height < 10 {
                    for y in 0..9 {
                        let block = if y < height - 1 {
                            BlockType::Sand
                        } else {
                            BlockType::Water
                        };
                        add_block(&mut blocks, &mut blocks_in_mesh, x, y, z, block);
                    }
                } else {
                    let snow_offset = (sample(simplex_x * 4.0, simplex_z * 4.0, simplex) * 20.0) as usize;
                    for y in 0..height {
                        let block = if y == height - 1 {
                            if height > 30 + snow_offset {
                                BlockType::Snow   
                            } else if height > 30 - snow_offset {
                                BlockType::Stone
                            } else if height == 10 {
                                BlockType::Sand
                            } else {
                                BlockType::Grass
                            }
                        } else if y > height - 3 {
                            if height > 20 {
                                BlockType::Stone
                            } else {
                                BlockType::Dirt
                            }
                        } else {
                            BlockType::Stone
                        };
                        add_block(&mut blocks, &mut blocks_in_mesh, x, y, z, block);
                    }
                }
            }
        }

        // tree generation logic (hacked together, refactor later)
        let mut rng = rand::thread_rng();
        if rng.gen::<f32>() < 0.9 {
            let x = (rng.gen::<f32>() * 11.0) as usize + 3;
            let z = (rng.gen::<f32>() * 11.0) as usize + 3;
            let top = blocks.highest_in_column(x, z);
            let block = blocks.get(x, top, z);
            if block != BlockType::Water && block != BlockType::Stone && block != BlockType::Sand && block != BlockType::Snow {
                // trunk
                for i in 1..4 {
                    add_block(&mut blocks, &mut blocks_in_mesh, x, top + i, z, BlockType::Log);
                }
               
                // leaf layer
                for ix in 0..3 {
                    for iz in 0..3 {
                        add_block(&mut blocks, &mut blocks_in_mesh, x + 1 - ix, top + 3, z + 1 - iz, BlockType::Leaves);
                    }
                }

                // second layer
                add_block(&mut blocks, &mut blocks_in_mesh, x, top + 4, z, BlockType::Leaves);
                add_block(&mut blocks, &mut blocks_in_mesh, x + 1, top + 4, z, BlockType::Leaves);
                add_block(&mut blocks, &mut blocks_in_mesh, x - 1, top + 4, z, BlockType::Leaves);
                add_block(&mut blocks, &mut blocks_in_mesh, x, top + 4, z + 1, BlockType::Leaves);
                add_block(&mut blocks, &mut blocks_in_mesh, x, top + 4, z - 1, BlockType::Leaves);

                // highest leaf block
                add_block(&mut blocks, &mut blocks_in_mesh, x, top + 5, z, BlockType::Leaves);
            }
        }


        let chunk = Chunk { blocks, blocks_in_mesh, x: x_offset, z: z_offset, save_path };
        chunk.save();
        chunk
    }

    fn save(&self) {
        fs::write(self.save_path.clone(), to_serialized(&self.blocks_in_mesh, &self.blocks))
            .expect(format!("Failed to save chunk to {}", self.save_path.clone()).as_str());
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        self.blocks.set(x, y, z, block);
        if block == BlockType::Air {
            for i in 0..self.blocks_in_mesh.len() - 1 {
                if self.blocks_in_mesh[i] == (x, y, z) {
                    self.blocks_in_mesh.remove(i);
                    break;
                }
            }
        } else {
            self.blocks_in_mesh.push((x, y, z));
        }
        self.save();
    }

    pub fn highest_in_column(&self, x: usize, z: usize) -> usize {
        self.blocks.highest_in_column(x, z)
    }
}

// utils
fn gen_heightmap(x: f32, z: f32, simplex: OpenSimplex) -> f32 {
    let x = x / 100.0;
    let z = z / 100.0;
    let coeff = sample(x, z, simplex) * 2.0;
    let height = coeff * sample(x, z, simplex)
    + coeff * sample(2.0 * x, 2.0 * z, simplex)
    + 0.5 * coeff * sample(4.0 * x, 4.0 * z, simplex);
    height.powf(1.5)
}

fn sample(x: f32, z: f32, simplex: OpenSimplex) -> f32 {
    // noise library returns noise value in range -1.0 to 1.0,
    // so shift over to 0.0 to 1.0 range
    ((simplex.get([x as f64, z as f64]) + 1.0) / 2.0) as f32
}

fn add_block(blocks: &mut BlockMap, blocks_in_mesh: &mut Vec<(usize, usize, usize)>, x: usize, y: usize, z: usize, block: BlockType) {
    blocks.set(x, y, z, block);
    blocks_in_mesh.push((x, y, z));
}