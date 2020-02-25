extern crate rand;
extern crate sdl2;

mod map;

use rand::Rng;

use std::path::Path;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::ttf::Font;
use sdl2::pixels::Color;

static BLACK: Color = Color::RGBA(0, 0, 0, 255);
static WHITE: Color = Color::RGBA(255, 255, 255, 255);
static GREY: Color = Color::RGBA(136, 136, 136, 255);
static GREEN: Color = Color::RGBA(46, 139, 87, 255);
static BROWN: Color = Color::RGBA(153, 0, 0, 255);
static BLUE: Color = Color::RGBA(0, 0, 221, 255);
static LIGHT_BLUE: Color = Color::RGBA(55, 198, 255, 255);
static BEIGE: Color = Color::RGBA(255, 178, 127, 255);

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

fn write_msg(state: &GameState, canvas: &mut WindowCanvas, font: &Font) -> Result<(), String> {
	canvas.fill_rect(Rect::new(0, 0, 29 * 14, 28));

    let surface = font.render(&state.msg_buff)
        .blended(WHITE).map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;
	let rect = Rect::new(0, 0, state.msg_buff.len() as u32 * 14, 28);
    canvas.copy(&texture, None, Some(rect))?;
	
	Ok(())
}

fn draw_sq(r: usize, c: usize, tile: map::Tile, canvas: &mut WindowCanvas, font: &Font) -> Result<(), String> {
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

	let surface = font.render_char(ch)
		.blended(char_colour).map_err(|e| e.to_string())?;
	let texture_creator = canvas.texture_creator();
	let texture = texture_creator.create_texture_from_surface(&surface)
		.map_err(|e| e.to_string())?;
	let rect = Rect::new(c as i32 * 14, (r as i32 + 1) * 28, 14, 28);
	canvas.copy(&texture, None, Some(rect))?;

	Ok(())
}

// Using bresenham line casting to detect blocked squares. If a ray hits
// a Wall before reaching target then we can't see it. Bresenham isn't 
// really a good way to do this because it leaves blindspots the further
// away you get. But it should suffice for this, where I'm just mucking
// around with displaying via SDL2. For a real game I'll use something
// like shadowcasting, like I did in crashRun.
// (Although honestly for this simple dmeo it seems to work okay! Mind you,
// this is a really inefficient implementation since we visible and mark
// the same squares several times)
fn mark_visible(x1: i32, y1: i32, x2: i32, y2: i32, map: &Vec<Vec<map::Tile>>,
		v_matrix: &mut Vec<Vec<map::Tile>>) {
	let mut x = x1;
	let mut y = y1;
	let mut error = 0;

	let mut x_step = 0;
	let mut delta_x = x2 - x;
	if delta_x < 0 {
		delta_x = -delta_x;
		x_step = -1;
	} else {
		x_step = 1;
	}

	let mut y_step = 0;
	let mut delta_y = y2 - y;
	if delta_y < 0 {
		delta_y = -delta_y;
		y_step = -1;
	} else {
		y_step = 1;
	}

	let mut x_end = x2;
	let mut y_end = y2;
	if delta_y <= delta_x {
		let criterion = delta_x / 2;
		loop {
			if x_step > 0 && x >= x_end + x_step {
				break;
			} else if x_step < 0 && x <= x_end + x_step {
				break;
			}

			if !map::in_bounds(map, x, y) {
				return;
			}

			v_matrix[(x - x1 + 10) as usize][(y - y1 + 20) as usize] = map[x as usize][y as usize];

			if !map::is_clear(map[x as usize][y as usize]) {
				return;
			}

			// I want trees to not totally block light, but instead reduce visibility
			if map::Tile::Tree == map[x as usize][y as usize] {
				if x_step > 0 {
					x_end -= 2;
				} else {
					x_end += 2;
				}
			}

			x += x_step;
			error += delta_y;
			if error > criterion {
				error -= delta_x;
				y += y_step;
			}
		} 	
	} else {
		let criterion = delta_y / 2;
		loop {
			if y_step > 0 && y >= y_end + y_step {
				break;
			} else if y_step < 0 && y <= y_end + y_step {
				break;
			}

			if !map::in_bounds(map, x, y) {
				return;
			}

			v_matrix[(x - x1 + 10) as usize][(y - y1 + 20) as usize] = map[x as usize][y as usize];

			if !map::is_clear(map[x as usize][y as usize]) {
				return;
			}
		
			// I want trees to not totally block light, but instead reduce visibility
			if map[x as usize][y as usize] == map::Tile::Tree {
				if y_step > 0 {
					y_end -= 2;
				} else {
					y_end += 2;
				}
			}
			
			y += y_step;
			error += delta_x;
			if error > criterion {
				error -= delta_y;
				x += x_step;
			}
		}
	}
}

