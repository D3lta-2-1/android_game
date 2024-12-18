use winit::event_loop::EventLoop;
use the_game::new_app;

fn main() {
    let mut app = new_app();
    EventLoop::new().unwrap()
        .run_app(&mut app)
        .expect("Couldn't run event loop");
}