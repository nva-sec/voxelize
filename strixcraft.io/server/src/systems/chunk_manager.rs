use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use log::{info, warn, error};

use crate::worlds::terrain_generator::TerrainGenerator;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub x: i32,
    pub z: i32,
    pub blocks: Vec<u8>,
    pub metadata: Vec<u8>,
    pub light: Vec<u8>,
    pub height_map: Vec<u8>,
    pub is_generated: bool,
    pub is_modified: bool,
    pub last_accessed: std::time::Instant,
}

#[derive(Debug)]
pub struct ChunkManager {
    chunks: HashMap<(i32, i32), Chunk>,
    load_distance: i32,
    terrain_generator: Arc<TerrainGenerator>,
    max_cached_chunks: usize,
}

impl ChunkManager {
    pub fn new(load_distance: i32, terrain_generator: Arc<TerrainGenerator>) -> Self {
        Self {
            chunks: HashMap::new(),
            load_distance,
            terrain_generator,
            max_cached_chunks: 1000, // Adjust based on memory constraints
        }
    }

    pub async fn get_chunk(&mut self, x: i32, z: i32) -> Option<Chunk> {
        let key = (x, z);
        
        if let Some(chunk) = self.chunks.get_mut(&key) {
            chunk.last_accessed = std::time::Instant::now();
            return Some(chunk.clone());
        }

        // Generate new chunk if not found
        let chunk = self.generate_chunk(x, z).await?;
        self.chunks.insert(key, chunk.clone());
        
        // Clean up old chunks if we exceed the limit
        self.cleanup_old_chunks().await;
        
        Some(chunk)
    }

    pub async fn get_chunks_in_radius(&mut self, center_x: i32, center_z: i32) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        
        for x in (center_x - self.load_distance)..=(center_x + self.load_distance) {
            for z in (center_z - self.load_distance)..=(center_z + self.load_distance) {
                if let Some(chunk) = self.get_chunk(x, z).await {
                    chunks.push(chunk);
                }
            }
        }
        
        chunks
    }

    pub async fn set_block(&mut self, x: i32, y: i32, z: i32, block_id: u8) -> Result<(), Box<dyn std::error::Error>> {
        let chunk_x = x >> 4; // Divide by 16
        let chunk_z = z >> 4;
        let local_x = x & 15; // Modulo 16
        let local_z = z & 15;
        
        let key = (chunk_x, chunk_z);
        
        if let Some(chunk) = self.chunks.get_mut(&key) {
            let index = (y as usize * 16 * 16) + (local_z as usize * 16) + local_x as usize;
            if index < chunk.blocks.len() {
                chunk.blocks[index] = block_id;
                chunk.is_modified = true;
                chunk.last_accessed = std::time::Instant::now();
            }
        }
        
        Ok(())
    }

    pub async fn get_block(&self, x: i32, y: i32, z: i32) -> Option<u8> {
        let chunk_x = x >> 4;
        let chunk_z = z >> 4;
        let local_x = x & 15;
        let local_z = z & 15;
        
        let key = (chunk_x, chunk_z);
        
        if let Some(chunk) = self.chunks.get(&key) {
            let index = (y as usize * 16 * 16) + (local_z as usize * 16) + local_x as usize;
            if index < chunk.blocks.len() {
                return Some(chunk.blocks[index]);
            }
        }
        
        None
    }

    async fn generate_chunk(&self, x: i32, z: i32) -> Option<Chunk> {
        let chunk_size = 16 * 16 * 256; // 16x16 chunks, 256 blocks tall
        let mut blocks = vec![0u8; chunk_size];
        let mut metadata = vec![0u8; chunk_size];
        let mut light = vec![15u8; chunk_size]; // Full light by default
        let mut height_map = vec![0u8; 16 * 16];
        
        // Generate terrain using the terrain generator
        for local_x in 0..16 {
            for local_z in 0..16 {
                let world_x = x * 16 + local_x;
                let world_z = z * 16 + local_z;
                
                // Get height from terrain generator
                let height = self.terrain_generator.get_height(world_x, world_z).await;
                height_map[local_z as usize * 16 + local_x as usize] = height as u8;
                
                // Fill blocks from bottom to height
                for y in 0..=height {
                    let index = (y as usize * 16 * 16) + (local_z as usize * 16) + local_x as usize;
                    if index < blocks.len() {
                        blocks[index] = self.get_block_type_for_height(y, height);
                    }
                }
            }
        }
        
        Some(Chunk {
            x,
            z,
            blocks,
            metadata,
            light,
            height_map,
            is_generated: true,
            is_modified: false,
            last_accessed: std::time::Instant::now(),
        })
    }

    fn get_block_type_for_height(&self, y: i32, max_height: i32) -> u8 {
        if y == 0 {
            7 // Bedrock
        } else if y < max_height - 4 {
            1 // Stone
        } else if y < max_height {
            3 // Dirt
        } else if y == max_height {
            2 // Grass
        } else {
            0 // Air
        }
    }

    async fn cleanup_old_chunks(&mut self) {
        if self.chunks.len() <= self.max_cached_chunks {
            return;
        }

        let mut chunks_to_remove = Vec::new();
        let now = std::time::Instant::now();
        
        // Find chunks that haven't been accessed recently
        for (key, chunk) in &self.chunks {
            if !chunk.is_modified && now.duration_since(chunk.last_accessed).as_secs() > 300 { // 5 minutes
                chunks_to_remove.push(*key);
            }
        }
        
        // Remove old chunks
        for key in chunks_to_remove {
            self.chunks.remove(&key);
        }
        
        info!("Cleaned up {} old chunks", chunks_to_remove.len());
    }

    pub async fn save_modified_chunks(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut saved_count = 0;
        
        for (key, chunk) in &self.chunks {
            if chunk.is_modified {
                // Save chunk to disk/database
                self.save_chunk_to_storage(*key, chunk).await?;
                saved_count += 1;
            }
        }
        
        if saved_count > 0 {
            info!("Saved {} modified chunks", saved_count);
        }
        
        Ok(())
    }

    async fn save_chunk_to_storage(&self, _key: (i32, i32), _chunk: &Chunk) -> Result<(), Box<dyn std::error::Error>> {
        // Implementation for saving chunk to disk or database
        // This would typically serialize the chunk data and write it to a file or database
        Ok(())
    }

    pub async fn get_chunk_stats(&self) -> ChunkStats {
        let total_chunks = self.chunks.len();
        let modified_chunks = self.chunks.values().filter(|c| c.is_modified).count();
        let generated_chunks = self.chunks.values().filter(|c| c.is_generated).count();
        
        ChunkStats {
            total_chunks,
            modified_chunks,
            generated_chunks,
            max_cached_chunks: self.max_cached_chunks,
        }
    }
}

#[derive(Debug)]
pub struct ChunkStats {
    pub total_chunks: usize,
    pub modified_chunks: usize,
    pub generated_chunks: usize,
    pub max_cached_chunks: usize,
}