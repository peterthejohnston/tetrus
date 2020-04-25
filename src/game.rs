use std::collections::HashMap;
use std::time::Duration;

use ggez::{Context, GameResult};
use ggez::event::{self, KeyCode, KeyMods};
use ggez::graphics::{self, Color, DrawParam};

use crate::tet::{Tet, TetType};

type Point2f32 = ggez::nalgebra::Point2<f32>;
type Point2 = ggez::nalgebra::Point2<i8>;

pub const TILE_SIZE: f32 = 20.0;
pub const TILES_WIDE: usize = 10;
pub const TILES_HIGH: usize = 20;

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

pub struct Tets {
    tets: [[Option<TetType>; TILES_WIDE]; TILES_HIGH]
}

impl Default for Tets {
    fn default() -> Tets {
        Tets { tets: [[None; TILES_WIDE]; TILES_HIGH] }
    }
}

impl Tets {
    pub fn at(&self, row: i8, col: i8) -> &Option<TetType> {
        self.tets.get(row as usize)
            .map_or(&None, |row| row.get(col as usize)
            .map_or(&None, |block| block))
    }

    fn set(&mut self, row: i8, col: i8, val: TetType) {
        self.tets[row as usize][col as usize] = Some(val);
    }

    fn clear(&mut self, row: i8) {
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

pub struct Game {
    assets: Assets,
    current_tet: Tet,
    has_tet: bool,
    tets: Tets,
    fall_timer: Duration,
    fall_mode: Mode,
}

impl Game {
    const NORMAL_INTERVAL: Duration = Duration::from_secs(1);
    const SOFT_DROP_INTERVAL: Duration = Duration::from_millis(100);

    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let game = Game {
            assets: Assets::load(ctx)?,
            current_tet: Tet::new(TetType::S, Point2::new(3, 0)),
            has_tet: true,
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
        self.has_tet = false;
        // TODO: set a timer to spawn new random tet
    }

    fn spawn_tet(&mut self, tet_type: TetType) {
        self.current_tet = Tet::new(tet_type, Point2::new(3, 0));
        self.has_tet = true;
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
            // TODO: update logic in here
        }

        if self.has_tet {
            self.fall_timer = match decrement(
                self.fall_timer,
                ggez::timer::delta(ctx),
                self.fall_interval()
            ) {
                TimerState::Ticking(time) => time,
                TimerState::Done(time) => {
                    if !self.current_tet.fall(&self.tets) {
                        self.new_tet();
                    }
                    time
                },
            };
        }

        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods, repeat: bool) {
        match keycode {
            KeyCode::Escape => event::quit(ctx),
            KeyCode::Left if self.has_tet => self.current_tet.move_left(&self.tets),
            KeyCode::Right if self.has_tet => self.current_tet.move_right(&self.tets),
            KeyCode::Up if self.has_tet => self.current_tet.rotate_c(&self.tets),
            KeyCode::Space if self.has_tet => self.hard_drop(),
            KeyCode::Down => {
                if !repeat {
                    self.fall_mode = Mode::SoftDrop;
                    self.fall_timer = Duration::from_secs(0);
                }
            },
            KeyCode::I if !self.has_tet => self.spawn_tet(TetType::I),
            KeyCode::J if !self.has_tet => self.spawn_tet(TetType::J),
            KeyCode::L if !self.has_tet => self.spawn_tet(TetType::L),
            KeyCode::O if !self.has_tet => self.spawn_tet(TetType::O),
            KeyCode::S if !self.has_tet => self.spawn_tet(TetType::S),
            KeyCode::T if !self.has_tet => self.spawn_tet(TetType::T),
            KeyCode::Z if !self.has_tet => self.spawn_tet(TetType::Z),
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
                            .dest(Point2f32::new(
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
                    .dest(Point2f32::new(
                        TILE_SIZE * (self.current_tet.pos.x + block.x) as f32,
                        TILE_SIZE * (self.current_tet.pos.y + block.y) as f32,
                    ))
            )?;
        }

        // TODO: FPS

        graphics::present(ctx)?;
        Ok(())
    }
}
