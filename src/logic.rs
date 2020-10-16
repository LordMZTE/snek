use std::{collections::LinkedList, convert::TryFrom, ops::Not};

use graphics::{math::Matrix2d, types::Color, Transformed};
use opengl_graphics::{GlGraphics, GlyphCache};
use piston::{Button, ButtonArgs, ButtonState, Key, RenderArgs, UpdateArgs};
use rand::{prelude::ThreadRng, Rng};

const BACKGROUND: Color = [0., 0., 0., 1.];
const SNEK_COLOR: Color = [1., 0., 0., 1.];
const OUT_OF_BOUNDS_COLOR: Color = [0., 0., 1., 1.];
const APPLE_COLOR: Color = [0., 1., 0., 1.];

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GameState {
    Lost,
    Running,
    Paused,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Right,
    Left,
}

impl Direction {
    pub fn move_pos(&self, distance: i8, x: &mut u8, y: &mut u8) {
        match self {
            Direction::Up => *y = y.overflowing_sub(distance as u8).0,
            Direction::Down => *y = y.overflowing_add(distance as u8).0,
            Direction::Right => *x = x.overflowing_add(distance as u8).0,
            Direction::Left => *x = x.overflowing_sub(distance as u8).0,
        }
    }
}

impl TryFrom<&Key> for Direction {
    type Error = ();

    fn try_from(value: &Key) -> Result<Self, Self::Error> {
        match value {
            Key::W | Key::Up => Ok(Direction::Up),
            Key::A | Key::Left => Ok(Direction::Left),
            Key::S | Key::Down => Ok(Direction::Down),
            Key::D | Key::Right => Ok(Direction::Right),
            _ => Err(()),
        }
    }
}

impl Not for &Direction {
    type Output = Direction;

    fn not(self) -> Self::Output {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Right => Direction::Left,
            Direction::Left => Direction::Right,
        }
    }
}

pub struct GameSettings<'a> {
    pub gl: GlGraphics,
    pub game_size: (u8, u8),
    pub glyphs: GlyphCache<'a>,
    pub updates_per_move: u8,
    pub tile_size: u16,
}

fn draw_tile(
    gl: &mut GlGraphics,
    transform: Matrix2d,
    tile_size: u16,
    pos: (u8, u8),
    color: Color,
) {
    graphics::rectangle(
        color,
        graphics::rectangle::square(
            (pos.0 as u16 * tile_size) as f64,
            (pos.1 as u16 * tile_size) as f64,
            tile_size as f64,
        ),
        transform,
        gl,
    );
}

pub struct Game<'a> {
    pub gl: GlGraphics,
    snek: Snek,
    pub apple_pos: Option<(u8, u8)>,
    pub apple_rand: ThreadRng,
    pub game_size: (u8, u8),
    pub state: GameState,
    pub glyphs: GlyphCache<'a>,
    pub updates_per_move: u8,
    pub tile_size: u16,
}

impl<'a> Game<'a> {
    pub fn new(sets: GameSettings<'a>) -> Self {
        Self {
            updates_per_move: sets.updates_per_move,
            tile_size: sets.tile_size,
            gl: sets.gl,
            game_size: sets.game_size,
            glyphs: sets.glyphs,
            state: GameState::Paused,
            apple_rand: rand::thread_rng(),
            snek: Snek::new(sets.updates_per_move),
            apple_pos: Default::default(),
        }
    }

    pub fn render(&mut self, args: &RenderArgs) {
        let size = self.game_size;
        let apple = self.apple_pos;
        let snek = &mut self.snek;
        let glyphs = &mut self.glyphs;
        let state = self.state;
        let tile_size = self.tile_size;

        self.gl.draw(args.viewport(), |c, g| {
            // clear
            graphics::clear(OUT_OF_BOUNDS_COLOR, g);
            // background
            graphics::rectangle(
                BACKGROUND,
                [
                    0.,
                    0.,
                    ((size.0 as u16) * tile_size) as f64,
                    ((size.1 as u16) * tile_size) as f64,
                ],
                c.transform,
                g,
            );
            // apple
            if let Some(a) = apple {
                draw_tile(g, c.transform, tile_size, a, APPLE_COLOR);
            }

            // snek
            snek.render(g, &args, tile_size);

            // score
            graphics::text(
                [1., 1., 1., 1.],
                32,
                format!("Score: {}", snek.segs.len()).as_str(),
                glyphs,
                c.transform.trans(10.0, 50.0),
                g,
            )
            .unwrap();

            // game over
            match state {
                GameState::Lost => graphics::text(
                    [1., 1., 1., 1.],
                    32,
                    "Game Over!",
                    glyphs,
                    c.transform.trans(10.0, 100.0),
                    g,
                )
                .unwrap(),
                GameState::Paused => graphics::text(
                    [1., 1., 1., 1.],
                    32,
                    "Paused",
                    glyphs,
                    c.transform.trans(10.0, 100.0),
                    g,
                )
                .unwrap(),
                _ => {},
            }
        });
    }

