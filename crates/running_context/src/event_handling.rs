use device_extensions::DeviceExtensions;
use egui::epaint;
use egui::output::OutputEvent;
use std::ops::Deref;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter, Registry};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowId};

pub struct EguiGuiExtendContext {
    pub context: egui::Context,
    //pub toasts: egui_notify::Toasts,
}

impl Deref for EguiGuiExtendContext {
    type Target = egui::Context;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

pub trait LogicHandler {
    /// Called on each frame, should not be used to process game logic
    fn update_gui(&mut self, ctx: &mut EguiGuiExtendContext);
    /// invoked when the application is about to exit
    fn exit(&mut self);
}

pub trait GraphicHandler {
    /// Surface should only be created here, this is a requirement on android
    fn resumed(&mut self, window: Arc<Window>);
    /// Surface should be deleted here
    fn suspended(&mut self);
    fn resized(&mut self, size: PhysicalSize<u32>);
    /// The Backend is responsible for drawing the GUI
    fn draw(
        &mut self,
        textures_delta: epaint::textures::TexturesDelta,
        primitives: Vec<epaint::ClippedPrimitive>,
        pixels_per_point: f32,
    );
    /// invoked when the application is about to exit
    fn exit(&mut self) {}
}

struct DeferredInit {
    window: Arc<Window>,
    egui_state: egui_winit::State,
    device_extensions: DeviceExtensions,
}

impl DeferredInit {
    fn new(event_loop: &ActiveEventLoop, ctx: egui::Context) -> DeferredInit {
        let attr = Window::default_attributes()
            .with_min_inner_size(LogicalSize::new(40, 40))
            .with_inner_size(LogicalSize::new(720, 480))
            .with_resizable(true)
            .with_title("game");
        let window = Arc::new(event_loop.create_window(attr).unwrap());
        let egui_state = egui_winit::State::new(
            ctx.clone(),
            ctx.viewport_id(),
            &window,
            Some(egui_winit::pixels_per_point(&ctx, &window)),
            window.theme(),
            None,
        );
        let mut viewport_info = egui::ViewportInfo::default();
        egui_winit::update_viewport_info(&mut viewport_info, &ctx, &window, true);

        let device_extensions = DeviceExtensions::new(event_loop);

        DeferredInit {
            window,
            egui_state,
            device_extensions,
        }
    }
}

#[derive(Eq, PartialEq)]
enum Activity {
    Resumed,
    Suspended,
}

pub struct EventHandler<Graphic: GraphicHandler, Logic: LogicHandler> {
    logic_handler: Logic,
    graphic_handler: Graphic,
    extend_ctx: EguiGuiExtendContext,
    deferred_init: Option<DeferredInit>,
    activity: Activity,
}

impl<Graphic: GraphicHandler, Logic: LogicHandler> EventHandler<Graphic, Logic> {
    pub fn new(builder: impl FnOnce() -> (Graphic, Logic)) -> Self {
        // We ensure that the subscriber is set up before any logging is done !
        Registry::default()
            .with(fmt::Layer::default())
            .with(EnvFilter::from_default_env())
            .init();

        info!("Starting the application");

        let (graphic_handler, logic_handler) = builder();

        let egui_context = EguiGuiExtendContext {
            context: egui::Context::default(),
            //toasts: egui_notify::Toasts::default().with_margin(egui::Vec2::new(8.0, 24.0)),
        };

        Self {
            logic_handler,
            graphic_handler,
            extend_ctx: egui_context,
            deferred_init: None,
            activity: Activity::Suspended,
        }
    }
}

impl<Graphic: GraphicHandler, Logic: LogicHandler> ApplicationHandler
    for EventHandler<Graphic, Logic>
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Activity::Resumed = self.activity {
            return;
        }

        let DeferredInit { window, .. } = self
            .deferred_init
            .get_or_insert_with(|| DeferredInit::new(event_loop, self.extend_ctx.context.clone()));
        self.graphic_handler.resumed(window.clone());
        self.activity = Activity::Resumed;
        event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.activity != Activity::Resumed {
            return;
        }
        let Some(DeferredInit {
            window,
            egui_state,
            device_extensions,
        }) = &mut self.deferred_init
        else {
            return;
        };
        if window_id != window.id() {
            return;
        }

        let egui_winit::EventResponse { .. } = egui_state.on_window_event(window, &event);

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
                window.set_visible(false);
            }
            WindowEvent::Resized(size) => {
                self.graphic_handler.resized(size);
            }
            WindowEvent::RedrawRequested => {
                let mut viewport_info = egui::ViewportInfo::default();
                egui_winit::update_viewport_info(
                    &mut viewport_info,
                    &self.extend_ctx,
                    window,
                    false,
                );
                let input = egui_state.take_egui_input(&window);

                let egui::FullOutput {
                    platform_output,
                    textures_delta,
                    shapes,
                    pixels_per_point,
                    viewport_output: _,
                } = self.extend_ctx.context.clone().run(input, |_| {
                    self.logic_handler.update_gui(&mut self.extend_ctx);
                    //self.extend_ctx.toasts.show(ctx);
                });

                let primitives = self.extend_ctx.context.tessellate(shapes, pixels_per_point);
                self.graphic_handler
                    .draw(textures_delta, primitives, pixels_per_point);

                for output in &platform_output.events {
                    match output {
                        OutputEvent::Clicked(_) => {
                            device_extensions.vibrate();
                        }
                        OutputEvent::DoubleClicked(_) => {}
                        OutputEvent::TripleClicked(_) => {}
                        OutputEvent::FocusGained(_) => {}
                        OutputEvent::TextSelectionChanged(_) => {}
                        OutputEvent::ValueChanged(_) => {}
                    }
                }

                egui_state.handle_platform_output(&window, platform_output);
                window.request_redraw();
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if self.activity != Activity::Resumed {
            return;
        }
        let Some(DeferredInit { egui_state, .. }) = &mut self.deferred_init else {
            return;
        };
        match event {
            DeviceEvent::MouseMotion { delta } => {
                egui_state.on_mouse_motion(delta);
            }
            _ => (),
        }
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        if Activity::Suspended == self.activity {
            return;
        }
        self.activity = Activity::Suspended;
        self.graphic_handler.suspended();
        event_loop.set_control_flow(ControlFlow::Wait);
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.graphic_handler.exit();
        self.logic_handler.exit();
    }
}
