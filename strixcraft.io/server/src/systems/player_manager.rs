use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use log::{info, warn, error};

use crate::auth::auth_service::AuthService;
use crate::database::player_repository::PlayerRepository;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub username: String,
    pub position: [f64; 3],
    pub rotation: [f64; 3],
    pub health: f32,
    pub max_health: f32,
    pub hunger: f32,
    pub max_hunger: f32,
    pub experience: i32,
    pub level: i32,
    pub inventory: Vec<InventoryItem>,
    pub selected_slot: usize,
    pub game_mode: GameMode,
    pub world_id: Option<String>,
    pub is_online: bool,
    pub last_seen: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: u32,
    pub count: u32,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameMode {
    Survival,
    Creative,
}

#[derive(Debug)]
pub struct PlayerManager {
    players: HashMap<String, Player>,
    online_players: HashMap<String, String>, // session_id -> player_id
    auth_service: Arc<AuthService>,
    player_repository: Arc<PlayerRepository>,
}

impl PlayerManager {
    pub fn new(
        player_repository: Arc<PlayerRepository>,
        auth_service: Arc<AuthService>,
    ) -> Self {
        Self {
            players: HashMap::new(),
            online_players: HashMap::new(),
            auth_service,
            player_repository,
        }
    }

    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing player manager...");
        
        // Load existing players from database
        let existing_players = self.player_repository.get_all_players().await?;
        
        for player_data in existing_players {
            let player = Player {
                id: player_data.id,
                username: player_data.username,
                position: [0.0, 64.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                health: 20.0,
                max_health: 20.0,
                hunger: 20.0,
                max_hunger: 20.0,
                experience: 0,
                level: 1,
                inventory: vec![],
                selected_slot: 0,
                game_mode: GameMode::Survival,
                world_id: None,
                is_online: false,
                last_seen: player_data.last_seen,
                created_at: player_data.created_at,
            };
            
            self.players.insert(player.id.clone(), player);
        }
        
        info!("Player manager initialized with {} players", self.players.len());
        Ok(())
    }

    pub async fn authenticate_player(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<Option<Player>, Box<dyn std::error::Error>> {
        match self.auth_service.authenticate(username, password).await? {
            Some(player_id) => {
                if let Some(player) = self.players.get_mut(&player_id) {
                    player.is_online = true;
                    player.last_seen = Utc::now();
                    
                    // Update in database
                    self.player_repository.update_player_last_seen(&player_id).await?;
                    
                    Ok(Some(player.clone()))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    pub async fn register_player(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<Player, Box<dyn std::error::Error>> {
        // Check if username already exists
        if self.players.values().any(|p| p.username == username) {
            return Err("Username already exists".into());
        }

        let player_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let player = Player {
            id: player_id.clone(),
            username: username.to_string(),
            position: [0.0, 64.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            health: 20.0,
            max_health: 20.0,
            hunger: 20.0,
            max_hunger: 20.0,
            experience: 0,
            level: 1,
            inventory: vec![],
            selected_slot: 0,
            game_mode: GameMode::Survival,
            world_id: None,
            is_online: false,
            last_seen: now,
            created_at: now,
        };

        // Create player in database
        self.player_repository.create_player(&player).await?;
        
        // Create authentication credentials
        self.auth_service.create_user(username, password, &player_id).await?;
        
        // Add to memory
        self.players.insert(player_id.clone(), player.clone());
        
        info!("Registered new player: {} (ID: {})", username, player_id);
        
        Ok(player)
    }

    pub async fn get_player(&self, player_id: &str) -> Option<Player> {
        self.players.get(player_id).cloned()
    }

    pub async fn get_player_by_username(&self, username: &str) -> Option<Player> {
        self.players.values().find(|p| p.username == username).cloned()
    }

    pub async fn get_online_players(&self) -> Vec<Player> {
        self.players.values().filter(|p| p.is_online).cloned().collect()
    }

    pub async fn update_player_position(
        &mut self,
        player_id: &str,
        position: [f64; 3],
        rotation: [f64; 3],
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(player) = self.players.get_mut(player_id) {
            player.position = position;
            player.rotation = rotation;
            player.last_seen = Utc::now();
        }
        
        Ok(())
    }

    pub async fn update_player_health(
        &mut self,
        player_id: &str,
        health: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(player) = self.players.get_mut(player_id) {
            player.health = health.max(0.0).min(player.max_health);
        }
        
        Ok(())
    }

    pub async fn update_player_hunger(
        &mut self,
        player_id: &str,
        hunger: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(player) = self.players.get_mut(player_id) {
            player.hunger = hunger.max(0.0).min(player.max_hunger);
        }
        
        Ok(())
    }

    pub async fn update_player_experience(
        &mut self,
        player_id: &str,
        experience: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(player) = self.players.get_mut(player_id) {
            player.experience = experience;
            
            // Calculate level based on experience
            let new_level = (experience as f32 / 100.0).floor() as i32 + 1;
            if new_level != player.level {
                player.level = new_level;
                info!("Player {} leveled up to level {}", player.username, new_level);
            }
        }
        
        Ok(())
    }

    pub async fn update_player_inventory(
        &mut self,
        player_id: &str,
        inventory: Vec<InventoryItem>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(player) = self.players.get_mut(player_id) {
            player.inventory = inventory;
        }
        
        Ok(())
    }

    pub async fn set_player_world(
        &mut self,
        player_id: &str,
        world_id: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(player) = self.players.get_mut(player_id) {
            player.world_id = world_id;
        }
        
        Ok(())
    }

    pub async fn player_disconnect(&mut self, player_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(player) = self.players.get_mut(player_id) {
            player.is_online = false;
            player.last_seen = Utc::now();
            
            // Update in database
            self.player_repository.update_player_last_seen(player_id).await?;
            
            info!("Player disconnected: {} (ID: {})", player.username, player_id);
        }
        
        Ok(())
    }

    pub async fn get_players_in_world(&self, world_id: &str) -> Vec<Player> {
        self.players
            .values()
            .filter(|p| p.is_online && p.world_id.as_ref() == Some(&world_id.to_string()))
            .cloned()
            .collect()
    }

    pub async fn get_player_stats(&self) -> PlayerStats {
        let total_players = self.players.len();
        let online_players = self.players.values().filter(|p| p.is_online).count();
        let total_experience: i32 = self.players.values().map(|p| p.experience).sum();
        let average_level = if total_players > 0 {
            self.players.values().map(|p| p.level).sum::<i32>() as f32 / total_players as f32
        } else {
            0.0
        };
        
        PlayerStats {
            total_players,
            online_players,
            total_experience,
            average_level,
        }
    }
}

#[derive(Debug)]
pub struct PlayerStats {
    pub total_players: usize,
    pub online_players: usize,
    pub total_experience: i32,
    pub average_level: f32,
}