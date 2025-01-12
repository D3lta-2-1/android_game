#[cfg(target_os = "android")]
use main::new_app;

#[cfg(target_os = "android")]
#[export_name = "android_main"]
pub fn main(android_app: winit::platform::android::activity::AndroidApp) {
    extern crate android_logger;
    use winit::event_loop::EventLoop;
    use winit::platform::android::EventLoopBuilderExtAndroid;

    let mut app = new_app();

    EventLoop::with_user_event().with_android_app(android_app).build().unwrap()
        .run_app(&mut app)
        .expect("Couldn't run event loop");
}


