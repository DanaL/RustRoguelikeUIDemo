use sdl2::pixels::Color;

pub enum ItemType {
	Weapon,
	Clothing,
	Drink,
}

pub struct Item {
	name: String,
	item_type: ItemType,
	weight: u8,
	symbol: char,
	color: Color,
}

impl Item {
	pub fn new(name: &str, item_type: ItemType, w: u8, sym: char, color: Color) -> Item {
		Item { name: String::from(name), 
			item_type, weight: w, symbol: sym, color }
	}

	// I wonder if I should implement a Trait like HasTileInfo since there's a 
	// copy of this function in Act Trait in actor.rs
	pub fn get_tile_info(&self) -> (Color, char) {
		(self.color, self.symbol)
	}
}
