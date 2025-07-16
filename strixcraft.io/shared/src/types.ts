export interface WorldInfo {
  id: string
  name: string
  seed: number
  gameMode: GameMode
  playerCount: number
  maxPlayers: number
  createdAt: string
  lastActive: string
  isOnline: boolean
}

export enum GameMode {
  SURVIVAL = 'survival',
  CREATIVE = 'creative'
}

export interface Player {
  id: string
  username: string
  position: [number, number, number]
  rotation: [number, number, number]
  health: number
  maxHealth: number
  hunger: number
  maxHunger: number
  experience: number
  level: number
  inventory: InventoryItem[]
  selectedSlot: number
  gameMode: GameMode
}

export interface InventoryItem {
  id: number
  count: number
  metadata?: Record<string, any>
}

export interface Block {
  id: number
  name: string
  texture: string
  hardness: number
  toolRequired?: string
  drops: DropItem[]
  isTransparent: boolean
  isSolid: boolean
  isActive: boolean
}

export interface DropItem {
  id: number
  count: number
  probability: number
}

export interface CraftingRecipe {
  id: string
  name: string
  ingredients: CraftingIngredient[]
  result: CraftingResult
  craftingTable: boolean
}

export interface CraftingIngredient {
  id: number
  count: number
  position: [number, number]
}

export interface CraftingResult {
  id: number
  count: number
}

export interface Biome {
  id: string
  name: string
  temperature: number
  humidity: number
  baseHeight: number
  heightVariation: number
  blocks: BiomeBlocks
  structures: Structure[]
  mobs: MobSpawn[]
}

export interface BiomeBlocks {
  surface: number
  subsurface: number
  deep: number
  water: number
}

export interface Structure {
  id: string
  name: string
  rarity: number
  minHeight: number
  maxHeight: number
  blocks: [number, number, number, number][] // [x, y, z, blockId]
}

export interface MobSpawn {
  mobId: string
  weight: number
  minGroup: number
  maxGroup: number
}

export interface ChatMessage {
  id: string
  sender: string
  content: string
  timestamp: string
  type: 'chat' | 'system' | 'command'
}

export interface ServerStats {
  uptime: number
  playerCount: number
  maxPlayers: number
  worlds: number
  chunksLoaded: number
  memoryUsage: number
  cpuUsage: number
}

export interface WorldSettings {
  name: string
  seed: number
  gameMode: GameMode
  maxPlayers: number
  allowPvP: boolean
  allowMobGriefing: boolean
  keepInventory: boolean
  naturalRegeneration: boolean
  difficulty: 'peaceful' | 'easy' | 'normal' | 'hard'
}

export interface ChunkData {
  x: number
  z: number
  blocks: Uint8Array
  metadata: Uint8Array
  light: Uint8Array
}

export interface Entity {
  id: string
  type: string
  position: [number, number, number]
  rotation: [number, number, number]
  velocity: [number, number, number]
  health: number
  maxHealth: number
  metadata: Record<string, any>
}

export interface NetworkMessage {
  type: string
  payload: any
  timestamp: number
}

export interface AuthResponse {
  success: boolean
  token?: string
  player?: Player
  error?: string
}

export interface WorldJoinResponse {
  success: boolean
  world?: WorldInfo
  player?: Player
  error?: string
}