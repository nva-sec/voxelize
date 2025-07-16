use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use log::{info, warn, error};

use crate::worlds::{
    terrain_generator::TerrainGenerator,
    biome_system::BiomeSystem,
    structure_generator::StructureGenerator,
};

use crate::database::world_repository::WorldRepository;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldInfo {
    pub id: String,
    pub name: String,
    pub seed: i64,
    pub game_mode: GameMode,
    pub player_count: usize,
    pub max_players: usize,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub is_online: bool,
    pub settings: WorldSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameMode {
    Survival,
    Creative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSettings {
    pub allow_pvp: bool,
    pub allow_mob_griefing: bool,
    pub keep_inventory: bool,
    pub natural_regeneration: bool,
    pub difficulty: Difficulty,
    pub weather_enabled: bool,
    pub time_enabled: bool,
    pub mobs_enabled: bool,
    pub physics_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

#[derive(Debug)]
pub struct WorldManager {
    worlds: HashMap<String, WorldInfo>,
    world_repository: Arc<WorldRepository>,
    terrain_generator: Arc<TerrainGenerator>,
    biome_system: Arc<BiomeSystem>,
    structure_generator: Arc<StructureGenerator>,
}

impl WorldManager {
    pub fn new(
        world_repository: Arc<WorldRepository>,
        terrain_generator: Arc<TerrainGenerator>,
        biome_system: Arc<BiomeSystem>,
        structure_generator: Arc<StructureGenerator>,
    ) -> Self {
        Self {
            worlds: HashMap::new(),
            world_repository,
            terrain_generator,
            biome_system,
            structure_generator,
        }
    }

    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing world manager...");
        
        // Load existing worlds from database
        let existing_worlds = self.world_repository.get_all_worlds().await?;
        
        for world_data in existing_worlds {
            let world_info = WorldInfo {
                id: world_data.id,
                name: world_data.name,
                seed: world_data.seed,
                game_mode: match world_data.game_mode.as_str() {
                    "survival" => GameMode::Survival,
                    "creative" => GameMode::Creative,
                    _ => GameMode::Survival,
                },
                player_count: 0,
                max_players: world_data.max_players,
                created_at: world_data.created_at,
                last_active: world_data.last_active,
                is_online: false,
                settings: serde_json::from_value(world_data.settings)?,
            };
            
            self.worlds.insert(world_info.id.clone(), world_info);
        }
        
        info!("World manager initialized with {} worlds", self.worlds.len());
        Ok(())
    }

    pub async fn create_world(
        &mut self,
        name: String,
        seed: i64,
        game_mode: GameMode,
        settings: WorldSettings,
    ) -> Result<WorldInfo, Box<dyn std::error::Error>> {
        let world_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let world_info = WorldInfo {
            id: world_id.clone(),
            name: name.clone(),
            seed,
            game_mode: game_mode.clone(),
            player_count: 0,
            max_players: 20,
            created_at: now,
            last_active: now,
            is_online: false,
            settings,
        };

        // Save to database
        self.world_repository.create_world(&world_info).await?;
        
        // Add to memory
        self.worlds.insert(world_id.clone(), world_info.clone());
        
        info!("Created new world: {} (ID: {})", name, world_id);
        
        Ok(world_info)
    }

    pub async fn get_world(&self, world_id: &str) -> Option<WorldInfo> {
        self.worlds.get(world_id).cloned()
    }

    pub async fn get_all_worlds(&self) -> Vec<WorldInfo> {
        self.worlds.values().cloned().collect()
    }

    pub async fn update_world(&mut self, world_id: &str, updates: WorldUpdate) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(world) = self.worlds.get_mut(world_id) {
            match updates {
                WorldUpdate::PlayerCount(count) => {
                    world.player_count = count;
                }
                WorldUpdate::LastActive(time) => {
                    world.last_active = time;
                }
                WorldUpdate::IsOnline(online) => {
                    world.is_online = online;
                }
                WorldUpdate::Settings(settings) => {
                    world.settings = settings;
                }
            }
            
            // Update in database
            self.world_repository.update_world(world_id, &updates).await?;
        }
        
        Ok(())
    }

    pub async fn delete_world(&mut self, world_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(world) = self.worlds.remove(world_id) {
            // Delete from database
            self.world_repository.delete_world(world_id).await?;
            
            info!("Deleted world: {} (ID: {})", world.name, world_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn join_world(&mut self, world_id: &str) -> Result<WorldInfo, Box<dyn std::error::Error>> {
        if let Some(world) = self.worlds.get_mut(world_id) {
            if world.player_count >= world.max_players {
                return Err("World is full".into());
            }
            
            world.player_count += 1;
            world.last_active = Utc::now();
            world.is_online = true;
            
            // Update in database
            self.world_repository.update_world(world_id, &WorldUpdate::PlayerCount(world.player_count)).await?;
            
            Ok(world.clone())
        } else {
            Err("World not found".into())
        }
    }

    pub async fn leave_world(&mut self, world_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(world) = self.worlds.get_mut(world_id) {
            if world.player_count > 0 {
                world.player_count -= 1;
                
                if world.player_count == 0 {
                    world.is_online = false;
                }
                
                world.last_active = Utc::now();
                
                // Update in database
                self.world_repository.update_world(world_id, &WorldUpdate::PlayerCount(world.player_count)).await?;
            }
        }
        
        Ok(())
    }

    pub async fn get_world_stats(&self) -> WorldStats {
        let total_worlds = self.worlds.len();
        let online_worlds = self.worlds.values().filter(|w| w.is_online).count();
        let total_players = self.worlds.values().map(|w| w.player_count).sum();
        
        WorldStats {
            total_worlds,
            online_worlds,
            total_players,
        }
    }
}

#[derive(Debug)]
pub enum WorldUpdate {
    PlayerCount(usize),
    LastActive(DateTime<Utc>),
    IsOnline(bool),
    Settings(WorldSettings),
}

#[derive(Debug)]
pub struct WorldStats {
    pub total_worlds: usize,
    pub online_worlds: usize,
    pub total_players: usize,
}