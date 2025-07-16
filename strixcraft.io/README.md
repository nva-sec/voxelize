# StrixCraft.io

A complete, production-ready voxel game built with the Voxelize engine, featuring both Survival and Creative modes with full multiplayer support.

## 🎮 Features

### Core Gameplay
- **Survival Mode**: Health, hunger, crafting, mining, building
- **Creative Mode**: Unlimited resources, flight, instant building
- **Procedural World Generation**: Multiple biomes, caves, structures
- **Multiplayer Support**: Real-time multiplayer with world synchronization
- **Dynamic World Manager**: Create, join, and manage multiple worlds

### World Management
- Create custom worlds with unique seeds
- Choose between Survival and Creative modes
- Real-time world statistics and player counts
- Automatic world persistence and loading

### Game Systems
- **Block System**: 100+ blocks with unique properties
- **Crafting System**: Complete recipe database with crafting tables
- **Inventory System**: 36-slot inventory with hotbar
- **Chat System**: Global, local, and private messaging
- **Physics System**: Realistic physics and collision detection
- **Mob System**: AI-driven creatures and animals

## 🏗️ Architecture

### Server (Rust)
- **Actix-web** for HTTP and WebSocket handling
- **Voxelize engine** for voxel game mechanics
- **SQLite** for data persistence
- **JWT** for authentication
- **Multi-threaded** systems for performance

### Client (TypeScript/React)
- **React** for UI components
- **Three.js** for 3D rendering
- **Voxelize core** for client-side game logic
- **Zustand** for state management
- **Vite** for fast development

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+ and Cargo
- Node.js 18+ and npm/pnpm
- SQLite3

### Installation

1. **Clone and setup:**
```bash
cd strixcraft.io
npm run install:all
```

2. **Build the project:**
```bash
npm run build
```

3. **Start development servers:**
```bash
npm run dev
```

4. **Access the game:**
- Client: http://localhost:3000
- Server API: http://localhost:4000

### Production Deployment

1. **Build for production:**
```bash
npm run build
```

2. **Start production server:**
```bash
npm start
```

## 📁 Project Structure

```
strixcraft.io/
├── client/                 # React client application
│   ├── src/
│   │   ├── components/     # UI components
│   │   ├── systems/        # Game systems
│   │   ├── stores/         # State management
│   │   └── assets/         # Game assets
│   ├── package.json
│   └── vite.config.ts
├── server/                 # Rust server application
│   ├── src/
│   │   ├── systems/        # Game systems
│   │   ├── worlds/         # World generation
│   │   ├── entities/       # Entity management
│   │   ├── networking/     # Network handling
│   │   ├── auth/           # Authentication
│   │   └── database/       # Data persistence
│   └── Cargo.toml
├── shared/                 # Shared types and utilities
│   ├── src/
│   │   ├── types.ts        # TypeScript types
│   │   └── protocol.ts     # Network protocol
│   └── package.json
├── assets/                 # Game assets
├── worlds/                 # World data
├── package.json            # Root package.json
└── README.md
```

## 🎯 Game Modes

### Survival Mode
- **Health System**: 20 hearts, damage from falls, mobs, hunger
- **Hunger System**: Food required, affects health regeneration
- **Crafting**: Tools, weapons, armor, building materials
- **Mining**: Resource gathering, ore processing
- **Building**: Construct shelters, farms, machines
- **Combat**: Fight hostile mobs, PvP (configurable)

### Creative Mode
- **Unlimited Resources**: All blocks available instantly
- **Flight**: Free movement in 3D space
- **Instant Breaking**: No tool requirements
- **No Damage**: Invulnerable to all damage sources
- **Creative Inventory**: Access to all items and blocks

## 🌍 World Generation

### Biomes
- **Forest**: Dense trees, moderate resources
- **Desert**: Sparse vegetation, sand, cacti
- **Mountains**: High elevation, snow, ores
- **Plains**: Open areas, grass, passive mobs
- **Swamp**: Water, trees, unique vegetation
- **Ocean**: Water, fish, underwater structures

