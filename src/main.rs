extern crate rand;
extern crate sdl2;

mod map;

use rand::Rng;

use std::path::Path;

use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::ttf::Font;
use sdl2::pixels::Color;

const SCREEN_WIDTH: u32 = 49;
const SCREEN_HEIGHT: u32 = 22;

static BLACK: Color = Color::RGBA(0, 0, 0, 255);
static WHITE: Color = Color::RGBA(255, 255, 255, 255);
static GREY: Color = Color::RGBA(136, 136, 136, 255);
static GREEN: Color = Color::RGBA(46, 139, 87, 255);
static BROWN: Color = Color::RGBA(153, 0, 0, 255);
static BLUE: Color = Color::RGBA(0, 0, 221, 255);
static LIGHT_BLUE: Color = Color::RGBA(55, 198, 255, 255);
static BEIGE: Color = Color::RGBA(255, 178, 127, 255);

enum Cmd {
	Exit,
	MoveN,
	MoveS,
	MoveE,
	MoveW,
	MoveNW,
	MoveNE,
	MoveSW,
	MoveSE,
}

// I have literally zero clue why Rust wants two lifetime parameters
// here for the Font ref but this shuts the compiler the hell up...
struct GameUI<'a, 'b> {
	screen_width_px: u32,
	screen_height_px: u32,
	font_width: u32,
	font_height: u32,
	font: &'a Font<'a, 'b>,
	canvas: WindowCanvas,
	event_pump: EventPump,
}

impl<'a, 'b> GameUI<'a, 'b> {
	fn init(font: &'b Font) -> Result<GameUI<'a, 'b>, String> {
		let (font_width, font_height) = font.size_of_char(' ').unwrap();
		let screen_width_px = SCREEN_WIDTH * font_width;
		let screen_height_px = SCREEN_HEIGHT * font_height;

		let sdl_context = sdl2::init()?;
		let video_subsystem = sdl_context.video()?;
		let window = video_subsystem.window("RL Demo", screen_width_px, screen_height_px)
			.position_centered()
			.opengl()
			.build()
			.map_err(|e| e.to_string())?;

		let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
		let mut gui = GameUI { 
			screen_width_px, screen_height_px, 
			font_width, font_height, 
			font: font, canvas,
			event_pump: sdl_context.event_pump().unwrap(), 
		};

		gui.canvas.set_draw_color(BLACK);
		gui.canvas.clear();

		Ok(gui)
	}

	fn draw(&mut self) {
		self.canvas.present();
	}

	fn get_command(&mut self) -> Cmd {
		loop {
			for event in self.event_pump.poll_iter() {
				match event {
					Event::KeyDown {keycode: Some(Keycode::Escape), ..} |
					Event::Quit {..} |
					Event::KeyDown {keycode: Some(Keycode::Q), ..} => { return Cmd::Exit },
					Event::KeyDown {keycode: Some(Keycode::H), ..} => { return Cmd::MoveW },
					Event::KeyDown {keycode: Some(Keycode::J), ..} => { return Cmd::MoveS },
					Event::KeyDown {keycode: Some(Keycode::K), ..} => { return Cmd::MoveN },
					Event::KeyDown {keycode: Some(Keycode::L), ..} => { return Cmd::MoveE },
					Event::KeyDown {keycode: Some(Keycode::Y), ..} => { return Cmd::MoveNW },
					Event::KeyDown {keycode: Some(Keycode::U), ..} => { return Cmd::MoveNE },
					Event::KeyDown {keycode: Some(Keycode::B), ..} => { return Cmd::MoveSW },
					Event::KeyDown {keycode: Some(Keycode::N), ..} => { return Cmd::MoveSE },
					_ => { continue },
				}
			}
    	}
	}

	fn write_msg(&mut self, state: &GameState) -> Result<(), String> {
		self.canvas.fill_rect(Rect::new(0, 0, self.screen_width_px, self.font_height));

		let surface = self.font.render(&state.msg_buff)
			.blended(WHITE).map_err(|e| e.to_string())?;
		let texture_creator = self.canvas.texture_creator();
		let texture = texture_creator.create_texture_from_surface(&surface)
			.map_err(|e| e.to_string())?;
		let rect = Rect::new(0, 0, state.msg_buff.len() as u32 * self.font_width, self.font_height);
		self.canvas.copy(&texture, None, Some(rect))?;
		
		Ok(())
	}

