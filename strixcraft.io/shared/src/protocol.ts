export enum MessageType {
  // Authentication
  AUTH_REQUEST = 'auth_request',
  AUTH_RESPONSE = 'auth_response',
  
  // World Management
  WORLD_LIST_REQUEST = 'world_list_request',
  WORLD_LIST_RESPONSE = 'world_list_response',
  WORLD_CREATE_REQUEST = 'world_create_request',
  WORLD_CREATE_RESPONSE = 'world_create_response',
  WORLD_JOIN_REQUEST = 'world_join_request',
  WORLD_JOIN_RESPONSE = 'world_join_response',
  WORLD_LEAVE_REQUEST = 'world_leave_request',
  WORLD_DELETE_REQUEST = 'world_delete_request',
  
  // Player Management
  PLAYER_JOIN = 'player_join',
  PLAYER_LEAVE = 'player_leave',
  PLAYER_UPDATE = 'player_update',
  PLAYER_INVENTORY_UPDATE = 'player_inventory_update',
  
  // World Data
  CHUNK_REQUEST = 'chunk_request',
  CHUNK_DATA = 'chunk_data',
  BLOCK_UPDATE = 'block_update',
  ENTITY_SPAWN = 'entity_spawn',
  ENTITY_UPDATE = 'entity_update',
  ENTITY_DESPAWN = 'entity_despawn',
  
  // Game Mechanics
  CRAFTING_REQUEST = 'crafting_request',
  CRAFTING_RESPONSE = 'crafting_response',
  INVENTORY_ACTION = 'inventory_action',
  HEALTH_UPDATE = 'health_update',
  EXPERIENCE_UPDATE = 'experience_update',
  
  // Chat
  CHAT_MESSAGE = 'chat_message',
  COMMAND_REQUEST = 'command_request',
  COMMAND_RESPONSE = 'command_response',
  
  // System
  PING = 'ping',
  PONG = 'pong',
  ERROR = 'error',
  SERVER_STATS = 'server_stats'
}

export interface BaseMessage {
  type: MessageType
  id: string
  timestamp: number
}

export interface AuthRequestMessage extends BaseMessage {
  type: MessageType.AUTH_REQUEST
  username: string
  password: string
}

export interface AuthResponseMessage extends BaseMessage {
  type: MessageType.AUTH_RESPONSE
  success: boolean
  token?: string
  player?: any
  error?: string
}

export interface WorldListRequestMessage extends BaseMessage {
  type: MessageType.WORLD_LIST_REQUEST
}

export interface WorldListResponseMessage extends BaseMessage {
  type: MessageType.WORLD_LIST_RESPONSE
  worlds: any[]
}

export interface WorldCreateRequestMessage extends BaseMessage {
  type: MessageType.WORLD_CREATE_REQUEST
  name: string
  seed: number
  gameMode: string
  settings: any
}

export interface WorldCreateResponseMessage extends BaseMessage {
  type: MessageType.WORLD_CREATE_RESPONSE
  success: boolean
  world?: any
  error?: string
}

export interface WorldJoinRequestMessage extends BaseMessage {
  type: MessageType.WORLD_JOIN_REQUEST
  worldId: string
}

export interface WorldJoinResponseMessage extends BaseMessage {
  type: MessageType.WORLD_JOIN_RESPONSE
  success: boolean
  world?: any
  player?: any
  error?: string
}

export interface PlayerJoinMessage extends BaseMessage {
  type: MessageType.PLAYER_JOIN
  player: any
}

export interface PlayerLeaveMessage extends BaseMessage {
  type: MessageType.PLAYER_LEAVE
  playerId: string
}

export interface PlayerUpdateMessage extends BaseMessage {
  type: MessageType.PLAYER_UPDATE
  playerId: string
  position: [number, number, number]
  rotation: [number, number, number]
  health?: number
  hunger?: number
}

export interface ChunkRequestMessage extends BaseMessage {
  type: MessageType.CHUNK_REQUEST
  x: number
  z: number
}

export interface ChunkDataMessage extends BaseMessage {
  type: MessageType.CHUNK_DATA
  x: number
  z: number
  blocks: Uint8Array
  metadata: Uint8Array
  light: Uint8Array
}

export interface BlockUpdateMessage extends BaseMessage {
  type: MessageType.BLOCK_UPDATE
  x: number
  y: number
  z: number
  blockId: number
  metadata?: number
}

export interface ChatMessageMessage extends BaseMessage {
  type: MessageType.CHAT_MESSAGE
  sender: string
  content: string
  messageType: 'chat' | 'system' | 'command'
}

export interface CommandRequestMessage extends BaseMessage {
  type: MessageType.COMMAND_REQUEST
  command: string
  args: string[]
}

export interface CommandResponseMessage extends BaseMessage {
  type: MessageType.COMMAND_RESPONSE
  success: boolean
  message: string
  data?: any
}

export interface PingMessage extends BaseMessage {
  type: MessageType.PING
}

export interface PongMessage extends BaseMessage {
  type: MessageType.PONG
  timestamp: number
}

export interface ErrorMessage extends BaseMessage {
  type: MessageType.ERROR
  error: string
  code?: number
}

export type NetworkMessage = 
  | AuthRequestMessage
  | AuthResponseMessage
  | WorldListRequestMessage
  | WorldListResponseMessage
  | WorldCreateRequestMessage
  | WorldCreateResponseMessage
  | WorldJoinRequestMessage
  | WorldJoinResponseMessage
  | PlayerJoinMessage
  | PlayerLeaveMessage
  | PlayerUpdateMessage
  | ChunkRequestMessage
  | ChunkDataMessage
  | BlockUpdateMessage
  | ChatMessageMessage
  | CommandRequestMessage
  | CommandResponseMessage
  | PingMessage
  | PongMessage
  | ErrorMessage