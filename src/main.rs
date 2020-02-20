extern crate sdl2;

use std::env;
use std::path::Path;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;
use sdl2::pixels::Color;

static SCREEN_WIDTH : u32 = 800;
static SCREEN_HEIGHT : u32 = 600;

fn run() -> Result<(), String> {
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
    let texture_creator = canvas.texture_creator();

    //font.set_style(sdl2::ttf::FontStyle::BOLD);

    // render a surface, and convert it to a texture bound to the canvas
	let msg = "A maze of twisty passages.";
    let surface = font.render(msg)
        .blended(Color::RGBA(255, 255, 255, 255)).map_err(|e| e.to_string())?;
    let texture = texture_creator.create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
    canvas.clear();

	let rect = Rect::new(0, 0, msg.len() as u32 * font_width, font_height);
    canvas.copy(&texture, None, Some(rect))?;

    canvas.present();

    'mainloop: loop {
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} |
                Event::Quit {..} => break 'mainloop,
                _ => {}
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), String> {
	run()?;

    Ok(())
}

