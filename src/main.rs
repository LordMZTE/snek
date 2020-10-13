#[macro_use]
extern crate smart_default;
#[macro_use]
extern crate snek;
use crate::logic::Game;
use anyhow::{Context, Result};
use clap::{App, Arg};
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, GlyphCache, OpenGL, TextureSettings};
use piston::{ButtonEvent, EventSettings, Events, RenderEvent, UpdateEvent, WindowSettings};

pub mod logic;
fn main() -> Result<()> {
    let matches = App::new("Snek")
        .author("LordMZTE")
        .about("Snek game!")
        .arg(
            Arg::with_name("width")
                .short("w")
                .help("the width of the board")
                .takes_value(true)
                .default_value("45"),
        )
        .arg(
            Arg::with_name("height")
                .short("h")
                .help("the height of the board")
                .takes_value(true)
                .default_value("45"),
        )
        .get_matches();

    let gl = OpenGL::V3_2;
    let size: (u8, u8) = (
        matches.value_of("width").context("arg fail")?.parse()?,
        matches.value_of("height").context("arg fail")?.parse()?,
    );

    let mut win: GlutinWindow =
        WindowSettings::new("Snek", (size.0 as u32 * 20, size.1 as u32 * 20))
            .graphics_api(gl)
            .exit_on_esc(true)
            .resizable(false)
            .build()
            .unwrap();

    let glyphs = GlyphCache::from_bytes(
        include_bytes!("../assets/FiraSans-Regular.ttf"),
        (),
        TextureSettings::new(),
    )
    .unwrap();

    let mut game = Game::new(GlGraphics::new(gl), size, glyphs);

    let mut events = Events::new(EventSettings::new());
    while let Some(ev) = events.next(&mut win) {
        if let Some(a) = ev.render_args() {
            game.render(&a);
        }

        if let Some(a) = ev.update_args() {
            game.update(&a);
        }

        if let Some(k) = ev.button_args() {
            game.keypress(&k);
        }
    }

    Ok(())
}