    pub fn update(&mut self, _args: &UpdateArgs) {
        if let GameState::Running = self.state {
            if self.apple_pos.is_none() {
                self.randomize_apple();
            }

            // if snek.update returns true, we gotta randomize the apple again
            let snek_update = self.snek.update(self.apple_pos.unwrap(), self.game_size);
            match snek_update {
                (_, true) => self.state = GameState::Lost,
                (true, _) => self.randomize_apple(),
                _ => {},
            }
        }
    }

    pub fn keypress(&mut self, btn: &ButtonArgs) {
        if let (ButtonState::Press, Button::Keyboard(k)) = (btn.state, btn.button) {
            if let Ok(d) = Direction::try_from(&k) {
                // this check is to prevent the snek from turning around
                if !&d != self.snek.dir {
                    self.snek.next_dir = d;
                }
            }

            match k {
                Key::Space => match self.state {
                    GameState::Running => self.state = GameState::Paused,
                    GameState::Paused => self.state = GameState::Running,
                    _ => {},
                },
                Key::R => {
                    self.state = GameState::Paused;
                    self.snek = Snek::new(self.updates_per_move);
                    self.randomize_apple();
                },
                _ => {},
            }
        }
    }

    fn randomize_apple(&mut self) {
        self.apple_pos = None;
        while self.apple_pos.is_none() || self.snek.check_collides(self.apple_pos.unwrap()) {
            self.apple_pos = Some((
                self.apple_rand.gen_range(0, self.game_size.0),
                self.apple_rand.gen_range(0, self.game_size.1),
            ));
        }
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
struct SnekSeg(pub u8, pub u8);

impl SnekSeg {
    pub fn move_dir(&self, dist: i8, dir: &Direction) -> SnekSeg {
        let mut seg = self.clone();
        dir.move_pos(dist, &mut seg.0, &mut seg.1);
        seg
    }
}

impl Into<(u8, u8)> for &SnekSeg {
    fn into(self) -> (u8, u8) {
        (self.0, self.1)
    }
}

impl From<(u8, u8)> for SnekSeg {
    fn from(t: (u8, u8)) -> Self {
        SnekSeg(t.0, t.1)
    }
}

#[derive(Debug)]
struct Snek {
    pub segs: LinkedList<SnekSeg>,
    pub dir: Direction,
    pub next_dir: Direction,
    pub move_counter: u8,
    pub updates_per_move: u8,
}

impl Snek {
    pub fn new(updates_per_move: u8) -> Self {
        Self {
            updates_per_move,
            segs: linked_list![SnekSeg(0, 0)],
            dir: Direction::Right,
            next_dir: Direction::Right,
            move_counter: 0,
        }
    }
    pub fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs, tile_size: u16) {
        let iter = self.segs.iter();

        gl.draw(args.viewport(), |c, gl| {
            for s in iter {
                draw_tile(gl, c.transform, tile_size, s.into(), SNEK_COLOR);
            }
        });
    }

    /// returns (apple_eaten, game_lost)
    pub fn update(&mut self, apple_pos: (u8, u8), board_size: (u8, u8)) -> (bool, bool) {
        let mut ate_apple = false;
        let mut lost = false;
        self.move_counter += 1;
        if self.move_counter >= self.updates_per_move {
            self.move_counter = 0;

            self.dir = self.next_dir;

            // look at first element, move forward 1 and then add that to front
            let mut moved = self
                .segs
                .front()
                .expect("snek has no body")
                .move_dir(1, &self.dir);

            // if the SnekSeg is outside the game board, wrap around to the other side
            if moved.0 == 255 {
                moved.0 = board_size.0 - 1;
            }

            if moved.1 == 255 {
                moved.1 = board_size.1 - 1;
            }

            if moved.0 >= board_size.0 {
                moved.0 = 0;
            }

            if moved.1 >= board_size.1 {
                moved.1 = 0;
            }

            // check if snek collides with self
            lost = self.segs.iter().any(|s| s == &moved);

            self.segs.push_front(moved);

            ate_apple = self.segs.front().expect("snek has no body") == &SnekSeg::from(apple_pos);
            if !ate_apple {
                self.segs.pop_back();
            }
        }

        (ate_apple, lost)
    }

    /// returns true if any part of the snek intersects with the apple
    pub fn check_collides(&self, pos: (u8, u8)) -> bool {
        self.segs.iter().any(|s| s == &SnekSeg::from(pos))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn direction_add() {
        let (mut x, mut y) = (0, 0);
        Direction::Down.move_pos(5, &mut x, &mut y);
        assert_eq!((0, 5), (x, y));
    }

    #[test]
    fn direction_add_negative() {
        let (mut x, mut y) = (0, 0);
        Direction::Up.move_pos(-5, &mut x, &mut y);
        assert_eq!((0, 5), (x, y));
    }
}
