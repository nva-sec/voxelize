use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use log::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftingRecipe {
    pub id: String,
    pub name: String,
    pub ingredients: Vec<CraftingIngredient>,
    pub result: CraftingResult,
    pub crafting_table: bool,
    pub shapeless: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftingIngredient {
    pub item_id: u32,
    pub count: u32,
    pub position: Option<(u8, u8)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftingResult {
    pub item_id: u32,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: u32,
    pub count: u32,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug)]
pub struct CraftingSystem {
    recipes: HashMap<String, CraftingRecipe>,
    shapeless_recipes: Vec<CraftingRecipe>,
}

impl CraftingSystem {
    pub fn new() -> Self {
        let mut system = Self {
            recipes: HashMap::new(),
            shapeless_recipes: Vec::new(),
        };
        
        system.initialize_default_recipes();
        system
    }

    pub fn add_recipe(&mut self, recipe: CraftingRecipe) {
        if recipe.shapeless {
            self.shapeless_recipes.push(recipe.clone());
        } else {
            self.recipes.insert(recipe.id.clone(), recipe);
        }
    }

    pub fn get_recipe(&self, recipe_id: &str) -> Option<&CraftingRecipe> {
        self.recipes.get(recipe_id)
    }

    pub fn get_all_recipes(&self) -> Vec<&CraftingRecipe> {
        let mut all_recipes: Vec<&CraftingRecipe> = self.recipes.values().collect();
        all_recipes.extend(self.shapeless_recipes.iter());
        all_recipes
    }

    pub fn find_matching_recipe(
        &self,
        ingredients: &[[Option<u32>; 3]; 3],
        use_crafting_table: bool,
    ) -> Option<&CraftingRecipe> {
        // Check shaped recipes first
        for recipe in self.recipes.values() {
            if recipe.crafting_table && !use_crafting_table {
                continue;
            }
            
            if self.matches_shaped_recipe(recipe, ingredients) {
                return Some(recipe);
            }
        }

        // Check shapeless recipes
        for recipe in &self.shapeless_recipes {
            if recipe.crafting_table && !use_crafting_table {
                continue;
            }
            
            if self.matches_shapeless_recipe(recipe, ingredients) {
                return Some(recipe);
            }
        }

        None
    }

    pub fn craft_item(
        &self,
        inventory: &mut Vec<InventoryItem>,
        recipe: &CraftingRecipe,
    ) -> Result<Option<InventoryItem>, String> {
        // Check if we have all ingredients
        if !self.has_ingredients(inventory, recipe) {
            return Err("Not enough ingredients".to_string());
        }

        // Consume ingredients
        self.consume_ingredients(inventory, recipe)?;

        // Create result item
        let result_item = InventoryItem {
            id: recipe.result.item_id,
            count: recipe.result.count,
            metadata: None,
        };

        // Add to inventory
        self.add_item_to_inventory(inventory, result_item.clone())?;

        Ok(Some(result_item))
    }

    fn matches_shaped_recipe(
        &self,
        recipe: &CraftingRecipe,
        ingredients: &[[Option<u32>; 3]; 3],
    ) -> bool {
        for ingredient in &recipe.ingredients {
            if let Some((x, y)) = ingredient.position {
                if x >= 3 || y >= 3 {
                    return false;
                }
                
                match ingredients[y as usize][x as usize] {
                    Some(item_id) if item_id == ingredient.item_id => {
                        // Check count if needed
                    }
                    _ => return false,
                }
            }
        }
        true
    }

    fn matches_shapeless_recipe(
        &self,
        recipe: &CraftingRecipe,
        ingredients: &[[Option<u32>; 3]; 3],
    ) -> bool {
        let mut available_ingredients: Vec<u32> = Vec::new();
        
        // Collect all non-empty ingredients
        for row in ingredients {
            for item in row {
                if let Some(item_id) = item {
                    available_ingredients.push(item_id);
                }
            }
        }

        // Check if we have all required ingredients
        for ingredient in &recipe.ingredients {
            let required_count = ingredient.count as usize;
            let available_count = available_ingredients
                .iter()
                .filter(|&&id| id == ingredient.item_id)
                .count();
            
            if available_count < required_count {
                return false;
            }
        }

        true
    }

    fn has_ingredients(
        &self,
        inventory: &[InventoryItem],
        recipe: &CraftingRecipe,
    ) -> bool {
        for ingredient in &recipe.ingredients {
            let available_count: u32 = inventory
                .iter()
                .filter(|item| item.id == ingredient.item_id)
                .map(|item| item.count)
                .sum();
            
            if available_count < ingredient.count {
                return false;
            }
        }
        true
    }

    fn consume_ingredients(
        &self,
        inventory: &mut Vec<InventoryItem>,
        recipe: &CraftingRecipe,
    ) -> Result<(), String> {
        for ingredient in &recipe.ingredients {
            let mut remaining = ingredient.count;
            
            for item in inventory.iter_mut() {
                if item.id == ingredient.item_id && remaining > 0 {
                    let consume_amount = std::cmp::min(remaining, item.count);
                    item.count -= consume_amount;
                    remaining -= consume_amount;
                    
                    if item.count == 0 {
                        // Remove empty items
                        inventory.retain(|i| i.count > 0);
                    }
                    
                    if remaining == 0 {
                        break;
                    }
                }
            }
            
            if remaining > 0 {
                return Err(format!("Not enough of item {}", ingredient.item_id));
            }
        }
        Ok(())
    }

    fn add_item_to_inventory(
        &self,
        inventory: &mut Vec<InventoryItem>,
        new_item: InventoryItem,
    ) -> Result<(), String> {
        // Try to stack with existing items
        for item in inventory.iter_mut() {
            if item.id == new_item.id {
                item.count += new_item.count;
                return Ok(());
            }
        }
        
        // Add as new item
        inventory.push(new_item);
        Ok(())
    }

    fn initialize_default_recipes(&mut self) {
        // Wooden Planks
        self.add_recipe(CraftingRecipe {
            id: "wooden_planks".to_string(),
            name: "Wooden Planks".to_string(),
            ingredients: vec![
                CraftingIngredient {
                    item_id: 17, // Oak Log
                    count: 1,
                    position: None,
                }
            ],
            result: CraftingResult {
                item_id: 5, // Oak Planks
                count: 4,
            },
            crafting_table: false,
            shapeless: true,
        });

        // Crafting Table
        self.add_recipe(CraftingRecipe {
            id: "crafting_table".to_string(),
            name: "Crafting Table".to_string(),
            ingredients: vec![
                CraftingIngredient {
                    item_id: 5, // Oak Planks
                    count: 4,
                    position: None,
                }
            ],
            result: CraftingResult {
                item_id: 58, // Crafting Table
                count: 1,
            },
            crafting_table: false,
            shapeless: true,
        });

        // Wooden Pickaxe
        self.add_recipe(CraftingRecipe {
            id: "wooden_pickaxe".to_string(),
            name: "Wooden Pickaxe".to_string(),
            ingredients: vec![
                CraftingIngredient {
                    item_id: 5, // Oak Planks
                    count: 3,
                    position: Some((0, 0)),
                },
                CraftingIngredient {
                    item_id: 280, // Stick
                    count: 2,
                    position: Some((1, 1)),
                }
            ],
            result: CraftingResult {
                item_id: 270, // Wooden Pickaxe
                count: 1,
            },
            crafting_table: true,
            shapeless: false,
        });

        // Stick
        self.add_recipe(CraftingRecipe {
            id: "stick".to_string(),
            name: "Stick".to_string(),
            ingredients: vec![
                CraftingIngredient {
                    item_id: 5, // Oak Planks
                    count: 2,
                    position: None,
                }
            ],
            result: CraftingResult {
                item_id: 280, // Stick
                count: 4,
            },
            crafting_table: false,
            shapeless: true,
        });

        info!("Initialized {} crafting recipes", self.recipes.len() + self.shapeless_recipes.len());
    }
}