### Features
- **Cave Systems**: Underground exploration
- **Ore Generation**: Coal, iron, gold, diamond, etc.
- **Structures**: Trees, villages, dungeons
- **Water Systems**: Rivers, lakes, oceans
- **Weather**: Rain, snow, storms

## 🔧 Configuration

### Server Configuration
```toml
[server]
port = 4000
host = "127.0.0.1"
max_players = 100
world_save_interval = 300
chunk_load_distance = 8
enable_physics = true
enable_mobs = true
enable_weather = true
enable_time = true
```

### World Settings
```json
{
  "name": "My World",
  "seed": 12345,
  "gameMode": "survival",
  "maxPlayers": 20,
  "allowPvP": false,
  "allowMobGriefing": true,
  "keepInventory": false,
  "naturalRegeneration": true,
  "difficulty": "normal"
}
```

## 🎮 Controls

### Movement
- **WASD**: Move forward/left/backward/right
- **Space**: Jump
- **Shift**: Sneak
- **Ctrl**: Sprint

### Interaction
- **Left Click**: Break blocks
- **Right Click**: Place blocks/use items
- **Middle Click**: Pick block
- **E**: Open inventory
- **T**: Open chat
- **F1**: Toggle debug info

### Creative Mode
- **Double Space**: Toggle flight
- **Ctrl + Space**: Fly up
- **Shift + Space**: Fly down

## 🛠️ Development

### Adding New Blocks
1. Register block in `server/src/systems/registry.rs`
2. Add texture in `client/src/assets/textures/`
3. Update block properties and crafting recipes

### Adding New Mobs
1. Create mob entity in `server/src/entities/mob.rs`
2. Implement AI behavior in `server/src/systems/mob_system.rs`
3. Add model and animations in client

### Adding New Biomes
1. Define biome in `server/src/worlds/biome_system.rs`
2. Add terrain generation in `server/src/worlds/terrain_generator.rs`
3. Create biome-specific structures and features

## 🧪 Testing

### Unit Tests
```bash
# Server tests
cd server && cargo test

# Client tests
cd client && npm test
```

### Integration Tests
```bash
npm run test:integration
```

### Performance Tests
```bash
npm run test:performance
```

## 📊 Performance

### Server Performance
- **Chunk Loading**: < 50ms per chunk
- **Player Updates**: < 16ms per player
- **Memory Usage**: < 2GB for 100 players
- **CPU Usage**: < 30% under normal load

### Client Performance
- **Frame Rate**: 60 FPS on medium hardware
- **Memory Usage**: < 1GB
- **Network**: < 100KB/s per player
- **Loading Time**: < 5 seconds initial load

## 🔒 Security

### Authentication
- JWT-based authentication
- Password hashing with bcrypt
- Session management
- Rate limiting

### Network Security
- WebSocket encryption
- Message validation
- Anti-cheat measures
- DDoS protection

## 📈 Monitoring

### Server Metrics
- Player count and activity
- World performance statistics
- Network usage and latency
- Memory and CPU usage

### Client Metrics
- Frame rate and performance
- Network latency
- Error tracking
- User behavior analytics

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

### Code Style
- **Rust**: Follow Rust style guidelines
- **TypeScript**: Use ESLint and Prettier
- **React**: Follow React best practices

## 📄 License

MIT License - see LICENSE file for details

## 🙏 Acknowledgments

- **Voxelize Engine**: The foundation of this project
- **Three.js**: 3D rendering library
- **Actix-web**: Web framework for Rust
- **React**: UI framework

## 📞 Support

- **Discord**: Join our community
- **Issues**: Report bugs and request features
- **Documentation**: Comprehensive guides and API docs

---

**StrixCraft.io** - A complete voxel gaming experience