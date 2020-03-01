extern crate rand;
extern crate sdl2;

mod actor;
mod fov;
mod items;
#[allow(dead_code)]
mod map;
#[allow(dead_code)]
mod pathfinding;

use crate::actor::Act;
use crate::items::ItemsTable;

use rand::Rng;

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::rc::Rc;

use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Mod;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::ttf::Font;
use sdl2::pixels::Color;

const BACKSPACE_CH: char = '\u{0008}';
const MSG_HISTORY_LENGTH: usize = 50;
const SCREEN_WIDTH: u32 = 49;
const SCREEN_HEIGHT: u32 = 22;
const FOV_WIDTH: usize = 41;
const FOV_HEIGHT: usize = 21;

static BLACK: Color = Color::RGBA(0, 0, 0, 255);
static WHITE: Color = Color::RGBA(255, 255, 255, 255);
static GREY: Color = Color::RGBA(136, 136, 136, 255);
static GREEN: Color = Color::RGBA(46, 139, 87, 255);
static BROWN: Color = Color::RGBA(153, 0, 0, 255);
static BLUE: Color = Color::RGBA(0, 0, 221, 255);
static LIGHT_BLUE: Color = Color::RGBA(55, 198, 255, 255);
static BEIGE: Color = Color::RGBA(255, 178, 127, 255);

type Map = Vec<Vec<map::Tile>>;
type NPCTable = HashMap<(usize, usize), Rc<RefCell<dyn actor::Act>>>;

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
	MsgHistory,
	TmpAsk,
}

// I have literally zero clue why Rust wants two lifetime parameters
// here for the Font ref but this shuts the compiler the hell up...
struct GameUI<'a, 'b> {
	screen_width_px: u32,
	screen_height_px: u32,
	font_width: u32,
	font_height: u32,
	font: &'a Font<'a, 'b>,
	sm_font_width: u32,
	sm_font_height: u32,
	sm_font: &'a Font<'a, 'b>,
	canvas: WindowCanvas,
	event_pump: EventPump,
	curr_msg: String,
	v_matrix: Map,
}

impl<'a, 'b> GameUI<'a, 'b> {
	fn init(font: &'b Font, sm_font: &'b Font) -> Result<GameUI<'a, 'b>, String> {
		let (font_width, font_height) = font.size_of_char(' ').unwrap();
		let screen_width_px = SCREEN_WIDTH * font_width;
		let screen_height_px = SCREEN_HEIGHT * font_height;

		let (sm_font_width, sm_font_height) = sm_font.size_of_char(' ').unwrap();

		let sdl_context = sdl2::init()?;
		let video_subsystem = sdl_context.video()?;
		let window = video_subsystem.window("RL Demo", screen_width_px, screen_height_px)
			.position_centered()
			.opengl()
			.build()
			.map_err(|e| e.to_string())?;

		let v_matrix = vec![vec![map::Tile::Blank; FOV_WIDTH]; FOV_HEIGHT];
		let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
		let mut gui = GameUI { 
			screen_width_px, screen_height_px, 
			font, font_width, font_height, 
			canvas,
			event_pump: sdl_context.event_pump().unwrap(),
			sm_font, sm_font_width, sm_font_height,
			curr_msg: String::from(""),
			v_matrix,
		};

		Ok(gui)
	}

	// I need to handle quitting the app actions here too
	fn wait_for_key_input(&mut self) -> Result<char, String> {
		loop {
			for event in self.event_pump.poll_iter() {
				match event {
					Event::TextInput { text:val, .. } => { 
						let ch = val.as_bytes()[0];
						return Ok(ch as char);
					},
					Event::KeyDown {keycode: Some(Keycode::Return), .. } => return Ok('\n'),
					Event::KeyDown {keycode: Some(Keycode::Backspace), .. } => return Ok(BACKSPACE_CH),
					_ => { continue; }
				}
			}
		}
	}

	fn query_user(&mut self, question: &str) -> String {
		let mut answer = String::from("");

		loop {
			let mut s = String::from(question);
			s.push(' ');
			s.push_str(&answer);

			self.curr_msg = s;
			self.write_screen();

			let ch = self.wait_for_key_input().unwrap();
			match ch {
				'\n' => { break; },
				BACKSPACE_CH => { answer.pop(); },
				_ => { answer.push(ch); },
			}
		}

		answer
	}

