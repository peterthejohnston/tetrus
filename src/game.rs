use std::collections::HashMap;
use std::time::Duration;

use ggez::{Context, GameResult};
use ggez::event::{self, KeyCode, KeyMods};
use ggez::graphics::{self, Color, DrawParam, Text};
use ggez::timer;

use rand;

use crate::tet::{Tet, TetType};

type Point2f32 = ggez::nalgebra::Point2<f32>;
type Point2 = ggez::nalgebra::Point2<i8>;

const TILE_SIZE: f32 = 20.0;
pub const TILES_WIDE: usize = 10;
pub const TILES_HIGH: usize = 20;
const SIDEBAR_WIDTH: f32 = 6.0;
pub const WINDOW_WIDTH: f32 = (TILES_WIDE as f32 + SIDEBAR_WIDTH * 2.0) * TILE_SIZE;
pub const WINDOW_HEIGHT: f32 = TILES_HIGH as f32 * TILE_SIZE;

struct Assets {
    block_sprites: HashMap<TetType, graphics::Image>,
    preview_sprite: graphics::Image,
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
        let preview_sprite = graphics::Image::new(ctx, "/preview.png")?;
        Ok(Assets { block_sprites, preview_sprite })
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

    fn row_full(&self, row: i8) -> bool {
        self.tets[row as usize].iter().all(|block| block.is_some())
    }

    fn clear(&mut self, row: i8) {
        self.tets[row as usize] = [None; TILES_WIDE];
        for row in (0..row as usize).rev() {
            for col in (0..TILES_WIDE).rev() {
                self.tets[row + 1][col] = self.tets[row][col];
            }
        }
    }

    fn iter(&self) -> std::slice::Iter<[Option<TetType>; TILES_WIDE]> {
        self.tets.iter()
    }
}

#[derive(Debug)]
enum FallMode {
    Normal,
    SoftDrop,
}

enum Moving {
    Left,
    Right,
    None,
}

pub struct Game {
    assets: Assets,
    tets: Tets,
    current_tet: Tet,
    has_tet: bool,
    next_tet: TetType,
    held_tet: Option<TetType>,
    already_held: bool,
    fall_timer: Duration,
    fall_mode: FallMode,
    spawn_timer: Duration,
    move_timer: Duration,
    moving: Moving,
}

impl Game {
    const NORMAL_INTERVAL: Duration = Duration::from_secs(1);
    // TODO: maybe this should shorten depending on the level, or have a total
    // limit even with resets (piece has to stop eventually)
    const PAUSE_AT_BOTTOM: Duration = Duration::from_millis(300);
    const SOFT_DROP_INTERVAL: Duration = Duration::from_millis(100);
    // TODO: maybe there is only a wait at the end of a normal/soft drop
    const SPAWN_INTERVAL: Duration = Duration::from_millis(0);
    const MOVE_WAIT: Duration = Duration::from_millis(300);
    const MOVE_INTERVAL: Duration = Duration::from_millis(70);

    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let tet_type = rand::random::<TetType>();
        let next_tet = rand::random::<TetType>();

        let game = Self {
            assets: Assets::load(ctx)?,
            tets: Tets::default(),
            current_tet: Tet::new(tet_type, Point2::new(3, 0)),
            has_tet: true,
            next_tet,
            held_tet: None,
            already_held: false,
            fall_timer: Self::NORMAL_INTERVAL,
            fall_mode: FallMode::Normal,
            spawn_timer: Self::SPAWN_INTERVAL,
            move_timer: Self::MOVE_WAIT,
            moving: Moving::None,
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
        for row in 0..TILES_HIGH {
            if self.tets.row_full(row as i8) {
                self.tets.clear(row as i8);
            }
        }
        self.has_tet = false;
        self.spawn_timer = Self::SPAWN_INTERVAL;
    }

    fn preview_tet(&self) -> Tet {
        let mut preview_tet = self.current_tet.clone();
        while preview_tet.fall(&self.tets) {}
        preview_tet
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
            FallMode::Normal => Self::NORMAL_INTERVAL,
            FallMode::SoftDrop => Self::SOFT_DROP_INTERVAL,
        }
    }
}

