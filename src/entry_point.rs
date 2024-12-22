use egui_winit::winit::event_loop::EventLoop;
use env_logger::Env;
use the_game::new_app;

fn main() {
    let env = Env::default()
        .filter_or("egui_renderer", "warn")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let mut app = new_app();
    EventLoop::new().unwrap()
        .run_app(&mut app)
        .expect("Couldn't run event loop");
}