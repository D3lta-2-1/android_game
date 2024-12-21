pub mod drawing;
mod contexts;

use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use winit::dpi::PhysicalSize;
use winit::window::Window;
use crate::event_handling::GraphicHandler;
use crate::rendering::contexts::{RenderContext, RenderSurface};
use crate::rendering::drawing::CommandBundle;

pub struct Graphic<'s> {
    ctx: RenderContext,
    surface: Option<RenderSurface<'s>>,
    incoming_data: Receiver<CommandBundle>,
    last_data: CommandBundle,
    tick_duration: Duration,
}

impl<'s> Graphic<'s> {
    pub fn new(incoming_data: Receiver<CommandBundle>, tick_duration: Duration) -> Self {
        Self {
            ctx: RenderContext::new(),
            surface: None,
            incoming_data,
            last_data: CommandBundle::new_empty(),
            tick_duration,
        }
    }

    fn build_scene(&mut self) {

        for data in self.incoming_data.try_iter() {
            self.last_data = data;
        }

        //self.last_data.append_to_scene(&mut self.scene, &self.tick_duration)
    }
}

impl<'s> GraphicHandler for Graphic<'s> {
    fn resumed(&mut self, window: Arc<Window>) {
        let size = window.inner_size();
        let surface_future = self.ctx.create_surface(
            window,
            size.into(),
            wgpu::PresentMode::AutoVsync,
        );
        let surface = pollster::block_on(surface_future).expect("Error creating surface");
        self.surface = Some(surface);
    }

    fn suspended(&mut self) {
        self.surface.take();
    }

    fn resized(&mut self, size: PhysicalSize<u32>) {
        if let Some(surface) = &mut self.surface {
            surface.resize(&self.ctx, size.into());
        }
    }

    fn draw(&mut self) {
        self.build_scene();
        let surface = self.surface.as_ref().unwrap();

        // Get a handle to the device
        let device_handle = self.ctx.get_device_handle(surface);

        // Get the surface's texture
        let surface_texture = surface
            .surface
            .get_current_texture()
            .expect("failed to get surface texture");
        let view = surface_texture.texture.create_view(&Default::default());

        let mut encoder = device_handle.device.create_command_encoder(&Default::default());
        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
            label: None,
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment{
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations{
                        load: wgpu::LoadOp::Clear(wgpu::Color{
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: Default::default(),
                    }
                })
            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        drop(_render_pass);

        device_handle.queue.submit(std::iter::once(encoder.finish()));
        // Queue the texture to be presented on the surface
        surface_texture.present();
    }
}