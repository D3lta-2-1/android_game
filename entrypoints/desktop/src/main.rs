use main::new_app;
use winit::event_loop::EventLoop;

pub fn main() {
    let mut app = new_app();

    EventLoop::with_user_event()
        .build()
        .unwrap()
        .run_app(&mut app)
        .expect("Couldn't run event loop");
}
