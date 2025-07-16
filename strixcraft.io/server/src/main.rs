use actix_web::{web, App, HttpServer, middleware, HttpResponse};
use actix_cors::Cors;
use actix_files::Files;
use log::{info, error};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

mod systems;
mod worlds;
mod entities;
mod networking;
mod auth;
mod database;

use crate::systems::{
    world_manager::WorldManager,
    player_manager::PlayerManager,
    chunk_manager::ChunkManager,
    entity_manager::EntityManager,
    crafting_system::CraftingSystem,
    inventory_system::InventorySystem,
    chat_system::ChatSystem,
    command_system::CommandSystem,
    physics_system::PhysicsSystem,
    mob_system::MobSystem,
    weather_system::WeatherSystem,
    time_system::TimeSystem,
    save_system::SaveSystem,
};

use crate::worlds::{
    terrain_generator::TerrainGenerator,
    biome_system::BiomeSystem,
    structure_generator::StructureGenerator,
};

use crate::entities::{
    player::Player,
    mob::Mob,
    item::Item,
};

use crate::networking::{
    websocket_handler::WebSocketHandler,
    message_handler::MessageHandler,
    protocol::Protocol,
};

use crate::auth::{
    auth_service::AuthService,
    jwt_service::JwtService,
};

use crate::database::{
    database_service::DatabaseService,
    world_repository::WorldRepository,
    player_repository::PlayerRepository,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub max_players: usize,
    pub world_save_interval: u64,
    pub chunk_load_distance: i32,
    pub enable_physics: bool,
    pub enable_mobs: bool,
    pub enable_weather: bool,
    pub enable_time: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 4000,
            host: "127.0.0.1".to_string(),
            max_players: 100,
            world_save_interval: 300, // 5 minutes
            chunk_load_distance: 8,
            enable_physics: true,
            enable_mobs: true,
            enable_weather: true,
            enable_time: true,
        }
    }
}

#[derive(Debug)]
pub struct StrixCraftServer {
    config: ServerConfig,
    world_manager: Arc<RwLock<WorldManager>>,
    player_manager: Arc<RwLock<PlayerManager>>,
    chunk_manager: Arc<RwLock<ChunkManager>>,
    entity_manager: Arc<RwLock<EntityManager>>,
    crafting_system: Arc<RwLock<CraftingSystem>>,
    inventory_system: Arc<RwLock<InventorySystem>>,
    chat_system: Arc<RwLock<ChatSystem>>,
    command_system: Arc<RwLock<CommandSystem>>,
    physics_system: Arc<RwLock<PhysicsSystem>>,
    mob_system: Arc<RwLock<MobSystem>>,
    weather_system: Arc<RwLock<WeatherSystem>>,
    time_system: Arc<RwLock<TimeSystem>>,
    save_system: Arc<RwLock<SaveSystem>>,
    terrain_generator: Arc<TerrainGenerator>,
    biome_system: Arc<BiomeSystem>,
    structure_generator: Arc<StructureGenerator>,
    auth_service: Arc<AuthService>,
    jwt_service: Arc<JwtService>,
    database_service: Arc<DatabaseService>,
    world_repository: Arc<WorldRepository>,
    player_repository: Arc<PlayerRepository>,
    websocket_handler: Arc<WebSocketHandler>,
    message_handler: Arc<MessageHandler>,
    protocol: Arc<Protocol>,
}

