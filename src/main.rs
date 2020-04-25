#![allow(dead_code)]

use ggez::{Context, ContextBuilder, GameResult};
use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{self, KeyCode, KeyMods};
use ggez::graphics::{self, Color, DrawParam};

use std::collections::HashMap;
use std::path;
use std::time::Duration;

type Point2 = ggez::nalgebra::Point2<f32>;
type Point2u8 = ggez::nalgebra::Point2<u8>;
type Vec2 = ggez::nalgebra::Vector2<f32>;

const TILE_SIZE: f32 = 20.0;
const TILES_WIDE: usize = 10;
const TILES_HIGH: usize = 20;
const WINDOW_WIDTH: f32 = TILES_WIDE as f32 * TILE_SIZE;
const WINDOW_HEIGHT: f32 = TILES_HIGH as f32 * TILE_SIZE;

struct Assets {
    block_sprites: HashMap<TetType, graphics::Image>,
}

impl Assets {
    fn load(ctx: &mut Context) -> GameResult<Assets> {
        let mut block_sprites = HashMap::new();
        block_sprites.insert(TetType::I, graphics::Image::new(ctx, "/i.png")?);
        block_sprites.insert(TetType::J, graphics::Image::new(ctx, "/j.png")?);
        block_sprites.insert(TetType::L, graphics::Image::new(ctx, "/l.png")?);
        block_sprites.insert(TetType::O, graphics::Image::new(ctx, "/o.png")?);
        block_sprites.insert(TetType::S, graphics::Image::new(ctx, "/s.png")?);
        block_sprites.insert(TetType::T, graphics::Image::new(ctx, "/t.png")?);
        block_sprites.insert(TetType::Z, graphics::Image::new(ctx, "/z.png")?);
        Ok(Assets { block_sprites })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum TetType {
    I, J, L, O, S, T, Z,
}

struct Tet {
    tet_type: TetType,
    blocks: [Point2u8; 4],
    pos: Point2u8,
}

impl Tet {
    fn new(tet_type: TetType, pos: Point2u8) -> Self {
        Self {
            tet_type,
            pos,
            blocks: match tet_type {
                TetType::I => [[0, 0].into(), [1, 0].into(), [2, 0].into(), [3, 0].into()],
                TetType::J => [[0, 0].into(), [0, 1].into(), [1, 1].into(), [2, 1].into()],
                TetType::L => [[0, 1].into(), [1, 1].into(), [2, 1].into(), [2, 0].into()],
                TetType::O => [[0, 0].into(), [0, 1].into(), [1, 0].into(), [1, 1].into()],
                TetType::S => [[0, 1].into(), [1, 1].into(), [1, 0].into(), [2, 0].into()],
                TetType::T => [[1, 0].into(), [0, 1].into(), [1, 1].into(), [2, 1].into()],
                TetType::Z => [[0, 0].into(), [1, 0].into(), [1, 1].into(), [2, 1].into()],
            }
        }
    }

    fn left(&self) -> u8 {
        self.blocks.iter().min_by(|b1, b2| b1.x.cmp(&b2.x)).unwrap().x
    }

    fn right(&self) -> u8 {
        self.blocks.iter().max_by(|b1, b2| b1.x.cmp(&b2.x)).unwrap().x
    }

    fn bottom(&self) -> u8 {
        self.blocks.iter().max_by(|b1, b2| b1.y.cmp(&b2.y)).unwrap().y
    }

    fn fall(&mut self, tets: &Tets) -> bool {
        for block in self.blocks.iter() {
            if self.pos.y + block.y + 1 >= TILES_HIGH as u8 ||
               tets.at(self.pos.y + block.y + 1, self.pos.x + block.x).is_some() {
                return false;
            }
        }
        self.pos.y += 1;
        true
    }

    fn move_left(&mut self, tets: &Tets) {
        for block in self.blocks.iter() {
            if self.pos.x + block.x == 0 ||
               tets.at(self.pos.y + block.y, self.pos.x + block.x - 1).is_some() {
                return;
            }
        }
        self.pos.x -= 1;
    }

    fn move_right(&mut self, tets: &Tets) {
        for block in self.blocks.iter() {
            if self.pos.x + block.x + 1 >= TILES_WIDE as u8 ||
               tets.at(self.pos.y + block.y, self.pos.x + block.x + 1).is_some() {
                return;
            }
        }
        self.pos.x += 1;
    }
}

struct Tets {
    tets: [[Option<TetType>; TILES_WIDE]; TILES_HIGH]
}

impl Default for Tets {
    fn default() -> Tets {
        Tets { tets: [[None; TILES_WIDE]; TILES_HIGH] }
    }
}

impl Tets {
    fn at(&self, row: u8, col: u8) -> &Option<TetType> {
        self.tets.get(row as usize)
            .map_or(&None, |row| row.get(col as usize)
            .map_or(&None, |block| block))
    }

    fn set(&mut self, row: u8, col: u8, val: TetType) {
        self.tets[row as usize][col as usize] = Some(val);
    }

    fn clear(&mut self, row: u8) {
        self.tets[row as usize] = [None; TILES_WIDE];
    }

    fn iter(&self) -> std::slice::Iter<[Option<TetType>; TILES_WIDE]> {
        self.tets.iter()
    }
}

#[derive(Debug)]
enum Mode {
    Normal,
    SoftDrop,
}

struct Game {
    assets: Assets,
    current_tet: Tet,
    tets: Tets,
    fall_timer: Duration,
    fall_mode: Mode,
}

impl Game {
    const NORMAL_INTERVAL: Duration = Duration::from_secs(1);
    const SOFT_DROP_INTERVAL: Duration = Duration::from_millis(200);

    fn new(ctx: &mut Context) -> GameResult<Self> {
        let game = Game {
            assets: Assets::load(ctx)?,
            current_tet: Tet::new(TetType::S, Point2u8::new(3, 0)),
            tets: Tets::default(),
            fall_timer: Self::NORMAL_INTERVAL,
            fall_mode: Mode::Normal,
        };

        Ok(game)
    }

    fn new_tet(&mut self) {
        for block in self.current_tet.blocks.iter() {
            self.tets.set(
                self.current_tet.pos.y + block.y,
                self.current_tet.pos.x + block.x,
                self.current_tet.tet_type
            );
        }
        self.current_tet = Tet::new(TetType::S, Point2u8::new(3, 4));
    }

    fn hard_drop(&mut self) {
        while self.current_tet.fall(&self.tets) {}
        self.new_tet();
        self.fall_timer = self.fall_interval();
    }

    fn fall_interval(&self) -> Duration {
        match self.fall_mode {
            Mode::Normal => Self::NORMAL_INTERVAL,
            Mode::SoftDrop => Self::SOFT_DROP_INTERVAL,
        }
    }
}

enum TimerState {
    Ticking(Duration),
    Done(Duration),
}

fn decrement(lhs: Duration, rhs: Duration, reset: Duration) -> TimerState {
    if lhs < rhs || lhs - rhs == Duration::from_millis(0) {
        TimerState::Done(reset - (rhs - lhs))
    } else {
        TimerState::Ticking(lhs - rhs)
    }
}

impl event::EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;

        while ggez::timer::check_update_time(ctx, DESIRED_FPS) {
            // Update
        }

        self.fall_timer = match decrement(self.fall_timer, ggez::timer::delta(ctx), self.fall_interval()) {
            TimerState::Ticking(time) => time,
            TimerState::Done(time) => {
                if !self.current_tet.fall(&self.tets) {
                    self.new_tet();
                }
                time
            },
        };

        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods, repeat: bool) {
        match keycode {
            KeyCode::Escape => event::quit(ctx),
            KeyCode::Left => self.current_tet.move_left(&self.tets),
            KeyCode::Right => self.current_tet.move_right(&self.tets),
            KeyCode::Up => self.hard_drop(),
            KeyCode::Down => {
                if !repeat {
                    self.fall_mode = Mode::SoftDrop;
                    self.fall_timer = Duration::from_secs(0);
                }
            },
            _ => ()
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        match keycode {
            KeyCode::Down => self.fall_mode = Mode::Normal,
            _ => ()
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::from_rgb(0, 0, 0));

        for (y, row) in self.tets.iter().enumerate() {
            for (x, block) in row.iter().enumerate() {
                if let Some(block) = block {
                    graphics::draw(
                        ctx,
                        &self.assets.block_sprites[&block],
                        DrawParam::default()
                            .dest(Point2::new(
                                TILE_SIZE * x as f32,
                                TILE_SIZE * y as f32,
                            ))
                    )?;
                }
            }
        }
        for block in self.current_tet.blocks.iter() {
            graphics::draw(
                ctx,
                &self.assets.block_sprites[&self.current_tet.tet_type],
                DrawParam::default()
                    .dest(Point2::new(
                        TILE_SIZE * (self.current_tet.pos.x + block.x) as f32,
                        TILE_SIZE * (self.current_tet.pos.y + block.y) as f32,
                    ))
            )?;
        }

        graphics::present(ctx)?;
        Ok(())
    }
}

fn main() -> GameResult {
    let resource_dir = path::PathBuf::from("./res");

    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new("tetris", "peter")
        .window_setup(WindowSetup::default().title("tetris"))
        .window_mode(WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT))
        .add_resource_path(resource_dir)
        .build()?;

    let mut game = Game::new(ctx)?;
    event::run(ctx, event_loop, &mut game)
}
