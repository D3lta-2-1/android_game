use std::time::Duration;
use winit::application::ApplicationHandler;
use crate::event_handling::EventHandler;
use crate::rendering::Graphic;
use crate::game_core::GameCore;
use crate::logic_hook::LogicHook;

mod event_handling;
mod rendering;
mod logic_hook;
mod game_core;

pub fn new_app() -> impl ApplicationHandler {
    // Setup a bunch of state:
    let tick_duration = Duration::from_millis(16);

    let logic = LogicHook::new(GameCore::new(), tick_duration);
    let graphics = Graphic::new();
    EventHandler::new(graphics, logic)
}

/*fn main() {
    let env = Env::default()
        .filter_or("egui_renderer", "warn")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let mut app = new_app();
    EventLoop::new().unwrap()
        .run_app(&mut app)
        .expect("Couldn't run event loop");
}*/
