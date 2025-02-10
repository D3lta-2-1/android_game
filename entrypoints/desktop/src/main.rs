use winit::event_loop::EventLoop;
use main::new_app;

pub fn main() {
    let mut app = new_app();

    EventLoop::with_user_event().build().unwrap()
        .run_app(&mut app)
        .expect("Couldn't run event loop");
}
