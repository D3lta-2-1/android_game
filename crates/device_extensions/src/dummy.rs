
// Dummy implementation of the DeviceExtension, which does nothing. used for platforms other than Android.
use winit::event_loop::ActiveEventLoop;
pub struct DeviceExtensions {}

impl DeviceExtensions {

    pub fn new(_active_event_loop: &ActiveEventLoop) -> Self {
        Self {}
    }

    pub fn vibrate(&mut self) {
    }
}