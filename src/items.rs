use std::collections::{HashMap, VecDeque};

use sdl2::pixels::Color;

pub trait TileInfo {
	fn get_tile_info(&self) -> (Color, char);
}

pub enum ItemType {
	Weapon,
	Clothing,
	Drink,
}

pub struct ItemsTable {
	table: HashMap<(usize, usize), VecDeque<Item>>,
}

impl ItemsTable {
	pub fn new() -> ItemsTable {
		ItemsTable { table: HashMap::new() }
	}

	pub fn add(&mut self, r: usize, c: usize, item: Item) {
		if !self.table.contains_key(&(r, c)) {
			self.table.insert((r, c,), VecDeque::new());
		}

		let stack = self.table.get_mut(&(r, c)).unwrap();
		stack.push_front(item);
	}

	pub fn count_at(&self, r: usize, c: usize) -> u8 {
		let res = if !self.table.contains_key(&(r, c)) {
			0
		} else {
			self.table[&(r, c)].len()
		};

		res as u8
	}

	pub fn get_top(&self, r: usize, c: usize) -> &Item {
		let stack = self.table.get(&(r, c)).unwrap();
		stack.front().unwrap()
	}
}

pub struct Item {
	pub name: String,
	pub item_type: ItemType,
	pub weight: u8,
	symbol: char,
	color: Color,
}

impl Item {
	pub fn new(name: &str, item_type: ItemType, w: u8, sym: char, color: Color) -> Item {
		Item { name: String::from(name), 
			item_type, weight: w, symbol: sym, color }
	}
}

impl TileInfo for Item {
	// basically a duplicate of the same method for the Act trait in actor.rs
	// but I don't think having my NPCs list in the main program be a vec of TileInfos
	// insteaf of Act will work for the purposes I want to use it for ;/
	fn get_tile_info(&self) -> (Color, char) {
		(self.color, self.symbol)
	}
}
