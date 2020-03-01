use sdl2::pixels::Color;

pub trait Act {
	fn act(&mut self, state: &mut super::GameState);
	fn get_tile_info(&self) -> (Color, char);
}

pub struct Monster {
	ac: u8,
	hp: u8,
	symbol: char,
	row: usize,
	col: usize,
	color: Color,
}

impl Monster {
	pub fn new(ac:u8, hp: u8, symbol: char, row: usize, col: usize, color: Color) -> Monster {
		Monster { ac, hp, symbol, row, col, color }
	}
}

impl Act for Monster {
	fn act(&mut self, state: &mut super::GameState) {
		println!("My location is ({}, {})", self.row, self.col);
	}

	fn get_tile_info(&self) -> (Color, char) {
		(self.color, self.symbol)
	}
}
