use std::path;

use ggez::{ContextBuilder, GameResult};
use ggez::conf::{WindowMode, WindowSetup};
use ggez::event;

use tetris::game::{self, Game};

fn main() -> GameResult {
    let resource_dir = path::PathBuf::from("./res");

    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new("tetris", "peter")
        .window_setup(WindowSetup::default()
            .title("tetris"))
        .window_mode(WindowMode::default()
            .dimensions(game::WINDOW_WIDTH, game::WINDOW_HEIGHT))
        .add_resource_path(resource_dir)
        .build()?;

    let mut game = Game::new(ctx)?;
    event::run(ctx, event_loop, &mut game)
}
