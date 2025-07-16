use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use log::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub sender: String,
    pub content: String,
    pub message_type: MessageType,
    pub timestamp: DateTime<Utc>,
    pub world_id: Option<String>,
    pub target_player: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Chat,
    System,
    Command,
    Whisper,
    Global,
    Team,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChannel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_global: bool,
    pub is_private: bool,
    pub members: Vec<String>,
    pub moderators: Vec<String>,
}

#[derive(Debug)]
pub struct ChatSystem {
    messages: Vec<ChatMessage>,
    channels: HashMap<String, ChatChannel>,
    max_messages: usize,
    profanity_filter: bool,
    rate_limiting: HashMap<String, DateTime<Utc>>,
    muted_players: HashMap<String, DateTime<Utc>>,
}

impl ChatSystem {
    pub fn new() -> Self {
        let mut system = Self {
            messages: Vec::new(),
            channels: HashMap::new(),
            max_messages: 1000,
            profanity_filter: true,
            rate_limiting: HashMap::new(),
            muted_players: HashMap::new(),
        };
        
        system.initialize_default_channels();
        system
    }

    pub fn send_message(
        &mut self,
        sender: &str,
        content: &str,
        message_type: MessageType,
        world_id: Option<String>,
        target_player: Option<String>,
    ) -> Result<ChatMessage, String> {
        // Check if player is muted
        if self.is_player_muted(sender) {
            return Err("You are currently muted".to_string());
        }

        // Rate limiting
        if !self.check_rate_limit(sender) {
            return Err("You are sending messages too quickly".to_string());
        }

        // Profanity filter
        let filtered_content = if self.profanity_filter {
            self.filter_profanity(content)
        } else {
            content.to_string()
        };

        let message = ChatMessage {
            id: Uuid::new_v4().to_string(),
            sender: sender.to_string(),
            content: filtered_content,
            message_type: message_type.clone(),
            timestamp: Utc::now(),
            world_id,
            target_player,
        };

        // Add to message history
        self.messages.push(message.clone());
        
        // Clean up old messages
        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }

        // Update rate limiting
        self.rate_limiting.insert(sender.to_string(), Utc::now());

        info!("Chat message from {}: {}", sender, filtered_content);
        
        Ok(message)
    }

    pub fn get_recent_messages(
        &self,
        count: usize,
        world_id: Option<&str>,
        channel_id: Option<&str>,
    ) -> Vec<ChatMessage> {
        self.messages
            .iter()
            .rev()
            .filter(|msg| {
                let world_match = world_id.map_or(true, |id| msg.world_id.as_deref() == Some(id));
                let channel_match = channel_id.map_or(true, |_| {
                    // Channel filtering logic would go here
                    true
                });
                world_match && channel_match
            })
            .take(count)
            .cloned()
            .collect()
    }

    pub fn create_channel(
        &mut self,
        id: String,
        name: String,
        description: String,
        is_global: bool,
        is_private: bool,
        creator: String,
    ) -> Result<ChatChannel, String> {
        if self.channels.contains_key(&id) {
            return Err("Channel already exists".to_string());
        }

        let channel = ChatChannel {
            id: id.clone(),
            name,
            description,
            is_global,
            is_private,
            members: vec![creator.clone()],
            moderators: vec![creator],
        };

        self.channels.insert(id.clone(), channel.clone());
        
        info!("Created chat channel: {}", name);
        
        Ok(channel)
    }

    pub fn join_channel(&mut self, channel_id: &str, player: &str) -> Result<(), String> {
        if let Some(channel) = self.channels.get_mut(channel_id) {
            if !channel.members.contains(&player.to_string()) {
                channel.members.push(player.to_string());
            }
            Ok(())
        } else {
            Err("Channel not found".to_string())
        }
    }

    pub fn leave_channel(&mut self, channel_id: &str, player: &str) -> Result<(), String> {
        if let Some(channel) = self.channels.get_mut(channel_id) {
            channel.members.retain(|member| member != player);
            Ok(())
        } else {
            Err("Channel not found".to_string())
        }
    }

    pub fn mute_player(&mut self, player: &str, duration_minutes: u32) {
        let mute_until = Utc::now() + chrono::Duration::minutes(duration_minutes as i64);
        self.muted_players.insert(player.to_string(), mute_until);
        
        info!("Muted player {} for {} minutes", player, duration_minutes);
    }

    pub fn unmute_player(&mut self, player: &str) -> bool {
        self.muted_players.remove(player).is_some()
    }

    pub fn is_player_muted(&self, player: &str) -> bool {
        if let Some(mute_until) = self.muted_players.get(player) {
            if Utc::now() > *mute_until {
                // Mute has expired, but we'll clean it up later
                return false;
            }
            true
        } else {
            false
        }
    }

    pub fn get_channel(&self, channel_id: &str) -> Option<&ChatChannel> {
        self.channels.get(channel_id)
    }

    pub fn get_all_channels(&self) -> Vec<&ChatChannel> {
        self.channels.values().collect()
    }

    pub fn get_player_channels(&self, player: &str) -> Vec<&ChatChannel> {
        self.channels
            .values()
            .filter(|channel| channel.members.contains(&player.to_string()))
            .collect()
    }

    pub fn broadcast_system_message(
        &mut self,
        content: &str,
        world_id: Option<String>,
    ) -> ChatMessage {
        self.send_message(
            "SYSTEM",
            content,
            MessageType::System,
            world_id,
            None,
        ).unwrap()
    }

    pub fn send_whisper(
        &mut self,
        sender: &str,
        target: &str,
        content: &str,
    ) -> Result<ChatMessage, String> {
        self.send_message(
            sender,
            content,
            MessageType::Whisper,
            None,
            Some(target.to_string()),
        )
    }

    pub fn get_chat_stats(&self) -> ChatStats {
        let total_messages = self.messages.len();
        let total_channels = self.channels.len();
        let muted_players = self.muted_players.len();
        
        let mut message_type_counts = HashMap::new();
        for message in &self.messages {
            *message_type_counts.entry(message.message_type.clone()).or_insert(0) += 1;
        }

        ChatStats {
            total_messages,
            total_channels,
            muted_players,
            message_type_counts,
        }
    }

    fn check_rate_limit(&self, player: &str) -> bool {
        if let Some(last_message) = self.rate_limiting.get(player) {
            let time_since = Utc::now().signed_duration_since(*last_message);
            time_since.num_seconds() >= 1 // 1 second between messages
        } else {
            true
        }
    }

    fn filter_profanity(&self, content: &str) -> String {
        // Simple profanity filter - in a real implementation, this would be more sophisticated
        let mut filtered = content.to_lowercase();
        
        let profane_words = vec![
            "badword1", "badword2", "badword3", // Add actual profane words here
        ];
        
        for word in profane_words {
            filtered = filtered.replace(word, &"*".repeat(word.len()));
        }
        
        filtered
    }

    fn initialize_default_channels(&mut self) {
        // Global channel
        self.create_channel(
            "global".to_string(),
            "Global".to_string(),
            "Global chat channel".to_string(),
            true,
            false,
            "SYSTEM".to_string(),
        ).unwrap();

        // Local channel
        self.create_channel(
            "local".to_string(),
            "Local".to_string(),
            "Local chat channel".to_string(),
            false,
            false,
            "SYSTEM".to_string(),
        ).unwrap();

        info!("Initialized default chat channels");
    }
}

#[derive(Debug)]
pub struct ChatStats {
    pub total_messages: usize,
    pub total_channels: usize,
    pub muted_players: usize,
    pub message_type_counts: HashMap<MessageType, usize>,
}