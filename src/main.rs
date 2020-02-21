extern crate rand;
extern crate sdl2;

use rand::Rng;

use std::cell::Cell;
use std::env;
use std::fs;
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
static DARK_GREY: Color = Color::RGBA(85, 85, 85, 255);
static GREEN: Color = Color::RGBA(46, 139, 87, 255);
static BROWN: Color = Color::RGBA(153, 0, 0, 255);
static BLUE: Color = Color::RGBA(0, 0, 221, 255);

#[derive(Debug, Clone, Copy)]
enum Tile {
	Blank,
	Wall,
	Tree,
	Dirt,
	Grass,
	Player,
	Water,
}

struct GameState {
	player_row: usize,
	player_col: usize,
}

impl GameState {
	fn new(r: usize, c: usize) -> GameState {
		GameState {player_row: r, player_col: c }
	}
}

fn make_rando_test_dungeon() -> Vec<Vec<Tile>> {
	let mut dungeon = vec![vec![Tile::Wall; 30]];

	for _ in 0..28 {
		let mut row = vec![Tile::Wall];
		for _ in 0..28 {
			let x = rand::thread_rng().gen_range(0, 3);
			if x == 0 {
				row.push(Tile::Tree);
			} else if x == 1 {
				row.push(Tile::Grass);
			} else {
				row.push(Tile::Dirt);
			}
		}
		row.push(Tile::Wall);
		dungeon.push(row);
	}

	for _ in 0..3 {
		let r = rand::thread_rng().gen_range(5, 24);
		let cc = rand::thread_rng().gen_range(5, 24);

		for rd in -1..2 {
			let cs = cc - rand::thread_rng().gen_range(2, 5);
			let ce = cc + rand::thread_rng().gen_range(2, 5);
			for c in cs..ce+1 {
				dungeon[(r + rd as i32) as usize][c] = Tile::Water;
			}
		}	
	}

	for _ in 0..3 {
		let r = rand::thread_rng().gen_range(5, 25);
		let c = rand::thread_rng().gen_range(5, 25);
		let n = rand::thread_rng().gen_range(0, 2);
		if n == 0 {
			for c2 in c..c+4 {
				dungeon[r][c2] = Tile::Wall;
			}
		} else {
			for r2 in r..r+4 {
				dungeon[r2][c] = Tile::Wall;
			}
		}
	}

	dungeon[1][4] = Tile::Wall;
	dungeon[2][4] = Tile::Wall;
	dungeon[3][4] = Tile::Wall;
	dungeon[4][4] = Tile::Wall;

	dungeon.push(vec![Tile::Wall; 30]);

	dungeon
}

fn write_msg(msg: &str, canvas: &mut WindowCanvas, font: &Font) -> Result<(), String> {
	canvas.fill_rect(Rect::new(0, 0, 29 * 14, 28));

    let surface = font.render(msg)
        .blended(WHITE).map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;
	let rect = Rect::new(0, 0, msg.len() as u32 * 14, 28);
    canvas.copy(&texture, None, Some(rect))?;
	
	Ok(())
}

