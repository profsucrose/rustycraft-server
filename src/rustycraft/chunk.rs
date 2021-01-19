use std::{fs, rc::Rc, sync::Arc};

use noise::{NoiseFn, OpenSimplex};

use rand::prelude::*;

use crate::rustycraft::{block_map::BlockMap, block_type::{BlockType, index_to_block}};

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_HEIGHT: usize = 256;

#[derive(Clone)]
pub struct Chunk {
    pub blocks: BlockMap,
    pub blocks_in_mesh: Vec<(usize, usize, usize)>,
    x: i32,
    z: i32,
    save_path: String,
    pub serialized_blocks: String
}

impl Chunk {
    pub fn from(save_path: String, contents: String, x: i32, z: i32) -> Chunk {
        // follows format
        // [x] [y] [z] [block_index]
        // [x1] [y1] [z1] [block_index1]
        // ...
        let mut blocks = BlockMap::new();
        let mut blocks_in_mesh = vec![];
        for lines in contents.lines() {
            let words: Vec<&str> = lines.split(" ").collect();
            let x = words[0].parse::<usize>().unwrap();
            let y = words[1].parse::<usize>().unwrap();
            let z = words[2].parse::<usize>().unwrap();
            let block_index = words[3].parse::<usize>().unwrap();
            blocks.set(x, y, z, index_to_block(block_index));
            blocks_in_mesh.push((x, y, z));
        }
        let mut chunk = Chunk { blocks, blocks_in_mesh, x: x * 16, z: z * 16, save_path, serialized_blocks: String::new() };
        chunk.reserialize();
        chunk
    }

