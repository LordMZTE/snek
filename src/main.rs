#[macro_use]
extern crate snek;
use crate::logic::Game;
use anyhow::{Context, Result};
use clap::{App, Arg};
use font_kit::{family_name::FamilyName, properties::Properties, source::SystemSource};
use glutin_window::GlutinWindow;
use logic::GameSettings;
use opengl_graphics::{GlGraphics, GlyphCache, OpenGL, TextureSettings};
use piston::{ButtonEvent, EventSettings, Events, RenderEvent, UpdateEvent, WindowSettings};
use snek::load_font_bytes;

pub mod logic;

const ARG_FAIL_MESSAGE: &str = "arg fail";

fn main() -> Result<()> {
    let matches = App::new("Snek")
        .author("LordMZTE")
        .about("Snek game!")
        .arg(
            Arg::with_name("width")
                .short("x")
                .help("the width of the board")
                .takes_value(true)
                .default_value("45"),
        )
        .arg(
            Arg::with_name("height")
                .short("y")
                .help("the height of the board")
                .takes_value(true)
                .default_value("45"),
        )
        .arg(
            Arg::with_name("tile_size")
                .short("t")
                .help("the size that each tile will have")
                .takes_value(true)
                .default_value("20"),
        )
        .arg(
            Arg::with_name("updates_per_move")
                .short("u")
                .help("how many updates it takes for the snek to move 1 tile")
                .takes_value(true)
                .default_value("10"),
        )
        .get_matches();

    let gl = OpenGL::V3_2;
    let size: (u8, u8) = (
        matches
            .value_of("width")
            .context(ARG_FAIL_MESSAGE)?
            .parse()?,
        matches
            .value_of("height")
            .context(ARG_FAIL_MESSAGE)?
            .parse()?,
    );

    let tile_size: u16 = matches
        .value_of("tile_size")
        .context(ARG_FAIL_MESSAGE)?
        .parse()?;

    let mut win: GlutinWindow = WindowSettings::new(
        "Snek",
        (
            size.0 as u32 * (tile_size as u32),
            size.1 as u32 * (tile_size as u32),
        ),
    )
    .graphics_api(gl)
    .exit_on_esc(true)
    .resizable(false)
    .build()
    .unwrap();

    let font_bytes = load_font_bytes(
        SystemSource::new().select_best_match(&[FamilyName::SansSerif], &Properties::new())?,
    )?;

    let glyphs = GlyphCache::from_bytes(
        // include_bytes!("../assets/FiraSans-Regular.ttf"),
        &*font_bytes,
        (),
        TextureSettings::new(),
    )
    .unwrap();

    let mut game = Game::new(GameSettings {
        gl: GlGraphics::new(gl),
        game_size: size,
        glyphs,
        tile_size,
        // TODO add command line args
        updates_per_move: matches
            .value_of("updates_per_move")
            .context(ARG_FAIL_MESSAGE)?
            .parse()?,
    });

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
