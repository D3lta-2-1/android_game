extern crate core;

use crate::game_core::GameCore;
use crate::logic_hook::LogicHook;
use running_context::event_handling::EventHandler;
use running_context::rendering::Graphic;
use std::time::Duration;
use winit::application::ApplicationHandler;

mod fluid_simulation;
mod game_core;
mod logic_hook;
mod rigid_body;

pub fn new_app() -> impl ApplicationHandler {
    // for tracing purposes, nothing should be created before the EventHandler itself
    EventHandler::new(|| {
        // Setup a bunch of state:
        let tick_duration = Duration::from_millis(8);
        let logic = LogicHook::new(GameCore::new(tick_duration), tick_duration);
        let graphics = Graphic::new();
        (graphics, logic)
    })
}