    pub fn new(x_offset: i32, z_offset: i32, simplex: Arc<OpenSimplex>, chunk_dir: String) -> Chunk {
        let save_path = format!("{}/{}_{}", chunk_dir, x_offset, z_offset);
        let contents = fs::read_to_string(save_path.clone());
        if let Ok(contents) = contents {
            return Chunk::from(save_path, contents, x_offset, z_offset)
        }

        let amplitude = 5.0;
        let mut blocks = BlockMap::new();
        let mut blocks_in_mesh = Vec::new();
        let x_offset = x_offset * 16;
        let z_offset = z_offset * 16;
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let simplex_x = (x as i32 + x_offset) as f32;
                let simplex_z = (z as i32 + z_offset) as f32;
                let noise = gen_heightmap(simplex_x, simplex_z, simplex.clone());
                let height = (noise * amplitude) as usize + 3;
                if noise < 0.4 {
                    // if height is low enough make flat water
                    // let sand_level = 3;
                    // for y in 0..sand_level {
                    //     blocks.set(x, y, z, BlockType::Water);
                    // }
                    for y in 0..(0.4 * amplitude) as usize + 2 {
                        if y < height - 1 {
                            blocks.set(x, y, z, BlockType::Sand);
                        } else {
                            blocks.set(x, y, z, BlockType::Water);
                        }
                        blocks_in_mesh.push((x, y, z));
                    }
                } else {
                    for y in 0..height {
                        let distance_to_top = height - y;
                        let block =
                            if noise < 0.7 {
                                BlockType::Sand
                            } else {
                                match distance_to_top {
                                    1 => BlockType::Grass,
                                    2 | 3 => BlockType::Dirt,
                                    _ => BlockType::Stone
                                }
                            };
                        
                        blocks.set(x, y, z, block);
                        blocks_in_mesh.push((x, y, z));
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
            if blocks.get(x, top, z) != BlockType::Water {
                blocks.set(x, top, z, BlockType::Log);
                blocks_in_mesh.push((x, top, z));

                blocks.set(x, top + 1, z, BlockType::Log);
                blocks_in_mesh.push((x, top + 1, z));

                blocks.set(x, top + 2, z, BlockType::Log);
                blocks_in_mesh.push((x, top + 2, z));

                blocks.set(x, top + 3, z, BlockType::Log);
                blocks_in_mesh.push((x, top + 3, z));

                blocks.set(x + 1, top + 3, z, BlockType::Leaves);
                blocks_in_mesh.push((x + 1, top + 3, z));
                
                blocks.set(x - 1, top + 3, z, BlockType::Leaves);
                blocks_in_mesh.push((x - 1, top + 3, z));

                blocks.set(x, top + 3, z + 1, BlockType::Leaves);
                blocks_in_mesh.push((x, top + 3, z + 1));

                blocks.set(x, top + 3, z - 1, BlockType::Leaves);
                blocks_in_mesh.push((x, top + 3, z - 1));

                blocks.set(x, top + 4, z, BlockType::Leaves);
                blocks_in_mesh.push((x, top + 4, z));
            }
        }

        let mut chunk = Chunk { blocks, blocks_in_mesh, x: x_offset, z: z_offset, save_path, serialized_blocks: String::new() };
        chunk.reserialize();
        chunk.save();
        chunk
    }

    fn reserialize(&mut self) {
        self.blocks_in_mesh.sort_by(|(a, _, _), (b, _, _)| a.partial_cmp(b).unwrap());
        //println!("{:?}", self.blocks_in_mesh);
        let mut serialized_blocks = String::new();
        for (x, y, z) in self.blocks_in_mesh.iter() {
            let line = format!("{} {} {} {}\n", *x, *y, *z, self.blocks.get(*x, *y, *z) as usize);
            serialized_blocks.push_str(line.as_str());
        }
        self.serialized_blocks = serialized_blocks;
    }

    fn save(&self) {
        let mut string = String::new();


        for (x, y, z) in self.blocks_in_mesh.iter() {
            string.push_str(format!("{} {} {} {}\n", *x, *y, *z, self.blocks.get(*x, *y, *z) as usize).as_str())
        }
        fs::write(self.save_path.clone(), string)
            .expect(format!("Failed to save chunk to {}", self.save_path.clone()).as_str());
    }

    pub fn block_at(&self, x: usize, y: usize, z: usize) -> BlockType {
        self.blocks.get(x, y, z)
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
        self.reserialize();
        //self.gen_mesh();
    }

    pub fn can_place_at_local_spot(&self, x: i32, y: i32, z: i32, block: BlockType) -> bool {
        if y < 0 {
            return false
        }

        // if x < 0 || x >= CHUNK_SIZE as i32
        //     || y >= CHUNK_HEIGHT as i32
        //     || z < 0 || z >= CHUNK_SIZE as i32 {
        //     return true
        // }

        let block_spot = self.blocks.get(x as usize, y as usize, z as usize);
        block_spot == BlockType::Air || (block != BlockType::Water && block_spot == BlockType::Water)
    }
    
    pub fn highest_in_column(&self, x: usize, z: usize) -> usize {
        self.blocks.highest_in_column(x, z)
    }

    pub fn highest_in_column_from_y(&self, x: usize, y: usize, z: usize) -> usize {
        self.blocks.highest_in_column_from_y(x, y, z)
    }
}

fn gen_heightmap(x: f32, z: f32, simplex: Arc<OpenSimplex>) -> f32 {
    // get distance from center
    let nx = x / 5.0 - 0.5;
    let nz = z / 5.0 - 0.5;
    let d = (nx * nx + nz * nz).sqrt() / (0.5 as f32).sqrt(); 

    let height = 5.0 * sample_simplex(x / 35.0, z / 35.0, simplex.clone())
    + 2.0 * sample_simplex(x / 10.0, z / 10.0, simplex.clone())
    + 0.25 * sample_simplex(x / 4.0, z / 4.0, simplex.clone());
    let height = height.powf(1.3);
    (1.0 + height - d.powf(0.5)) / 2.0
}

fn sample_simplex(x: f32, z: f32, simplex: Arc<OpenSimplex>) -> f32 {
    // noise library returns noise value in range -1.0 to 1.0,
    // so shift over to 0.0 to 1.0 range
    ((simplex.get([x as f64, z as f64]) + 1.0) / 2.0) as f32
}