use std::time::Duration;
use crate::event_handler::EventHandler;
use crate::rendering::VelloGraphic;
use crate::game_loop::GameLoop;
use crate::logic_hook::LogicHook;

mod event_handler;
mod rendering;
mod logic_hook;
mod game_loop;

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

    // Setup a bunch of state:
    let tick_length = Duration::from_millis(100);
    let (logic, receiver) = LogicHook::new(GameLoop{}, tick_length);
    let graphics = VelloGraphic::new(receiver);
    let mut app = EventHandler::new(graphics, logic);

    // Create and run a winit event loop
    EventLoop::with_user_event().with_android_app(android_app).handle_volume_keys().build().unwrap()
        .run_app(&mut app)
        .expect("Couldn't run event loop");
}