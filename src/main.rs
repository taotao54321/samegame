use anyhow::Result;
use ggez::conf;
use ggez::event;
use ggez::ContextBuilder;

mod board;
mod game_state;

use crate::game_state::GameState;

fn main() -> Result<()> {
    let cb = ContextBuilder::new("samegame", "author")
        .window_setup(conf::WindowSetup::default().title("samegame"))
        .window_mode(conf::WindowMode::default().dimensions(640.0, 480.0))
        .add_resource_path(concat!(env!("CARGO_MANIFEST_DIR"), "/asset"));

    let (mut ctx, mut events_loop) = cb.build()?;
    let mut state = GameState::new(&mut ctx)?;
    event::run(&mut ctx, &mut events_loop, &mut state)?;

    Ok(())
}