impl StrixCraftServer {
    pub async fn new(config: ServerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing StrixCraft.io server...");

        // Initialize database
        let database_service = Arc::new(DatabaseService::new().await?);
        let world_repository = Arc::new(WorldRepository::new(database_service.clone()));
        let player_repository = Arc::new(PlayerRepository::new(database_service.clone()));

        // Initialize services
        let jwt_service = Arc::new(JwtService::new("your-secret-key".to_string()));
        let auth_service = Arc::new(AuthService::new(
            player_repository.clone(),
            jwt_service.clone(),
        ));

        // Initialize world generation systems
        let terrain_generator = Arc::new(TerrainGenerator::new());
        let biome_system = Arc::new(BiomeSystem::new());
        let structure_generator = Arc::new(StructureGenerator::new());

        // Initialize game systems
        let world_manager = Arc::new(RwLock::new(WorldManager::new(
            world_repository.clone(),
            terrain_generator.clone(),
            biome_system.clone(),
            structure_generator.clone(),
        )));

        let player_manager = Arc::new(RwLock::new(PlayerManager::new(
            player_repository.clone(),
            auth_service.clone(),
        )));

        let chunk_manager = Arc::new(RwLock::new(ChunkManager::new(
            config.chunk_load_distance,
            terrain_generator.clone(),
        )));

        let entity_manager = Arc::new(RwLock::new(EntityManager::new()));
        let crafting_system = Arc::new(RwLock::new(CraftingSystem::new()));
        let inventory_system = Arc::new(RwLock::new(InventorySystem::new()));
        let chat_system = Arc::new(RwLock::new(ChatSystem::new()));
        let command_system = Arc::new(RwLock::new(CommandSystem::new()));

        let physics_system = if config.enable_physics {
            Arc::new(RwLock::new(PhysicsSystem::new()))
        } else {
            Arc::new(RwLock::new(PhysicsSystem::new_disabled()))
        };

        let mob_system = if config.enable_mobs {
            Arc::new(RwLock::new(MobSystem::new()))
        } else {
            Arc::new(RwLock::new(MobSystem::new_disabled()))
        };

        let weather_system = if config.enable_weather {
            Arc::new(RwLock::new(WeatherSystem::new()))
        } else {
            Arc::new(RwLock::new(WeatherSystem::new_disabled()))
        };

        let time_system = if config.enable_time {
            Arc::new(RwLock::new(TimeSystem::new()))
        } else {
            Arc::new(RwLock::new(TimeSystem::new_disabled()))
        };

        let save_system = Arc::new(RwLock::new(SaveSystem::new(
            world_repository.clone(),
            player_repository.clone(),
            config.world_save_interval,
        )));

        // Initialize networking
        let protocol = Arc::new(Protocol::new());
        let message_handler = Arc::new(MessageHandler::new(
            world_manager.clone(),
            player_manager.clone(),
            chunk_manager.clone(),
            entity_manager.clone(),
            crafting_system.clone(),
            inventory_system.clone(),
            chat_system.clone(),
            command_system.clone(),
            protocol.clone(),
        ));

        let websocket_handler = Arc::new(WebSocketHandler::new(
            message_handler.clone(),
            protocol.clone(),
        ));

        info!("StrixCraft.io server initialized successfully!");

        Ok(Self {
            config,
            world_manager,
            player_manager,
            chunk_manager,
            entity_manager,
            crafting_system,
            inventory_system,
            chat_system,
            command_system,
            physics_system,
            mob_system,
            weather_system,
            time_system,
            save_system,
            terrain_generator,
            biome_system,
            structure_generator,
            auth_service,
            jwt_service,
            database_service,
            world_repository,
            player_repository,
            websocket_handler,
            message_handler,
            protocol,
        })
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting StrixCraft.io server on {}:{}", self.config.host, self.config.port);

        // Start background tasks
        self.start_background_tasks().await;

        // Start HTTP server
        HttpServer::new(move || {
            let cors = Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .supports_credentials();

            App::new()
                .wrap(middleware::Logger::default())
                .wrap(cors)
                .service(
                    web::scope("/api")
                        .route("/worlds", web::get().to(get_worlds))
                        .route("/worlds", web::post().to(create_world))
                        .route("/worlds/{id}", web::get().to(get_world))
                        .route("/worlds/{id}", web::delete().to(delete_world))
                        .route("/auth/login", web::post().to(login))
                        .route("/auth/register", web::post().to(register))
                        .route("/auth/verify", web::post().to(verify_token))
                        .route("/stats", web::get().to(get_server_stats))
                )
                .service(
                    web::scope("/ws")
                        .route("/game", web::get().to(websocket_route))
                )
                .service(Files::new("/", "../client/dist").index_file("index.html"))
        })
        .bind((self.config.host.clone(), self.config.port))?
        .run()
        .await?;

        Ok(())
    }

    async fn start_background_tasks(&self) {
        let save_system = self.save_system.clone();
        let time_system = self.time_system.clone();
        let weather_system = self.weather_system.clone();
        let mob_system = self.mob_system.clone();
        let physics_system = self.physics_system.clone();

        // Start save system
        tokio::spawn(async move {
            save_system.read().await.run().await;
        });

        // Start time system
        tokio::spawn(async move {
            time_system.read().await.run().await;
        });

        // Start weather system
        tokio::spawn(async move {
            weather_system.read().await.run().await;
        });

        // Start mob system
        tokio::spawn(async move {
            mob_system.read().await.run().await;
        });

        // Start physics system
        tokio::spawn(async move {
            physics_system.read().await.run().await;
        });
    }
}

// HTTP API endpoints
async fn get_worlds() -> HttpResponse {
    // Implementation for getting world list
    HttpResponse::Ok().json(vec![])
}

async fn create_world() -> HttpResponse {
    // Implementation for creating a new world
    HttpResponse::Ok().json(serde_json::json!({"success": true}))
}

async fn get_world(path: web::Path<String>) -> HttpResponse {
    let world_id = path.into_inner();
    // Implementation for getting world details
    HttpResponse::Ok().json(serde_json::json!({"id": world_id}))
}

async fn delete_world(path: web::Path<String>) -> HttpResponse {
    let world_id = path.into_inner();
    // Implementation for deleting a world
    HttpResponse::Ok().json(serde_json::json!({"success": true}))
}

async fn login() -> HttpResponse {
    // Implementation for user login
    HttpResponse::Ok().json(serde_json::json!({"success": true}))
}

async fn register() -> HttpResponse {
    // Implementation for user registration
    HttpResponse::Ok().json(serde_json::json!({"success": true}))
}

async fn verify_token() -> HttpResponse {
    // Implementation for token verification
    HttpResponse::Ok().json(serde_json::json!({"success": true}))
}

async fn get_server_stats() -> HttpResponse {
    // Implementation for getting server statistics
    HttpResponse::Ok().json(serde_json::json!({
        "uptime": 0,
        "playerCount": 0,
        "maxPlayers": 100,
        "worlds": 0,
        "chunksLoaded": 0,
        "memoryUsage": 0,
        "cpuUsage": 0
    }))
}

async fn websocket_route(
    req: actix_web::HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    // Implementation for WebSocket connection
    Ok(HttpResponse::Ok().finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file("strixcraft.log")?)
        .apply()
        .unwrap();

    info!("Starting StrixCraft.io server...");

    let config = ServerConfig::default();
    let server = StrixCraftServer::new(config).await.unwrap();
    
    server.start().await.unwrap();

    Ok(())
}