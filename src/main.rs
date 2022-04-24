use ggez::conf::{WindowMode, WindowSetup};
use ggez::{event, ContextBuilder, GameResult};
use raycaster::MainState;

const TITLE: &str = "RayCaster";

fn main() -> GameResult {
    let window_mode = WindowMode::default().dimensions(1920.0, 1080.0);
    let window_setup = WindowSetup::default().title(TITLE).vsync(false);
    let (mut ctx, events_loop) = ContextBuilder::new(TITLE, "migue")
        .window_mode(window_mode)
        .window_setup(window_setup)
        .add_resource_path("assets")
        .build()?;
    let main_state = MainState::new(&mut ctx)?;
    event::run(ctx, events_loop, main_state)
}