fn draw_dungeon(dungeon: &Vec<Vec<map::Tile>>, canvas: &mut WindowCanvas, font: &Font, state: &GameState) {
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
				actual_r as i32, actual_c as i32, dungeon, &mut v_matrix);
		}
	}
	
	v_matrix[10][20] = map::Tile::Player;
	canvas.fill_rect(Rect::new(0, 28, 49 * 14, 58 * 28));

	for row in 0..21 {
		for col in 0..41 {
			draw_sq(row, col, v_matrix[row][col], canvas, font);
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
		state.write_msg_buff("You cannot go that way.");
	}
}

fn run(dungeon: &Vec<Vec<map::Tile>>) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
	//let font_path: &Path = Path::new("VeraMono.ttf");
	let font_path: &Path = Path::new("DejaVuSansMono.ttf");
    let font = ttf_context.load_font(font_path, 24)?;
	let (font_width, font_height) = font.size_of_char(' ').unwrap();
	let screen_width = 49 * font_width;
	let screen_height = 22 * font_height;

    let window = video_subsystem.window("RL Demo", screen_width, screen_height)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_draw_color(BLACK);
    canvas.clear();

	let mut state = GameState::new(0, 0);
	loop {
		let r = rand::thread_rng().gen_range(1, dungeon.len() - 1);
		let c = rand::thread_rng().gen_range(1, dungeon.len() - 1);
		match dungeon[r][c] {
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
	write_msg(&state, &mut canvas, &font);
	draw_dungeon(dungeon, &mut canvas, &font, &state);
	canvas.present();

    'mainloop: loop {
		let mut update = false;
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} |
                Event::Quit {..} |
				Event::KeyDown {keycode: Some(Keycode::Q), ..} => break 'mainloop,
				Event::KeyDown {keycode: Some(Keycode::H), ..} => {
					do_move(&dungeon, &mut state, "W");
					update = true;
				},
				Event::KeyDown {keycode: Some(Keycode::J), ..} => {
					do_move(&dungeon, &mut state, "S");
					update = true;
				},
				Event::KeyDown {keycode: Some(Keycode::K), ..} => {
					do_move(&dungeon, &mut state, "N");
					update = true;
				},
				Event::KeyDown {keycode: Some(Keycode::L), ..} => {
					do_move(&dungeon, &mut state, "E");
					update = true;
				},
				Event::KeyDown {keycode: Some(Keycode::Y), ..} => {
					do_move(&dungeon, &mut state, "NW");
					update = true;
				},
				Event::KeyDown {keycode: Some(Keycode::U), ..} => {
					do_move(&dungeon, &mut state, "NE");
					update = true;
				},
				Event::KeyDown {keycode: Some(Keycode::B), ..} => {
					do_move(&dungeon, &mut state, "SW");
					update = true;
				},
				Event::KeyDown {keycode: Some(Keycode::N), ..} => {
					do_move(&dungeon, &mut state, "SE");
					update = true;
				},
                _ => {}
            }
        }
	
		if update {
			write_msg(&state, &mut canvas, &font);
			draw_dungeon(dungeon, &mut canvas, &font, &state);
			canvas.present();
		}
    }

    Ok(())
}

fn main() -> Result<(), String> {
	let map = map::generate_island(65);
	run(&map)?;

    Ok(())
}