	fn get_command(&mut self) -> Cmd {
		loop {
			for event in self.event_pump.poll_iter() {
				match event {
					Event::KeyDown {keycode: Some(Keycode::Escape), ..} 
						| Event::Quit {..} => { return Cmd::Exit },
					Event::KeyDown {keycode: Some(Keycode::H), keymod: Mod::LCTRLMOD, .. } |
					Event::KeyDown {keycode: Some(Keycode::H), keymod: Mod::RCTRLMOD, .. } => { 
						return Cmd::MsgHistory; 
					},
					Event::TextInput { text:val, .. } => {
						if val == "Q" {
							return Cmd::Exit;	
						} else if val == "q" {
							return Cmd::TmpAsk;
						} else if val == "k" {
							return Cmd::MoveN;
						} else if val == "j" {
							return Cmd::MoveS;
						} else if val == "l" {
							return Cmd::MoveE;
						} else if val == "h" {
							return Cmd::MoveW;
						} else if val == "y" {
							return Cmd::MoveNW;
						} else if val == "u" {
							return Cmd::MoveNE;
						} else if val == "b" {
							return Cmd::MoveSW;
						} else if val == "n" {
							return Cmd::MoveSE;
						}
					},
					_ => { continue },
				}
			}
    	}
	}

	fn pause_for_more(&mut self) {
		loop {
			for event in self.event_pump.poll_iter() {
				// I need to handle a Quit/Exit event here	
				match event {
					Event::KeyDown {keycode: Some(Keycode::Escape), ..} |
					Event::KeyDown {keycode: Some(Keycode::Space), ..} => {
						// It seemed like the ' ' event was still in the queue.
						// I guess a TextInput event along with the KeyDown event?
						self.event_pump.poll_event();
						return;
					},
					_ => continue,
				}
			}
		}
	}

	fn write_line(&mut self, row: i32, line: &str, small_font: bool) {
		let fw: u32;
		let fh: u32;	
		let f: &Font;

		if small_font {
			f = self.sm_font;
			fw = self.sm_font_width;
			fh = self.sm_font_height;
		} else {
			f = self.font;
			fw = self.font_width;
			fh = self.font_height;
		}

		if line.len() == 0 {
			self.canvas
				.fill_rect(Rect::new(0, row * fh as i32, self.screen_width_px, fh))
				.expect("Error line!");

			return;
		}

		let surface = f.render(line)
			.blended(WHITE)
			.expect("Error rendering message line!");
		let texture_creator = self.canvas.texture_creator();
		let texture = texture_creator.create_texture_from_surface(&surface)
			.expect("Error create texture for messsage line!");
		let rect = Rect::new(0, row * fh as i32, line.len() as u32 * fw, fh);
		self.canvas.copy(&texture, None, Some(rect))
			.expect("Error copying message line texture to canvas!");
	}

	// What I should do here but am not is make sure each line will fit on the
	// screen without being cut off. For the moment, I just gotta make sure any
	// lines don't have too many characterse. Something for a post 7DRL world
	// I guess.
	fn write_long_msg(&mut self, lines: &Vec<String>) {
		self.canvas.clear();
		
		let display_lines = (self.screen_height_px / self.sm_font_height) as usize;
		let line_count = lines.len();
		let mut curr_line = 0;
		let mut curr_row = 0;
		while curr_line < line_count {
			self.write_line(curr_row as i32, &lines[curr_line], true);
			curr_line += 1;
			curr_row += 1;

			if curr_row == display_lines - 2 && curr_line < line_count {
				self.write_line(curr_row as i32, "", true);
				self.write_line(curr_row as i32 + 1, "-- Press space to continue --", true);
				self.canvas.present();
				self.pause_for_more();
				curr_row = 0;
				self.canvas.clear();
			}
		}

		self.write_line(curr_row as i32, "", true);
		self.write_line(curr_row as i32 + 1, "-- Press space to continue --", true);
		self.canvas.present();
		self.pause_for_more();
	}

	fn write_sq(&mut self, r: usize, c: usize, tile: map::Tile) {
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
			map::Tile::StoneFloor => ('.', GREY),
			map::Tile::Mountain => ('^', GREY),
			map::Tile::SnowPeak => ('^', WHITE),
			map::Tile::Gate => ('#', LIGHT_BLUE),
			map::Tile::Thing(color, ch) => (ch, color),
		};

