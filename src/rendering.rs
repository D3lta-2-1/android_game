use std::sync::Arc;
use std::sync::mpsc::Receiver;
use vello::{wgpu, AaConfig, Renderer, RendererOptions, Scene};
use vello::kurbo::{Affine, Circle, Point};
use vello::peniko::Color;
use vello::util::{RenderContext, RenderSurface};
use winit::dpi::PhysicalSize;
use winit::window::Window;
use crate::event_handler::GraphicHandler;
use crate::logic_hook::DrawContent;

pub struct VelloGraphic<'s> {
    context: RenderContext,
    renderers: Vec<Option<Renderer>>,
    surface: Option<RenderSurface<'s>>,
    scene: Scene,
    incoming_data: Receiver<DrawContent>,
    last_data: DrawContent,
}

impl<'s> VelloGraphic<'s> {
    pub fn new(incoming_data: Receiver<DrawContent>) -> Self {
        Self {
            context: RenderContext::new(),
            renderers: vec![],
            surface: None,
            scene: Scene::new(),
            incoming_data,
            last_data: DrawContent{ pos: Point::new(0.0, 0.0), tick: 0 }
        }
    }

    fn build_scene(&mut self) {
        self.scene.reset();

        for data in self.incoming_data.try_iter() {
            self.last_data = data;
        }

        let circle = Circle::new(self.last_data.pos, 60.0);
        let circle_fill_color = Color::rgba(0.9529, 0.5451, 0.6588, 1.);
        self.scene.fill(
            vello::peniko::Fill::NonZero,
            Affine::IDENTITY,
            circle_fill_color,
            None,
            &circle,
        );
    }

    fn create_renderer(ctx: &RenderContext, surface: &RenderSurface) -> Renderer {
        Renderer::new(
            &ctx.devices[surface.dev_id].device,
            RendererOptions {
                surface_format: Some(surface.format),
                use_cpu: false,
                antialiasing_support: vello::AaSupport::area_only(),
                num_init_threads: None,
            },
        ).expect("Couldn't create renderer")
    }
}

impl<'s> GraphicHandler for VelloGraphic<'s> {
    fn resumed(&mut self, window: Arc<Window>) {
        let size = window.inner_size();
        let surface_future = self.context.create_surface(
            window,
            size.width,
            size.height,
            wgpu::PresentMode::AutoVsync,
        );
        let surface = pollster::block_on(surface_future).expect("Error creating surface");
        // Create a vello Renderer for the surface (using its device id)
        self.renderers
            .resize_with(self.context.devices.len(), || None);
        self.renderers[surface.dev_id]
            .get_or_insert_with(|| Self::create_renderer(&self.context, &surface));

        self.surface = Some(surface);
    }

    fn suspended(&mut self) {
        self.surface.take();
    }

    fn resized(&mut self, size: PhysicalSize<u32>) {
        self.context.resize_surface(self.surface.as_mut().unwrap(), size.width, size.height);
    }

    fn draw(&mut self) {
        self.build_scene();
        let surface = self.surface.as_ref().unwrap();

        // Get the window size
        let width = surface.config.width;
        let height = surface.config.height;

        // Get a handle to the device
        let device_handle = &self.context.devices[surface.dev_id];

        // Get the surface's texture
        let surface_texture = surface
            .surface
            .get_current_texture()
            .expect("failed to get surface texture");

        // Render to the surface's texture
        self.renderers[surface.dev_id]
            .as_mut()
            .unwrap()
            .render_to_surface(
                &device_handle.device,
                &device_handle.queue,
                &self.scene,
                &surface_texture,
                &vello::RenderParams {
                    base_color: Color::GRAY, // Background color
                    width,
                    height,
                    antialiasing_method: AaConfig::Area,
                },
            )
            .expect("failed to render to surface");

        // Queue the texture to be presented on the surface
        surface_texture.present();
    }
}