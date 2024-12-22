pub mod drawing;
mod contexts;

use std::collections::HashMap;
use std::iter::once;
use std::sync::Arc;
use egui_wgpu::Renderer;
use log::trace;
use winit::dpi::PhysicalSize;
use winit::window::Window;
use crate::event_handling::{EguiDrawingResources, GraphicHandler};
use crate::rendering::contexts::{DeviceID, RenderContext, RenderSurface};

pub struct Graphic<'s> {
    ctx: RenderContext,
    renderers: HashMap<DeviceID, Renderer>,
    surface: Option<RenderSurface<'s>>,
}

impl<'s> Graphic<'s> {
    pub fn new() -> Self {
        Self {
            ctx: RenderContext::new(),
            renderers: HashMap::new(),
            surface: None,
        }
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
        let entry = self.renderers.entry(surface.associated_device);
        entry.or_insert_with(|| {
            let device_handle = self.ctx.get_device_handle(&surface);
            Renderer::new(&device_handle.device, surface.config.format, None, 1, true)
        });
        trace!("Surface created");
        self.surface = Some(surface);
    }

    fn suspended(&mut self) {
        trace!("Surface destroyed");
        self.surface.take();
    }

    fn resized(&mut self, size: PhysicalSize<u32>) {
        if let Some(surface) = &mut self.surface {
            surface.resize(&self.ctx, size.into());
        }
    }

    fn draw(&mut self, resources: EguiDrawingResources) {
        let surface = self.surface.as_ref().unwrap();
        if !surface.is_valid() {
            return;
        }

        // Get a handle to the device
        let device_handle = self.ctx.get_device_handle(surface);

        trace!("Drawing");
        // Get the surface's texture
        let surface_texture = surface
            .surface
            .get_current_texture()
            .expect("failed to get surface texture");
        let view = surface_texture.texture.create_view(&Default::default());

        let egui_renderer = self.renderers.get_mut(&surface.associated_device).unwrap();
        let EguiDrawingResources {
            textures_delta,
            primitives,
            pixels_per_point,
        } = resources;

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [
                surface.config.width,
                surface.config.height,
            ],
            pixels_per_point,
        };

        // Update the textures
        for (texture_id, image_delta) in textures_delta.set {
            egui_renderer.update_texture(&device_handle.device, &device_handle.queue, texture_id, &image_delta);
        }

        let mut encoder = device_handle.device.create_command_encoder(&Default::default());
        let commands = egui_renderer.update_buffers(&device_handle.device, &device_handle.queue, &mut encoder, &primitives, &screen_descriptor);
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
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

            egui_renderer.render(
                &mut render_pass.forget_lifetime(),
                &primitives,
                &screen_descriptor,
            );
        }

        for texture_id in textures_delta.free {
            egui_renderer.free_texture(&texture_id);
        }

        device_handle.queue.submit(commands.into_iter().chain(once(encoder.finish())));
        // Queue the texture to be presented on the surface
        surface_texture.present();
    }
}