	fn write_sq(&mut self, r: usize, c: usize, tile: map::Tile) -> Result<(), String> {
		let (ch, char_colour) = match tile {
			map::Tile::Blank => (' ', BLACK),
			map::Tile::Wall => ('#', GREY),
			map::Tile::Tree => ('\u{03D9}', GREEN),
			map::Tile::Dirt => ('.', BROWN),
			map::Tile::Grass => ('\u{0316}', GREEN),
			map::Tile::Player => ('@', WHITE),
			map::Tile::Water => ('}', LIGHT_BLUE),
			map::Tile::DeepWater => ('}', BLUE),
			map::Tile::Sand => ('.', BEIGE),
			map::Tile::Mountain => ('^', GREY),
			map::Tile::SnowPeak => ('^', WHITE),
		};

		let surface = self.font.render_char(ch)
			.blended(char_colour).map_err(|e| e.to_string())?;
		let texture_creator = self.canvas.texture_creator();
		let texture = texture_creator.create_texture_from_surface(&surface)
			.map_err(|e| e.to_string())?;
		let rect = Rect::new(c as i32 * self.font_width as i32, 
			(r as i32 + 1) * self.font_height as i32, self.font_width, self.font_height);
		self.canvas.copy(&texture, None, Some(rect))?;

		Ok(())
	}

	fn write_map_to_screen(&mut self, map: &Vec<Vec<map::Tile>>, state: &GameState) {
		// create a matrix of tiles to display, starting off with blanks and then we'll fill
		// in the squares that are actually visible.
		let mut v_matrix: Vec<Vec<map::Tile>> = Vec::new();
		for _ in 0..21 {
			v_matrix.push(vec![map::Tile::Blank; 41]);
		}

		for row in -10..11 {
			for col in -20..21 {
				let actual_r: i32 = state.player_row as i32 + row;
				let actual_c: i32 = state.player_col as i32 + col;

				mark_visible(state.player_row as i32, state.player_col as i32,
					actual_r as i32, actual_c as i32, map, &mut v_matrix);
			}
		}
		
		v_matrix[10][20] = map::Tile::Player;
		self.canvas.fill_rect(
			Rect::new(0, self.font_height as i32, self.screen_width_px, self.screen_height_px));

		for row in 0..21 {
			for col in 0..41 {
				self.write_sq(row, col, v_matrix[row][col]);
			}
		}
	}

}

struct GameState {
	player_row: usize,
	player_col: usize,
	msg_buff: String,
}

impl GameState {
	fn new(r: usize, c: usize) -> GameState {
		GameState {player_row: r, player_col: c, msg_buff: String::from("") }
	}

	fn write_msg_buff(&mut self, msg: &str) {
		self.msg_buff = String::from(msg);
	}
}
/*
fn write_screen(lines: Vec<String>, ui_info: &mut GameUI) -> Result<(), String> {
	ui_info.canvas.fill_rect(Rect::new(0, 0, ui_info.screen_width_px, ui_info.screen_height_px);

	Ok(())
}
*/

// Using bresenham line casting to detect blocked squares. If a ray hits
// a Wall before reaching target then we can't see it. Bresenham isn't 
// really a good way to do this because it leaves blindspots the further
// away you get. But it should suffice for this, where I'm just mucking
// around with displaying via SDL2. For a real game I'll use something
// like shadowcasting, like I did in crashRun.
// (Although honestly for this simple dmeo it seems to work okay! Mind you,
// this is a really inefficient implementation since we visible and mark
// the same squares several times)
fn mark_visible(r1: i32, c1: i32, r2: i32, c2: i32, map: &Vec<Vec<map::Tile>>,
		v_matrix: &mut Vec<Vec<map::Tile>>) {
	let mut r = r1;
	let mut c = c1;
	let mut error = 0;

	let mut r_step = 1;
	let mut delta_r = r2 - r;
	if delta_r < 0 {
		delta_r = -delta_r;
		r_step = -1;
	} 

	let mut c_step = 1;
	let mut delta_c = c2 - c;
	if delta_c < 0 {
		delta_c = -delta_c;
		c_step = -1;
	} 

	let mut r_end = r2;
	let mut c_end = c2;
	if delta_c <= delta_r {
		let criterion = delta_r / 2;
		loop {
			if r_step > 0 && r >= r_end + r_step {
				break;
			} else if r_step < 0 && r <= r_end + r_step {
				break;
			}

			if !map::in_bounds(map, r, c) {
				return;
			}

			v_matrix[(r - r1 + 10) as usize][(c - c1 + 20) as usize] = map[r as usize][c as usize];

			if !map::is_clear(map[r as usize][c as usize]) {
				return;
			}

			// I want trees to not totally block light, but instead reduce visibility
			if map::Tile::Tree == map[r as usize][c as usize] && !(r == r1 && c == c1) {
				if r_step > 0 {
					r_end -= 3;
				} else {
					r_end += 3;
				}
			}

			r += r_step;
			error += delta_c;
			if error > criterion {
				error -= delta_r;
				c += c_step;
			}
		} 	
	} else {
		let criterion = delta_c / 2;
		loop {
			if c_step > 0 && c >= c_end + c_step {
				break;
			} else if c_step < 0 && c <= c_end + c_step {
				break;
			}

			if !map::in_bounds(map, r, c) {
				return;
			}

			v_matrix[(r - r1 + 10) as usize][(c - c1 + 20) as usize] = map[r as usize][c as usize];

			if !map::is_clear(map[r as usize][c as usize]) {
				return;
			}
		
			// I want trees to not totally block light, but instead reduce visibility
			if map::Tile::Tree == map[r as usize][c as usize] && !(r == r1 && c == c1) {
				if c_step > 0 {
					c_end -= 3;
				} else {
					c_end += 3;
				}
			}
			
			c += c_step;
			error += delta_r;
			if error > criterion {
				error -= delta_c;
				r += r_step;
			}
		}
	}
}

