use std::mem::replace;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{Touch, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowId};

pub trait LogicHandler {
    fn exit(&mut self);
    fn touch_event(&mut self, touch: Touch);
}

pub trait GraphicHandler {
    /** Surface should only be created here **/
    fn resumed(&mut self, window: Arc<Window>);
    /** Surface should be deleted here **/
    fn suspended(&mut self);
    fn resized(&mut self, size: PhysicalSize<u32>);
    fn draw(&mut self);
    fn exit(&mut self) {}
}

enum Activity {
    None,
    Resumed(Arc<Window>),
    Suspended(Arc<Window>),
}

pub struct EventHandler<Graphic: GraphicHandler, Logic: LogicHandler> {
    logic_handler: Logic,
    graphic_handler: Graphic,
    activity: Activity,
}

impl<Graphic: GraphicHandler, Logic: LogicHandler> EventHandler<Graphic, Logic> {
    pub fn new(graphic_handler: Graphic, logic_handler: Logic) -> Self {
        Self {
            logic_handler,
            graphic_handler,
            activity: Activity::None,
        }
    }

    fn create_window(event_loop: &ActiveEventLoop) -> Arc<Window> {
        let attr = Window::default_attributes()
            .with_min_inner_size(LogicalSize::new(40, 40))
            .with_inner_size(LogicalSize::new(720, 480))
            .with_resizable(true)
            .with_title("game");
        Arc::new(event_loop.create_window(attr).unwrap())
    }
}

impl<Graphic: GraphicHandler, Logic: LogicHandler> ApplicationHandler for EventHandler<Graphic, Logic> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = match replace(&mut self.activity, Activity::None) {
            Activity::None => Self::create_window(event_loop),
            Activity::Resumed(_) => return,
            Activity::Suspended(window) => window,
        };

        self.graphic_handler.resumed(window.clone());
        self.activity = Activity::Resumed(window);
        event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let Activity::Resumed(window) = &self.activity else { return; };
        if window_id != window.id() { return; }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { .. } => {}, //NAVIGATE_BACK is a keyboard input
            WindowEvent::ActivationTokenDone { .. } => {},
            WindowEvent::Resized(size) => {
                self.graphic_handler.resized(size);
                window.request_redraw();
            },
            WindowEvent::Moved(_) => {},
            WindowEvent::Focused(_) => {},
            WindowEvent::AxisMotion { .. } => {},
            WindowEvent::Touch(touch) => self.logic_handler.touch_event(touch),
            WindowEvent::RedrawRequested => {
                self.graphic_handler.draw();
                window.request_redraw();
            },
            _ => {}
        }
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        let Activity::Resumed(window) = replace(&mut self.activity, Activity::None) else { return; };
        self.activity = Activity::Suspended(window);
        self.graphic_handler.suspended();
        event_loop.set_control_flow(ControlFlow::Wait);
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.logic_handler.exit();
        self.graphic_handler.exit();
    }
}