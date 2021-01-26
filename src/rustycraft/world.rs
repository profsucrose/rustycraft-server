use std::{fs, sync::Arc};
use std::time::{SystemTime, UNIX_EPOCH};

use noise::{OpenSimplex, Seedable};

use super::{block_type::BlockType, chunk::Chunk, coord_map::CoordMap};

#[derive(Clone)]
pub struct World {
    chunks: CoordMap<Chunk>,
    simplex: Arc<OpenSimplex>,
    player_chunk_x: i32,
    player_chunk_z: i32,
    pub save_dir: String
}

// handles world block data and rendering
impl World {
    pub fn new_with_seed(save_dir: &str, seed: u32) -> World {
        // create world directory if it does not exist
        let dir = format!("worlds/{}/chunks", save_dir);
        fs::create_dir_all(dir.clone()) 
            .expect(format!("Failed to recursively create {}", dir.clone()).as_str());

        let chunks = CoordMap::new();
        let simplex = OpenSimplex::new().set_seed(seed);
        let simplex = Arc::new(simplex);
        
        let save_dir = format!("worlds/{}", save_dir);
        World { chunks, simplex, player_chunk_x: 0, player_chunk_z: 0, save_dir }
    }

    pub fn new(save_dir: &str) -> World {
        let seed_path = format!("worlds/{}/seed", save_dir);
        let seed = fs::read_to_string(seed_path.clone());
        // read seed from world dir otherwise create
        // one and write to disk
        let seed = match seed {
            Ok(seed) => seed.parse::<u32>().unwrap(),
            Err(_) => {
                let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u32;
                fs::create_dir_all(format!("worlds/{}", save_dir))
                    .expect("Failed to create world directory");
                fs::write(seed_path.clone(), format!("{}", seed))
                    .expect(format!("Failed to write seed to {}", seed_path).as_str());
                seed
            }
        };
        World::new_with_seed(save_dir, seed)
    }

    pub fn get_or_insert_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> &Chunk {
        match self.chunks.contains(chunk_x, chunk_z) {
            true => self.chunks.get(chunk_x, chunk_z).unwrap(),
            false => {
                let c = Chunk::new(chunk_x, chunk_z, self.simplex.clone(), format!("{}/chunks", self.save_dir));
                self.chunks.insert(chunk_x, chunk_z, c);
                self.chunks.get(chunk_x, chunk_z).unwrap()
            }
        }
    }

    pub fn get_chunk_mut(&mut self, chunk_x: i32, chunk_z: i32) -> Option<&mut Chunk> {
        match self.chunks.contains(chunk_x, chunk_z) {
            true => self.chunks.get_mut(chunk_x, chunk_z),
            false => None
        }
    }

    pub fn get_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&Chunk> {
        self.chunks.get(chunk_x, chunk_z)
    }

    pub fn highest_in_column(&self, world_x: i32, world_z: i32) -> Option<usize> {
        let (chunk_x, chunk_z, local_x, local_z) = self.localize_coords_to_chunk(world_x, world_z);
        let chunk = self.get_chunk(chunk_x, chunk_z);
        if chunk.is_none() {
            return None
        }

        Some(chunk.unwrap().highest_in_column(local_x, local_z))
    }

    pub fn set_block(&mut self, world_x: i32, world_y: i32, world_z: i32, block: BlockType) {
        let (chunk_x, chunk_z, local_x, local_z) = self.localize_coords_to_chunk(world_x, world_z);
        self.get_or_insert_chunk(chunk_x, chunk_z);
        let chunk = self.get_chunk_mut(chunk_x, chunk_z).unwrap();
        chunk.set_block(local_x, world_y as usize, local_z, block);
    }

    pub fn localize_coords_to_chunk(&self, world_x: i32, world_z: i32) -> (i32, i32, usize, usize) {
        let mut chunk_x = (world_x + if world_x < 0 { 1 } else { 0 }) / 16;
        if world_x < 0 {
            chunk_x -= 1;
        }

        let mut chunk_z = (world_z + if world_z < 0 { 1 } else { 0 }) / 16;
        if world_z < 0 { 
            chunk_z -= 1;
        }

        let local_x = ((chunk_x.abs() * 16 + world_x) % 16).abs() as usize;
        let local_z = ((chunk_z.abs() * 16 + world_z) % 16).abs() as usize;
        (chunk_x, chunk_z, local_x, local_z)
    }
}