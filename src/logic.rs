use std::{collections::LinkedList, convert::TryFrom, ops::Not};

use graphics::{types::Color, Transformed};
use opengl_graphics::{GlGraphics, GlyphCache};
use piston::{Button, Key, RenderArgs, UpdateArgs, ButtonArgs, ButtonState};
use rand::{prelude::ThreadRng, Rng};

const BACKGROUND: Color = [0., 0., 0., 1.];
const SNEK_COLOR: Color = [1., 0., 0., 1.];
const OUT_OF_BOUNDS_COLOR: Color = [0., 0., 1., 1.];
const APPLE_COLOR: Color = [0., 1., 0., 1.];
const UPDATES_PER_MOVE: u8 = 10;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GameState {
    Lost,
    Running,
    Paused,
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Running
    }
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

pub struct Game<'a> {
    pub gl: GlGraphics,
    snek: Snek,
    pub apple_pos: Option<(u8, u8)>,
    pub apple_rand: ThreadRng,
    pub game_size: (u8, u8),
    pub state: GameState,
    pub glyphs: GlyphCache<'a>,
}

impl<'a> Game<'a> {
    pub fn new(gl: GlGraphics, game_size: (u8, u8), glyphs: GlyphCache<'a>) -> Self {
        Self {
            gl,
            game_size,
            glyphs,
            apple_rand: rand::thread_rng(),
            snek: Default::default(),
            apple_pos: Default::default(),
            state: Default::default(),
        }
    }

    pub fn render(&mut self, args: &RenderArgs) {
        let size = self.game_size;
        let apple = self.apple_pos;
        let snek = &mut self.snek;
        let glyphs = &mut self.glyphs;
        let state = self.state;

        self.gl.draw(args.viewport(), |c, g| {
            // clear
            graphics::clear(OUT_OF_BOUNDS_COLOR, g);
            // background
            graphics::rectangle(
                BACKGROUND,
                [
                    0.,
                    0.,
                    ((size.0 as u16) * 20) as f64,
                    ((size.1 as u16) * 20) as f64,
                ],
                c.transform,
                g,
            );
            // apple
            if let Some(a) = apple {
                graphics::rectangle(
                    APPLE_COLOR,
                    [
                        ((a.0 as u16) * 20) as f64,
                        ((a.1 as u16) * 20) as f64,
                        20.,
                        20.,
                    ],
                    c.transform,
                    g,
                );
            }

            // snek
            snek.render(g, &args);

            // score
            graphics::text(
                [1., 1., 1., 1.],
                32,
                format!("Score: {}", snek.segs.len()).as_str(),
                glyphs,
                c.transform.trans(10.0, 50.0),
                g,
            ).unwrap();

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
                _ => {}
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

            if let Key::Space = k {
                match self.state {
                    GameState::Running => self.state = GameState::Paused,
                    GameState::Paused => self.state = GameState::Running,
                    _ => {},
                }
            }
        }
    }

    fn randomize_apple(&mut self) {
        while self.apple_pos.is_none() || self.snek.check_collides(self.apple_pos.unwrap()) {
            self.apple_pos = Some((
                self.apple_rand.gen_range(0, self.game_size.0),
                self.apple_rand.gen_range(0, self.game_size.1),
            ));
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
struct SnekSeg(pub u8, pub u8);

impl SnekSeg {
    pub fn move_dir(&self, dist: i8, dir: &Direction) -> SnekSeg {
        let mut seg = self.clone();
        dir.move_pos(dist, &mut seg.0, &mut seg.1);
        seg
    }
}

impl PartialEq<(u8, u8)> for &SnekSeg {
    fn eq(&self, other: &(u8, u8)) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

#[derive(SmartDefault, Debug)]
struct Snek {
    #[default(linked_list![
        SnekSeg(0, 0),
    ])]
    pub segs: LinkedList<SnekSeg>,
    #[default(Direction::Right)]
    pub dir: Direction,
    #[default(Direction::Right)]
    pub next_dir: Direction,
    pub move_counter: u8,
}

impl Snek {
    pub fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs) {
        let rects = self.segs.iter().map(|s| {
            // type cast to u16 to prevent overflow when multiplying
            graphics::rectangle::square((s.0 as u16 * 20) as f64, (s.1 as u16 * 20) as f64, 20.)
        });

        gl.draw(args.viewport(), |c, gl| {
            for rect in rects {
                graphics::rectangle(SNEK_COLOR, rect, c.transform, gl)
            }
        });
    }

    /// returns (apple_eaten, game_lost)
    pub fn update(&mut self, apple_pos: (u8, u8), board_size: (u8, u8)) -> (bool, bool) {
        let mut ate_apple = false;
        let mut lost = false;
        self.move_counter += 1;
        if self.move_counter >= UPDATES_PER_MOVE {
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
            lost = self.segs.iter().any(|s| s == (moved.0, moved.1));

            self.segs.push_front(moved);

            ate_apple = self.segs.front().expect("snek has no body") == apple_pos;
            if !ate_apple {
                self.segs.pop_back();
            }
        }

        (ate_apple, lost)
    }

    /// returns true if any part of the snek intersects with the apple
    pub fn check_collides(&self, pos: (u8, u8)) -> bool {
        self.segs.iter().any(|s| s == pos)
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
