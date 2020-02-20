extern crate sdl2;

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

struct DisplayInfo {
	scr_width: u32,
	scr_height: u32,
}

impl DisplayInfo {
	fn new(w: u32, h: u32) -> DisplayInfo {
		DisplayInfo  {scr_width: w, scr_height: h }
	}
}

fn fetch_dungeon() -> Vec<Vec<char>> {
	let mut grid = Vec::new();
	let txt = fs::read_to_string("dungeon.txt").unwrap();

	for line in txt.split('\n') {
		grid.push(
			line.trim()
				.chars()
				.map(|ch| ch)
				.collect::<Vec<char>>());
	}

	grid
}

fn write_msg(msg: &str, canvas: &mut WindowCanvas, font: &Font, di: &DisplayInfo) -> Result<(), String> {
	println!("{}", msg);

    canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
	canvas.fill_rect(Rect::new(0, 0, di.scr_width * 14, 28));

    let surface = font.render(msg)
        .blended(Color::RGBA(255, 255, 255, 255)).map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;
    canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
	let rect = Rect::new(0, 0, msg.len() as u32 * 14, 28);
    canvas.copy(&texture, None, Some(rect))?;

    canvas.present();

	Ok(())
}

fn draw_dungeon(dungeon: &Vec<Vec<char>>, canvas: &mut WindowCanvas, font: &Font, di: &DisplayInfo) -> Result<(), String> {
	let mut rc = 1;
	for row in dungeon {
		let line: String = row.into_iter().collect();
		let surface = font.render(&line)
        	.blended(Color::RGBA(255, 255, 255, 255)).map_err(|e| e.to_string())?;
		let texture_creator = canvas.texture_creator();
		let texture = texture_creator.create_texture_from_surface(&surface)
			.map_err(|e| e.to_string())?;
		canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
		let rect = Rect::new(0, rc * 28, line.len() as u32 * 14, 28);
		rc += 1;
		canvas.copy(&texture, None, Some(rect))?;
	}
	canvas.present();

	Ok(())
}

fn run(dungeon: &Vec<Vec<char>>) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
	let font_path: &Path = Path::new("VeraMono.ttf");
    let mut font = ttf_context.load_font(font_path, 24)?;
	let (font_width, font_height) = font.size_of_char(' ').unwrap();
	println!("{}, {}", font_width, font_height);
	let screen_width = 26 * font_width;
	let screen_height = 11 * font_height;

    let window = video_subsys.window("Roguelike UI Demo", screen_width, screen_height)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.clear();

    //font.set_style(sdl2::ttf::FontStyle::BOLD);

    // render a surface, and convert it to a texture bound to the canvas
	let di = DisplayInfo::new(screen_width, screen_height);
	let msg = "A maze of twisty passages...";

	write_msg(msg, &mut canvas, &font, &di);
	draw_dungeon(dungeon, &mut canvas, &font, &di);
	canvas.present();

    'mainloop: loop {
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} |
                Event::Quit {..} => break 'mainloop,
				Event::KeyDown {keycode: Some(Keycode::Space), ..} => {
					write_msg("...all alike.", &mut canvas, &font, &di);
				},
				Event::KeyDown {keycode: Some(Keycode::H), ..} => {
					write_msg("move west...", &mut canvas, &font, &di);
				},
				Event::KeyDown {keycode: Some(Keycode::J), ..} => {
					write_msg("move north...", &mut canvas, &font, &di);
				},
				Event::KeyDown {keycode: Some(Keycode::K), ..} => {
					write_msg("move south...", &mut canvas, &font, &di);
				},
				Event::KeyDown {keycode: Some(Keycode::L), ..} => {
					write_msg("move east...", &mut canvas, &font, &di);
				},
                _ => {}
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), String> {
	let dungeon = fetch_dungeon();
	println!("{:?}", dungeon);
	run(&dungeon)?;

    Ok(())
}
