use std::time::Duration;
use winit::application::ApplicationHandler;
use winit::event_loop::EventLoop;
use crate::event_handling::EventHandler;
use crate::rendering::VelloGraphic;
use crate::game_loop::GameLoop;
use crate::logic_hook::LogicHook;

mod event_handling;
mod rendering;
mod logic_hook;
mod game_loop;

pub fn new_app() -> impl ApplicationHandler {
    // Setup a bunch of state:
    let tick_duration = Duration::from_millis(16);

    let (logic, receiver) = LogicHook::new(GameLoop::new(), tick_duration);
    let graphics = VelloGraphic::new(receiver, tick_duration);
    EventHandler::new(graphics, logic)
}

#[cfg(target_os = "android")]
#[export_name = "android_main"]
pub fn main(android_app: winit::platform::android::activity::AndroidApp) {

    extern crate android_logger;
    use android_logger::FilterBuilder;
    use log::LevelFilter::Off;

    use log::LevelFilter;
    use android_logger::Config;
    use winit::event_loop::EventLoop;

    let filter = FilterBuilder::new().filter(Some("wgpu_core"), Off).build();

    android_logger::init_once(
        Config::default().with_filter(filter).with_max_level(LevelFilter::Trace),
    );

    use winit::platform::android::EventLoopBuilderExtAndroid;

    let mut app = new_app();

    // Create and run a winit event loop
    EventLoop::with_user_event().with_android_app(android_app).build().unwrap()
        .run_app(&mut app)
        .expect("Couldn't run event loop");
}