fn get_move_tuple(mv: &str) -> (i16, i16) {
	let res: (i16, i16);

  	if mv == "N" {
		res = (-1, 0)
	} else if mv == "S" {
		res = (1, 0)
	} else if mv == "W" {
		res = (0, -1)
	} else if mv == "E" {
		res = (0, 1)
	} else if mv == "NW" {
		res = (-1, -1)
	} else if mv == "NE" {
		res = (-1, 1)
	} else if mv == "SW" {
		res = (1, -1)
	} else {
		res = (1, 1)
	}

	res
}

fn do_move(map: &Vec<Vec<map::Tile>>, state: &mut GameState, dir: &str) {
	let mv = get_move_tuple(dir);
	let next_row = state.player_row as i16 + mv.0;
	let next_col = state.player_col as i16 + mv.1;
	let tile = map[next_row as usize][next_col as usize];
	if map::is_passable(tile) {
		state.player_col = next_col as usize;
		state.player_row = next_row as usize;
		state.write_msg_buff("");
	} else  {
		if tile == map::Tile::DeepWater {
			state.write_msg_buff("You cannot swim!");
		} else {
			state.write_msg_buff("You cannot go that way.");
		}
	}
}

fn run(map: &Vec<Vec<map::Tile>>) -> Result<(), String> {
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
	let font_path: &Path = Path::new("DejaVuSansMono.ttf");
    let font = ttf_context.load_font(font_path, 24)?;

	let mut gui = GameUI::init(&font)?;


	let mut state = GameState::new(0, 0);
	loop {
		let r = rand::thread_rng().gen_range(1, map.len() - 1);
		let c = rand::thread_rng().gen_range(1, map.len() - 1);
		match map[r][c] {
			map::Tile::Water | map::Tile::Wall | map::Tile::DeepWater |
			map::Tile::Mountain | map::Tile::SnowPeak => { continue; },
			_ => {
				state.player_row = r;
				state.player_col = c;
				break;
			}
		}
	}
	
	state.write_msg_buff("A roguelike demo...");
	gui.write_msg(&state);
	gui.write_map_to_screen(map, &state);
	gui.draw();

    'mainloop: loop {
		let mut update = false;
		let cmd = gui.get_command();
		match cmd {
			Cmd::Exit => break 'mainloop,
			Cmd::MoveW => {
				do_move(&map, &mut state, "W");
				update = true;
			},
			Cmd::MoveS => {
				do_move(&map, &mut state, "S");
				update = true;
			},
			Cmd::MoveN => {
				do_move(&map, &mut state, "N");
				update = true;
			},
			Cmd::MoveE => {
				do_move(&map, &mut state, "E");
				update = true;
			},
			Cmd::MoveNW => {
				do_move(&map, &mut state, "NW");
				update = true;
			},
			Cmd::MoveNE => {
				do_move(&map, &mut state, "NE");
				update = true;
			},
			Cmd::MoveSW => {
				do_move(&map, &mut state, "SW");
				update = true;
			},
			Cmd::MoveSE => {
				do_move(&map, &mut state, "SE");
				update = true;
			},
        }
	
		if update {
			gui.write_msg(&state);
			gui.write_map_to_screen(map, &state);
			gui.draw();
		}
    }

    Ok(())
}

fn main() -> Result<(), String> {
	let map = map::generate_island(65);
	run(&map)?;

    Ok(())
}
