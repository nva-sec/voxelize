use serde::{Deserialize, Serialize};
use log::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: u32,
    pub count: u32,
    pub metadata: Option<serde_json::Value>,
    pub slot: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub items: Vec<Option<InventoryItem>>,
    pub size: usize,
    pub hotbar_size: usize,
    pub selected_slot: usize,
}

#[derive(Debug)]
pub struct InventorySystem;

impl InventorySystem {
    pub fn new() -> Self {
        Self
    }

    pub fn create_inventory(size: usize, hotbar_size: usize) -> Inventory {
        Inventory {
            items: vec![None; size],
            size,
            hotbar_size,
            selected_slot: 0,
        }
    }

    pub fn add_item(
        &self,
        inventory: &mut Inventory,
        item_id: u32,
        count: u32,
        metadata: Option<serde_json::Value>,
    ) -> Result<u32, String> {
        let mut remaining = count;

        // First, try to stack with existing items
        for item in inventory.items.iter_mut() {
            if let Some(existing_item) = item {
                if existing_item.id == item_id && existing_item.count < 64 {
                    let space_left = 64 - existing_item.count;
                    let to_add = std::cmp::min(remaining, space_left);
                    existing_item.count += to_add;
                    remaining -= to_add;

                    if remaining == 0 {
                        return Ok(0);
                    }
                }
            }
        }

        // Then, find empty slots
        for (slot, item) in inventory.items.iter_mut().enumerate() {
            if item.is_none() {
                let to_add = std::cmp::min(remaining, 64);
                *item = Some(InventoryItem {
                    id: item_id,
                    count: to_add,
                    metadata: metadata.clone(),
                    slot,
                });
                remaining -= to_add;

                if remaining == 0 {
                    return Ok(0);
                }
            }
        }

        Ok(remaining) // Return remaining items that couldn't be added
    }

    pub fn remove_item(
        &self,
        inventory: &mut Inventory,
        item_id: u32,
        count: u32,
    ) -> Result<u32, String> {
        let mut remaining = count;

        for item in inventory.items.iter_mut() {
            if let Some(existing_item) = item {
                if existing_item.id == item_id {
                    let to_remove = std::cmp::min(remaining, existing_item.count);
                    existing_item.count -= to_remove;
                    remaining -= to_remove;

                    if existing_item.count == 0 {
                        *item = None;
                    }

                    if remaining == 0 {
                        return Ok(0);
                    }
                }
            }
        }

        Ok(remaining) // Return remaining items that couldn't be removed
    }

    pub fn get_item_count(&self, inventory: &Inventory, item_id: u32) -> u32 {
        inventory
            .items
            .iter()
            .filter_map(|item| item.as_ref())
            .filter(|item| item.id == item_id)
            .map(|item| item.count)
            .sum()
    }

    pub fn has_item(&self, inventory: &Inventory, item_id: u32, count: u32) -> bool {
        self.get_item_count(inventory, item_id) >= count
    }

    pub fn get_selected_item(&self, inventory: &Inventory) -> Option<&InventoryItem> {
        if inventory.selected_slot < inventory.hotbar_size {
            inventory.items.get(inventory.selected_slot)?.as_ref()
        } else {
            None
        }
    }

    pub fn set_selected_slot(&self, inventory: &mut Inventory, slot: usize) -> Result<(), String> {
        if slot < inventory.hotbar_size {
            inventory.selected_slot = slot;
            Ok(())
        } else {
            Err("Invalid hotbar slot".to_string())
        }
    }

    pub fn move_item(
        &self,
        inventory: &mut Inventory,
        from_slot: usize,
        to_slot: usize,
    ) -> Result<(), String> {
        if from_slot >= inventory.size || to_slot >= inventory.size {
            return Err("Invalid slot".to_string());
        }

        let temp = inventory.items[from_slot].take();
        inventory.items[from_slot] = inventory.items[to_slot].take();
        inventory.items[to_slot] = temp;

        // Update slot numbers
        if let Some(item) = &mut inventory.items[from_slot] {
            item.slot = from_slot;
        }
        if let Some(item) = &mut inventory.items[to_slot] {
            item.slot = to_slot;
        }

        Ok(())
    }

    pub fn split_stack(
        &self,
        inventory: &mut Inventory,
        slot: usize,
    ) -> Result<(), String> {
        if slot >= inventory.size {
            return Err("Invalid slot".to_string());
        }

        if let Some(item) = &mut inventory.items[slot] {
            if item.count > 1 {
                let half = item.count / 2;
                item.count -= half;

                // Find an empty slot for the split stack
                for (empty_slot, empty_item) in inventory.items.iter_mut().enumerate() {
                    if empty_item.is_none() {
                        *empty_item = Some(InventoryItem {
                            id: item.id,
                            count: half,
                            metadata: item.metadata.clone(),
                            slot: empty_slot,
                        });
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn get_inventory_weight(&self, inventory: &Inventory) -> f32 {
        inventory
            .items
            .iter()
            .filter_map(|item| item.as_ref())
            .map(|item| self.get_item_weight(item.id) * item.count as f32)
            .sum()
    }

    pub fn get_inventory_value(&self, inventory: &Inventory) -> u32 {
        inventory
            .items
            .iter()
            .filter_map(|item| item.as_ref())
            .map(|item| self.get_item_value(item.id) * item.count)
            .sum()
    }

    pub fn clear_inventory(&self, inventory: &mut Inventory) {
        inventory.items.fill(None);
    }

    pub fn serialize_inventory(&self, inventory: &Inventory) -> serde_json::Value {
        serde_json::json!({
            "items": inventory.items,
            "size": inventory.size,
            "hotbar_size": inventory.hotbar_size,
            "selected_slot": inventory.selected_slot
        })
    }

    pub fn deserialize_inventory(&self, data: serde_json::Value) -> Result<Inventory, String> {
        let items = data["items"]
            .as_array()
            .ok_or("Invalid inventory data")?
            .iter()
            .map(|item| {
                if item.is_null() {
                    Ok(None)
                } else {
                    serde_json::from_value(item.clone()).map(Some)
                }
            })
            .collect::<Result<Vec<Option<InventoryItem>>, _>>()?;

        let size = data["size"]
            .as_u64()
            .ok_or("Invalid inventory size")? as usize;
        let hotbar_size = data["hotbar_size"]
            .as_u64()
            .ok_or("Invalid hotbar size")? as usize;
        let selected_slot = data["selected_slot"]
            .as_u64()
            .ok_or("Invalid selected slot")? as usize;

        Ok(Inventory {
            items,
            size,
            hotbar_size,
            selected_slot,
        })
    }

    fn get_item_weight(&self, item_id: u32) -> f32 {
        match item_id {
            1..=5 => 1.0,   // Stone blocks
            17..=21 => 0.5, // Wood
            263..=264 => 0.1, // Coal, Iron
            265..=266 => 0.2, // Gold, Redstone
            267..=268 => 0.3, // Diamond, Emerald
            _ => 0.1, // Default weight
        }
    }

    fn get_item_value(&self, item_id: u32) -> u32 {
        match item_id {
            1..=5 => 1,     // Stone blocks
            17..=21 => 2,   // Wood
            263 => 1,       // Coal
            264 => 5,       // Iron
            265 => 10,      // Gold
            266 => 2,       // Redstone
            267 => 50,      // Diamond
            268 => 30,      // Emerald
            _ => 1,         // Default value
        }
    }
}