fn draw_sq(r: usize, c: usize, tile: Tile, canvas: &mut WindowCanvas, font: &Font) -> Result<(), String> {
	let (ch, char_colour) = match tile {
		Tile::Blank => (' ', BLACK),
		Tile::Wall => ('#', GREY),
		Tile::Tree => ('#', GREEN),
		Tile::Dirt => ('.', BROWN),
		Tile::Grass => ('.', GREEN),
		Tile::Player => ('@', WHITE),
		Tile::Water => ('}', BLUE),
	};

	let surface = font.render(&ch.to_string())
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
fn mark_visible(x1: i32, y1: i32, x2: i32, y2: i32, dungeon: &Vec<Vec<Tile>>,
		v_matrix: &mut Vec<Vec<Tile>>) {
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

	if delta_y <= delta_x {
		let criterion = delta_x / 2;
		while x != x2 + x_step {
			v_matrix[(x - x1 + 10) as usize][(y - y1 + 10) as usize] = dungeon[x as usize][y as usize];
			if let Tile::Wall = dungeon[x as usize][y as usize] {
				return;
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
		while y != y2 + y_step {
			v_matrix[(x - x1 + 10) as usize][(y - y1 + 10) as usize] = dungeon[x as usize][y as usize];
			if let Tile::Wall = dungeon[x as usize][y as usize] {
				return;
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

fn draw_dungeon(dungeon: &Vec<Vec<Tile>>, canvas: &mut WindowCanvas, font: &Font, state: &GameState) -> Result<(), String> {
	// create a matrix of tiles to display, starting off with blanks and then we'll fill
	// in the squares that are actually visible.
	let mut v_matrix: Vec<Vec<Tile>> = Vec::new();
	for _ in 0..21 {
		v_matrix.push(vec![Tile::Blank; 21]);
	}

	for row in -10..11 {
		for col in -10..11 {
			let actual_r: i32 = state.player_row as i32 + row;
			let actual_c: i32 = state.player_col as i32 + col;

			mark_visible(state.player_row as i32, state.player_col as i32,
				actual_r as i32, actual_c as i32, dungeon, &mut v_matrix);
		}
	}

	v_matrix[10][10] = Tile::Player;
	canvas.fill_rect(Rect::new(0, 28, 39 * 14, 38 * 28));

	for row in 0..21 {
		for col in 0..21 {
			draw_sq(row, col, v_matrix[row][col], canvas, font);
		}
	}

	Ok(())
}

fn run(dungeon: &Vec<Vec<Tile>>) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
	let font_path: &Path = Path::new("VeraMono.ttf");
    let font = ttf_context.load_font(font_path, 24)?;
	let (font_width, font_height) = font.size_of_char(' ').unwrap();
	let screen_width = 29 * font_width;
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
		let r = rand::thread_rng().gen_range(1, 29);
		let c = rand::thread_rng().gen_range(1, 29);
		match dungeon[r][c] {
			Tile::Water | Tile::Wall => { continue; },
			_ => {
				state.player_row = r;
				state.player_col = c;
				break;
			}
		}
	}
	
	let mut msg_buff = "A roguelike demo...";
	write_msg(msg_buff, &mut canvas, &font);
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
					let tile = dungeon[state.player_row][state.player_col - 1];
					match tile {
						Tile::Water => { msg_buff = "You cannot swim."},
						Tile::Wall | Tile::Blank => { msg_buff = "Ouch!"},
						_ => { 
							state.player_col -= 1;
							msg_buff = "";
						},
					}

					update = true;
				},
				Event::KeyDown {keycode: Some(Keycode::J), ..} => {
					let tile = dungeon[state.player_row + 1][state.player_col];
					match tile {
						Tile::Water => { msg_buff = "You cannot swim."},
						Tile::Wall | Tile::Blank => { msg_buff = "You bump into a wall!"},
						_ => { 
							state.player_row += 1;
							msg_buff = "";
						},
					}

					update = true;
				},
				Event::KeyDown {keycode: Some(Keycode::K), ..} => {
					let tile = dungeon[state.player_row - 1][state.player_col];
					match tile {
						Tile::Water => { msg_buff = "You cannot swim."},
						Tile::Wall | Tile::Blank => { msg_buff = "You cannot go that way."},
						_ => { 
							state.player_row -= 1;
							msg_buff = "";
						},
					}

					update = true;
				},
				Event::KeyDown {keycode: Some(Keycode::L), ..} => {
					let tile = dungeon[state.player_row][state.player_col + 1];
					match tile {
						Tile::Water => { msg_buff = "You cannot swim."},
						Tile::Wall | Tile::Blank => { msg_buff = "Impassable!"},
						_ => { 
							state.player_col += 1;
							msg_buff = "";
						},
					}

					update = true;
				},
				Event::KeyDown {keycode: Some(Keycode::Y), ..} => {
					let tile = dungeon[state.player_row - 1][state.player_col - 1];
					match tile {
						Tile::Water => { msg_buff = "You cannot swim."},
						Tile::Wall | Tile::Blank => { msg_buff = "Impassable!"},
						_ => { 
							state.player_col -= 1;
							state.player_row -= 1;
							msg_buff = "";
						},
					}

					update = true;
				},
				Event::KeyDown {keycode: Some(Keycode::U), ..} => {
					let tile = dungeon[state.player_row - 1][state.player_col + 1];
					match tile {
						Tile::Water => { msg_buff = "You cannot swim."},
						Tile::Wall | Tile::Blank => { msg_buff = "Impassable!"},
						_ => { 
							state.player_col += 1;
							state.player_row -= 1;
							msg_buff = "";
						},
					}

					update = true;
				},
				Event::KeyDown {keycode: Some(Keycode::B), ..} => {
					let tile = dungeon[state.player_row + 1][state.player_col - 1];
					match tile {
						Tile::Water => { msg_buff = "You cannot swim."},
						Tile::Wall | Tile::Blank => { msg_buff = "Your way is blocked."},
						_ => { 
							state.player_col -= 1;
							state.player_row += 1;
							msg_buff = "";
						},
					}

					update = true;
				},
				Event::KeyDown {keycode: Some(Keycode::N), ..} => {
					let tile = dungeon[state.player_row + 1][state.player_col + 1];
					match tile {
						Tile::Water => { msg_buff = "You cannot swim."},
						Tile::Wall | Tile::Blank => { msg_buff = "Your way is blocked."},
						_ => { 
							state.player_col += 1;
							state.player_row += 1;
							msg_buff = "";
						},
					}

					update = true;
				},
                _ => {}
            }
        }
	
		if update {
			write_msg(msg_buff, &mut canvas, &font);
			draw_dungeon(dungeon, &mut canvas, &font, &state);
			canvas.present();
		}
    }

    Ok(())
}

fn main() -> Result<(), String> {
	let dungeon = make_rando_test_dungeon();
	run(&dungeon)?;

    Ok(())
}
