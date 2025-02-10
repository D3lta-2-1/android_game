use std::time::Duration;
use winit::application::ApplicationHandler;
use running_context::event_handling::EventHandler;
use running_context::rendering::Graphic;
use crate::game_core::GameCore;
use crate::logic_hook::LogicHook;

mod logic_hook;
mod game_core;

pub fn new_app() -> impl ApplicationHandler {
    // for tracing purposes, nothing should be created before the EventHandler itself
    EventHandler::new(|| {
        // Setup a bunch of state:
        let tick_duration = Duration::from_millis(16);
        let logic = LogicHook::new(GameCore::new(), tick_duration);
        let graphics = Graphic::new();
        (graphics, logic)
    })
}