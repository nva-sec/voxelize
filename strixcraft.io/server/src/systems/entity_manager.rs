use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use log::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub entity_type: EntityType,
    pub position: [f64; 3],
    pub rotation: [f64; 3],
    pub velocity: [f64; 3],
    pub health: f32,
    pub max_health: f32,
    pub metadata: serde_json::Value,
    pub world_id: String,
    pub is_active: bool,
    pub created_at: std::time::Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Player,
    Zombie,
    Skeleton,
    Creeper,
    Spider,
    Cow,
    Pig,
    Sheep,
    Chicken,
    Item,
    Projectile,
    Vehicle,
}

#[derive(Debug)]
pub struct EntityManager {
    entities: HashMap<String, Entity>,
    entities_by_world: HashMap<String, Vec<String>>,
    entity_counters: HashMap<EntityType, u32>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            entities_by_world: HashMap::new(),
            entity_counters: HashMap::new(),
        }
    }

    pub async fn spawn_entity(
        &mut self,
        entity_type: EntityType,
        position: [f64; 3],
        world_id: String,
        metadata: Option<serde_json::Value>,
    ) -> String {
        let entity_id = Uuid::new_v4().to_string();
        
        let entity = Entity {
            id: entity_id.clone(),
            entity_type: entity_type.clone(),
            position,
            rotation: [0.0, 0.0, 0.0],
            velocity: [0.0, 0.0, 0.0],
            health: self.get_default_health(&entity_type),
            max_health: self.get_default_health(&entity_type),
            metadata: metadata.unwrap_or(serde_json::json!({})),
            world_id: world_id.clone(),
            is_active: true,
            created_at: std::time::Instant::now(),
        };

        self.entities.insert(entity_id.clone(), entity);
        
        // Add to world index
        self.entities_by_world
            .entry(world_id)
            .or_insert_with(Vec::new)
            .push(entity_id.clone());

        // Update counter
        *self.entity_counters.entry(entity_type).or_insert(0) += 1;

        info!("Spawned entity: {:?} at {:?} in world {}", entity_type, position, world_id);
        
        entity_id
    }

    pub async fn despawn_entity(&mut self, entity_id: &str) -> bool {
        if let Some(entity) = self.entities.remove(entity_id) {
            // Remove from world index
            if let Some(world_entities) = self.entities_by_world.get_mut(&entity.world_id) {
                world_entities.retain(|id| id != entity_id);
            }

            // Update counter
            if let Some(counter) = self.entity_counters.get_mut(&entity.entity_type) {
                if *counter > 0 {
                    *counter -= 1;
                }
            }

            info!("Despawned entity: {} ({:?})", entity_id, entity.entity_type);
            true
        } else {
            false
        }
    }

    pub async fn get_entity(&self, entity_id: &str) -> Option<Entity> {
        self.entities.get(entity_id).cloned()
    }

    pub async fn get_entities_in_world(&self, world_id: &str) -> Vec<Entity> {
        if let Some(entity_ids) = self.entities_by_world.get(world_id) {
            entity_ids
                .iter()
                .filter_map(|id| self.entities.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    pub async fn get_entities_in_radius(
        &self,
        center: [f64; 3],
        radius: f64,
        world_id: &str,
    ) -> Vec<Entity> {
        self.get_entities_in_world(world_id)
            .await
            .into_iter()
            .filter(|entity| {
                let dx = entity.position[0] - center[0];
                let dy = entity.position[1] - center[1];
                let dz = entity.position[2] - center[2];
                let distance = (dx * dx + dy * dy + dz * dz).sqrt();
                distance <= radius
            })
            .collect()
    }

    pub async fn update_entity_position(
        &mut self,
        entity_id: &str,
        position: [f64; 3],
        rotation: Option<[f64; 3]>,
    ) -> bool {
        if let Some(entity) = self.entities.get_mut(entity_id) {
            entity.position = position;
            if let Some(rot) = rotation {
                entity.rotation = rot;
            }
            true
        } else {
            false
        }
    }

    pub async fn update_entity_velocity(
        &mut self,
        entity_id: &str,
        velocity: [f64; 3],
    ) -> bool {
        if let Some(entity) = self.entities.get_mut(entity_id) {
            entity.velocity = velocity;
            true
        } else {
            false
        }
    }

    pub async fn damage_entity(
        &mut self,
        entity_id: &str,
        damage: f32,
    ) -> Option<f32> {
        if let Some(entity) = self.entities.get_mut(entity_id) {
            entity.health = (entity.health - damage).max(0.0);
            
            if entity.health <= 0.0 {
                entity.is_active = false;
            }
            
            Some(entity.health)
        } else {
            None
        }
    }

    pub async fn heal_entity(
        &mut self,
        entity_id: &str,
        amount: f32,
    ) -> Option<f32> {
        if let Some(entity) = self.entities.get_mut(entity_id) {
            entity.health = (entity.health + amount).min(entity.max_health);
            Some(entity.health)
        } else {
            None
        }
    }

    pub async fn update_entity_metadata(
        &mut self,
        entity_id: &str,
        metadata: serde_json::Value,
    ) -> bool {
        if let Some(entity) = self.entities.get_mut(entity_id) {
            entity.metadata = metadata;
            true
        } else {
            false
        }
    }

    pub async fn get_entity_stats(&self) -> EntityStats {
        let total_entities = self.entities.len();
        let active_entities = self.entities.values().filter(|e| e.is_active).count();
        
        let mut type_counts = HashMap::new();
        for entity in self.entities.values() {
            *type_counts.entry(entity.entity_type.clone()).or_insert(0) += 1;
        }

        EntityStats {
            total_entities,
            active_entities,
            type_counts,
        }
    }

    fn get_default_health(&self, entity_type: &EntityType) -> f32 {
        match entity_type {
            EntityType::Player => 20.0,
            EntityType::Zombie => 20.0,
            EntityType::Skeleton => 20.0,
            EntityType::Creeper => 20.0,
            EntityType::Spider => 16.0,
            EntityType::Cow => 10.0,
            EntityType::Pig => 10.0,
            EntityType::Sheep => 8.0,
            EntityType::Chicken => 4.0,
            EntityType::Item => 1.0,
            EntityType::Projectile => 1.0,
            EntityType::Vehicle => 40.0,
        }
    }

    pub async fn cleanup_dead_entities(&mut self) {
        let mut to_remove = Vec::new();
        
        for (entity_id, entity) in &self.entities {
            if !entity.is_active {
                to_remove.push(entity_id.clone());
            }
        }
        
        for entity_id in to_remove {
            self.despawn_entity(&entity_id).await;
        }
    }
}

#[derive(Debug)]
pub struct EntityStats {
    pub total_entities: usize,
    pub active_entities: usize,
    pub type_counts: HashMap<EntityType, usize>,
}