enum TimerState {
    Ticking(Duration),
    Done,
}

fn decrement(lhs: Duration, rhs: Duration) -> TimerState {
    if lhs < rhs || lhs - rhs == Duration::from_millis(0) {
        TimerState::Done
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
            match decrement(self.fall_timer, ggez::timer::delta(ctx)) {
                TimerState::Ticking(time) => self.fall_timer = time,
                TimerState::Done => {
                    if !self.current_tet.fall(&self.tets) {
                        self.new_tet();
                    } else {
                        self.fall_timer = if self.current_tet.at_bottom(&self.tets) {
                            Self::PAUSE_AT_BOTTOM
                        } else {
                            self.fall_interval()
                        }
                    }
                },
            };
        } else {
            self.spawn_timer = match decrement(self.spawn_timer, ggez::timer::delta(ctx)) {
                TimerState::Ticking(time) => time,
                TimerState::Done => {
                    self.spawn_tet(self.next_tet);
                    self.next_tet = rand::random::<TetType>();
                    self.already_held = false;
                    Self::SPAWN_INTERVAL
                },
            };
        }

        match self.moving {
            Moving::None => (),
            Moving::Left | Moving::Right => {
                self.move_timer = match decrement(self.move_timer, ggez::timer::delta(ctx)) {
                    TimerState::Ticking(time) => time,
                    TimerState::Done => {
                        let actually_moved = match self.moving {
                            Moving::Left => self.current_tet.move_left(&self.tets),
                            Moving::Right => self.current_tet.move_right(&self.tets),
                            _ => false,
                        };
                        if actually_moved && self.current_tet.at_bottom(&self.tets) {
                            self.fall_timer = Self::PAUSE_AT_BOTTOM;
                        }
                        Self::MOVE_INTERVAL
                    },
                };
            },
        }

        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods, repeat: bool) {
        match keycode {
            KeyCode::Escape => event::quit(ctx),
            KeyCode::Left if self.has_tet && !repeat => {
                if let Moving::Left = self.moving {
                    return
                }
                self.moving = Moving::Left;
                self.move_timer = Self::MOVE_WAIT;
                let moved = self.current_tet.move_left(&self.tets);
                if moved && self.current_tet.at_bottom(&self.tets) {
                    self.fall_timer = Self::PAUSE_AT_BOTTOM;
                }
            },
            KeyCode::Right if self.has_tet && !repeat => {
                if let Moving::Right = self.moving {
                    return
                }
                self.moving = Moving::Right;
                self.move_timer = Self::MOVE_WAIT;
                let moved = self.current_tet.move_right(&self.tets);
                if moved && self.current_tet.at_bottom(&self.tets) {
                    self.fall_timer = Self::PAUSE_AT_BOTTOM;
                }
            },
            KeyCode::Up if self.has_tet => {
                let rotated = self.current_tet.rotate_c(&self.tets);
                if rotated && self.current_tet.at_bottom(&self.tets) {
                    self.fall_timer = Self::PAUSE_AT_BOTTOM;
                }
            },
            KeyCode::Space if self.has_tet => self.hard_drop(),
            KeyCode::Down => {
                if !repeat {
                    self.fall_mode = FallMode::SoftDrop;
                    self.fall_timer = Duration::from_secs(0);
                }
            },
            KeyCode::LShift | KeyCode::RShift if !self.already_held => {
                self.already_held = true;
                if let Some(held_tet) = self.held_tet {
                    self.held_tet = Some(self.current_tet.tet_type);
                    self.spawn_tet(held_tet);
                } else {
                    self.held_tet = Some(self.current_tet.tet_type);
                    self.spawn_tet(self.next_tet);
                    self.next_tet = rand::random::<TetType>();
                }
            }
            _ => ()
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        match keycode {
            KeyCode::Down => self.fall_mode = FallMode::Normal,
            KeyCode::Left => {
                if let Moving::Left = self.moving {
                    self.moving = Moving::None;
                }
            },
            KeyCode::Right => {
                if let Moving::Right = self.moving {
                    self.moving = Moving::None;
                }
            },
            _ => ()
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::from_rgb(80, 80, 80));

        let play_area = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            [
                SIDEBAR_WIDTH * TILE_SIZE, 0.0,
                TILES_WIDE as f32 * TILE_SIZE, TILES_HIGH as f32 * TILE_SIZE
            ].into(),
            graphics::BLACK,
        )?;
        let hold_area = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::stroke(2.0),
            [
                20.0, 40.0,
                TILE_SIZE * 4.0, TILE_SIZE * 4.0
            ].into(),
            graphics::WHITE,
        )?;
        if let Some(held_tet) = self.held_tet {
            let offset = held_tet.center_4x4();
            for block in held_tet.blocks().iter() {
                graphics::draw(
                    ctx,
                    &self.assets.block_sprites[&held_tet],
                    DrawParam::default()
                        .dest(Point2f32::new(
                            20.0 + TILE_SIZE * (block.x as f32 + offset.x),
                            40.0 + TILE_SIZE * (block.y as f32 + offset.y)
                        ))
                )?;
            }
        }
        let next_area = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::stroke(2.0),
            [
                (SIDEBAR_WIDTH + TILES_WIDE as f32) * TILE_SIZE + 20.0, 40.0,
                TILE_SIZE * 4.0, TILE_SIZE * 4.0
            ].into(),
            graphics::WHITE,
        )?;
        let offset = self.next_tet.center_4x4();
        for block in self.next_tet.blocks().iter() {
            graphics::draw(
                ctx,
                &self.assets.block_sprites[&self.next_tet],
                DrawParam::default()
                    .dest(Point2f32::new(
                        20.0 + TILE_SIZE * (SIDEBAR_WIDTH + TILES_WIDE as f32 + block.x as f32 + offset.x),
                        40.0 + TILE_SIZE * (block.y as f32 + offset.y)
                    ))
            )?;
        }

        graphics::draw(ctx, &play_area, DrawParam::default())?;
        graphics::draw(ctx, &hold_area, DrawParam::default())?;
        graphics::draw(ctx, &next_area, DrawParam::default())?;

        for (y, row) in self.tets.iter().enumerate() {
            for (x, block) in row.iter().enumerate() {
                if let Some(block) = block {
                    graphics::draw(
                        ctx,
                        &self.assets.block_sprites[&block],
                        DrawParam::default()
                            .dest(Point2f32::new(
                                SIDEBAR_WIDTH * TILE_SIZE + TILE_SIZE * x as f32,
                                TILE_SIZE * y as f32,
                            ))
                    )?;
                }
            }
        }
        if self.has_tet {
            let preview_tet = self.preview_tet();
            for block in preview_tet.blocks.iter() {
                graphics::draw(
                    ctx,
                    &self.assets.preview_sprite,
                    DrawParam::default()
                        .dest(Point2f32::new(
                            SIDEBAR_WIDTH * TILE_SIZE + TILE_SIZE * (preview_tet.pos.x + block.x) as f32,
                            TILE_SIZE * (preview_tet.pos.y + block.y) as f32,
                        ))
                )?;
            }
            for block in self.current_tet.blocks.iter() {
                graphics::draw(
                    ctx,
                    &self.assets.block_sprites[&self.current_tet.tet_type],
                    DrawParam::default()
                        .dest(Point2f32::new(
                            SIDEBAR_WIDTH * TILE_SIZE + TILE_SIZE * (self.current_tet.pos.x + block.x) as f32,
                            TILE_SIZE * (self.current_tet.pos.y + block.y) as f32,
                        ))
                )?;
            }
        }

        let fps = timer::fps(ctx);
        let fps_display = Text::new(format!("FPS: {:.0}", fps));
        graphics::draw(
            ctx,
            &fps_display,
            (Point2f32::new(10.0, 10.0), graphics::WHITE),
        )?;

        // TODO: FPS

        graphics::present(ctx)?;
        Ok(())
    }
}
