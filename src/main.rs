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

#[derive(Debug, Clone, Copy)]
enum Tile {
	Blank,
	Wall,
	Tree,
	Dirt,
	Player,
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
			if rand::thread_rng().gen_range(0, 2) == 0 {
				row.push(Tile::Tree);
			} else {
				row.push(Tile::Dirt);
			}
		}
		row.push(Tile::Wall);
		dungeon.push(row);
	}
	
	dungeon.push(vec![Tile::Wall; 30]);

	dungeon
}

fn write_msg(msg: &str, canvas: &mut WindowCanvas, font: &Font) -> Result<(), String> {
    canvas.set_draw_color(BLACK);
	canvas.fill_rect(Rect::new(0, 0, 29 * 14, 28));

    let surface = font.render(msg)
        .blended(WHITE).map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;
    canvas.set_draw_color(BLACK);
	let rect = Rect::new(0, 0, msg.len() as u32 * 14, 28);
    canvas.copy(&texture, None, Some(rect))?;

    canvas.present();

	Ok(())
}

fn draw_sq(r: usize, c: usize, tile: Tile, canvas: &mut WindowCanvas, font: &Font) -> Result<(), String> {
	let (ch, char_colour) = match tile {
		Tile::Blank => (' ', BLACK),
		Tile::Wall => ('#', GREY),
		Tile::Tree => ('#', GREEN),
		Tile::Dirt => ('.' ,BROWN),
		Tile::Player => ('@' ,WHITE),
	};

	let surface = font.render(&ch.to_string())
		.blended(char_colour).map_err(|e| e.to_string())?;
	let texture_creator = canvas.texture_creator();
	let texture = texture_creator.create_texture_from_surface(&surface)
		.map_err(|e| e.to_string())?;
	canvas.set_draw_color(BLACK);
	let rect = Rect::new(c as i32 * 14, (r as i32 + 1) * 28, 14, 28);
	canvas.copy(&texture, None, Some(rect))?;

	Ok(())
}

fn draw_dungeon(dungeon: &Vec<Vec<Tile>>, canvas: &mut WindowCanvas, font: &Font, state: &GameState) -> Result<(), String> {
	// instead of clear the canvas, draw a fill_rect to preserve the message line
	canvas.clear();
	for row in -5..5 {
		for col in -5..5 {
			let actual_r: i32 = state.player_row as i32 + row;
			let actual_c: i32 = state.player_col as i32 + col;
			let tile = if row == 0 && col == 0 {
				Tile::Player
			} else if actual_r < 0 || actual_c < 0 || actual_r >= 30 || actual_c >= 30 {
				Tile::Blank
			} else {
				dungeon[actual_r as usize][actual_c as usize]
			};

			draw_sq((row + 5) as usize, (col + 5) as usize, tile, canvas, font);
		}
	}
	canvas.present();

	Ok(())
}

fn run(dungeon: &Vec<Vec<Tile>>) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
	let font_path: &Path = Path::new("VeraMono.ttf");
    let font = ttf_context.load_font(font_path, 24)?;
	let (font_width, font_height) = font.size_of_char(' ').unwrap();
	let screen_width = 29 * font_width;
	let screen_height = 12 * font_height;

    let window = video_subsys.window("Roguelike UI Demo", screen_width, screen_height)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.clear();

	let msg = "A maze of twisty passages...";
	write_msg(msg, &mut canvas, &font);
	let mut state = GameState::new(1, 1);
	draw_dungeon(dungeon, &mut canvas, &font, &state);
	canvas.present();

    'mainloop: loop {
		let mut update = false;
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} |
                Event::Quit {..} => break 'mainloop,
				Event::KeyDown {keycode: Some(Keycode::Space), ..} => {
					write_msg("...all alike.", &mut canvas, &font);
				},
				Event::KeyDown {keycode: Some(Keycode::H), ..} => {
					state.player_col -= 1;
					update = true;
					write_msg("move west...", &mut canvas, &font);
				},
				Event::KeyDown {keycode: Some(Keycode::J), ..} => {
					state.player_row += 1;
					update = true;
					write_msg("move south...", &mut canvas, &font);
				},
				Event::KeyDown {keycode: Some(Keycode::K), ..} => {
					state.player_row -= 1;
					update = true;
					write_msg("move north...", &mut canvas, &font);
				},
				Event::KeyDown {keycode: Some(Keycode::L), ..} => {
					state.player_col += 1;
					update = true;
					write_msg("move east...", &mut canvas, &font);
				},
                _ => {}
            }
        }
	
		if update {
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