		let surface = self.font.render_char(ch)
			.blended(char_colour)
			.expect("Error creating character!");  
		let texture_creator = self.canvas.texture_creator();
		let texture = texture_creator.create_texture_from_surface(&surface)
			.expect("Error creating texture!");
		let rect = Rect::new(c as i32 * self.font_width as i32, 
			(r as i32 + 1) * self.font_height as i32, self.font_width, self.font_height);
		self.canvas.copy(&texture, None, Some(rect))
			.expect("Error copying to canvas!");
	}

	fn write_screen(&mut self) {
		self.canvas.set_draw_color(BLACK);
		self.canvas.clear();
		let s = self.curr_msg.clone();
		self.write_line(0, &s, false);
		for row in 0..FOV_HEIGHT {
			for col in 0..FOV_WIDTH {
				self.write_sq(row, col, self.v_matrix[row][col]);
			}
		}

		self.canvas.present();
	}
}

pub struct GameState {
	player_row: usize,
	player_col: usize,
	msg_buff: String,
	msg_history: VecDeque<(String, u32)>,
}

impl GameState {
	pub fn new(r: usize, c: usize) -> GameState {
		GameState {player_row: r, player_col: c, msg_buff: String::from(""),
			msg_history: VecDeque::new() }
	}

	pub fn write_msg_buff(&mut self, msg: &str) {
		self.msg_buff = String::from(msg);

		if msg.len() > 0 {
			if self.msg_history.len() == 0 || msg != self.msg_history[0].0 {
				self.msg_history.push_front((String::from(msg), 1));
			} else {
				self.msg_history[0].1 += 1;
			}

			if self.msg_history.len() > MSG_HISTORY_LENGTH {
				self.msg_history.pop_back();
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

fn do_move(map: &Map, state: &mut GameState, npcs: &NPCTable, items: &ItemsTable, dir: &str) {
	let mv = get_move_tuple(dir);
	let next_row = state.player_row as i16 + mv.0;
	let next_col = state.player_col as i16 + mv.1;
	let tile = map[next_row as usize][next_col as usize];
	
	if npcs.contains_key(&(next_row as usize, next_col as usize)) {
		state.write_msg_buff("There is someone in your way!");
	}
	else if map::is_passable(tile) {
		state.player_col = next_col as usize;
		state.player_row = next_row as usize;

		if tile == map::Tile::Water {
			state.write_msg_buff("You splash in the shallow water.");
		} else {
			state.write_msg_buff("");
		}

		let items_count = items.count_at(state.player_row, state.player_col);
		if items_count == 1 {
			let i = items.get_top(state.player_row, state.player_col);
			let s = format!("You see a {} here.", i.name);
			state.write_msg_buff(&s);
		} else if items_count > 1 {
			state.write_msg_buff("You see a few items here.");
		}	
	} else  {
		if tile == map::Tile::DeepWater {
			state.write_msg_buff("You cannot swim!");
		} else {
			state.write_msg_buff("You cannot go that way.");
		}
	}
}

fn show_message_history(state: &GameState, gui: &mut GameUI) {
	let mut lines = Vec::new();
	lines.push("".to_string());
	for j in 0..state.msg_history.len() {
		let mut s = state.msg_history[j].0.to_string();
		if state.msg_history[j].1 > 1 {
			s.push_str(" (x");
			s.push_str(&state.msg_history[j].1.to_string());
			s.push_str(")");
		}
		lines.push(s);
	}

	gui.write_long_msg(&lines);
}

fn show_intro(gui: &mut GameUI) {
	let mut lines = vec!["Welcome to a rogulike UI prototype!".to_string(), "".to_string()];
	lines.push("You can move around with vi-style keys and bump".to_string());
	lines.push("into water and mountains.".to_string());
	lines.push("".to_string());
	lines.push("There are no monsters or anything yet, though!".to_string());
	
	gui.write_long_msg(&lines);
}

fn test_query(gui: &mut GameUI, state: &GameState) {
	gui.query_user("What is the answer?");
}

fn add_monster(map: &Map, state: &mut GameState, npcs: &mut NPCTable) {
	let mut row = 0;
	let mut col = 0;
	loop {
		row = rand::thread_rng().gen_range(0, map.len());
		col = rand::thread_rng().gen_range(0, map[0].len());

		let tile = map[row][col];
		if map::is_passable(tile) { break; };
	}	
	
	let mut m = actor::Monster::new(13, 25, 'o', row, col, BLUE);
	npcs.insert((row, col), Rc::new(RefCell::new(m)));
}

fn add_test_item(map: &Map, items: &mut ItemsTable) {
	let mut row = 0;
	let mut col = 0;
	loop {
		row = rand::thread_rng().gen_range(0, map.len());
		col = rand::thread_rng().gen_range(0, map[0].len());

		let tile = map[row][col];
		if map::is_passable(tile) { break; };
	}	

	let i = items::Item::new("draught of rum", items::ItemType::Drink, 1,
		'!', BROWN);
	items.add(row, col, i);	

	let i = items::Item::new("rusty cutlass", items::ItemType::Weapon, 3,
		'|', WHITE);
	items.add(row, col, i);	

	let i = items::Item::new("draught of gin", items::ItemType::Weapon, 3,
		'!', WHITE);
	items.add(row, col + 1, i);	
}

fn run(map: &Map) {
    let ttf_context = sdl2::ttf::init()
		.expect("Error creating ttf context on start-up!");
	let font_path: &Path = Path::new("DejaVuSansMono.ttf");
    let font = ttf_context.load_font(font_path, 24)
		.expect("Error loading game font!");
	let sm_font = ttf_context.load_font(font_path, 18)
		.expect("Error loading small game font!");
	let mut gui = GameUI::init(&font, &sm_font)
		.expect("Error initializing GameUI object.");

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

	show_intro(&mut gui);

	let mut lines = vec!["Here is a second page of text".to_string()];
	lines.push("lorem ipsum".to_string());
	lines.push("".to_string());
	lines.push("lorem ipsum".to_string());
	gui.write_long_msg(&lines);
		
	let player_name = gui.query_user("Who are you?");
	
	let mut npcs: NPCTable = HashMap::new();
	add_monster(map, &mut state, &mut npcs);

	let mut items = ItemsTable::new();
	add_test_item(map, &mut items);

	state.write_msg_buff(&format!("Welcome, {}!", player_name));
	gui.curr_msg = state.msg_buff.to_string();
	gui.v_matrix = fov::calc_v_matrix(&map, &npcs, &items,
		state.player_row, state.player_col, FOV_HEIGHT, FOV_WIDTH);
	gui.write_screen();
	
    'mainloop: loop {
		//let mut m = npcs.get(&(17, 17)).unwrap().borrow_mut();
		//let initiative_order = vec![m];

		let mut update = false;
		let cmd = gui.get_command();
		match cmd {
			Cmd::Exit => break 'mainloop,
			Cmd::MoveW => {
				do_move(&map, &mut state, &npcs, &items, "W");
				update = true;
			},
			Cmd::MoveS => {
				do_move(&map, &mut state, &npcs, &items, "S");
				update = true;
			},
			Cmd::MoveN => {
				do_move(&map, &mut state, &npcs, &items, "N");
				update = true;
			},
			Cmd::MoveE => {
				do_move(&map, &mut state, &npcs, &items, "E");
				update = true;
			},
			Cmd::MoveNW => {
				do_move(&map, &mut state, &npcs, &items, "NW");
				update = true;
			},
			Cmd::MoveNE => {
				do_move(&map, &mut state, &npcs, &items, "NE");
				update = true;
			},
			Cmd::MoveSW => {
				do_move(&map, &mut state, &npcs, &items, "SW");
				update = true;
			},
			Cmd::MoveSE => {
				do_move(&map, &mut state, &npcs, &items, "SE");
				update = true;
			},
			Cmd::MsgHistory => {
				show_message_history(&state, &mut gui);
				update = true;
			},

			Cmd::TmpAsk => {
				test_query(&mut gui, &state);
				update = true;
			},
        }
	
		if update {
			gui.v_matrix = fov::calc_v_matrix(&map, &npcs, &items,
				state.player_row, state.player_col, FOV_HEIGHT, FOV_WIDTH);
			gui.curr_msg = state.msg_buff.to_string();
			gui.write_screen();
		}
    }
}

fn main() {
	let map = map::generate_island(65);
	//let map = map::generate_cave(20, 10);
	//let path = pathfinding::find_path(&map, 4, 4, 9, 9);
	
	run(&map);
}
