mod contexts;
//mod line_renderer;

use crate::event_handling::GraphicHandler;
use crate::rendering::contexts::{RenderContext, RenderSurface};
use egui::epaint;
use std::iter::once;
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::window::Window;

struct Renderer {
    egui: egui_wgpu::Renderer,
    //line_renderer: line_renderer::LineRender,
}
pub struct Graphic<'s> {
    ctx: RenderContext,
    renderer: Option<Renderer>,
    surface: Option<RenderSurface<'s>>,
}

impl<'s> Graphic<'s> {
    pub fn new() -> Self {
        Self {
            ctx: RenderContext::new(),
            renderer: None,
            surface: None,
        }
    }
}

impl<'s> GraphicHandler for Graphic<'s> {
    fn resumed(&mut self, window: Arc<Window>) {
        let size = window.inner_size();
        let surface_future =
            self.ctx
                .create_surface(window, size.into(), wgpu::PresentMode::AutoVsync);
        let surface = pollster::block_on(surface_future).expect("Error creating surface");

        self.renderer = self.renderer.take().or(Some(Renderer {
            egui: egui_wgpu::Renderer::new(
                &self.ctx.device().device,
                surface.config.format,
                None,
                1,
                true,
            ),
            //line_renderer: line_renderer::LineRender::build(self.ctx.device(), surface.config.format),
        }));
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

    fn draw(
        &mut self,
        textures_delta: epaint::textures::TexturesDelta,
        primitives: Vec<epaint::ClippedPrimitive>,
        pixels_per_point: f32,
    ) {
        let surface = self.surface.as_ref().unwrap();
        if !surface.is_valid() {
            return;
        }

        // Get a handle to the device
        let device_handle = self.ctx.device();

        // Get the surface's texture
        let surface_texture = surface
            .surface
            .get_current_texture()
            .expect("failed to get surface texture");
        let view = surface_texture.texture.create_view(&Default::default());

        let renderer = self.renderer.as_mut().unwrap();

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [surface.config.width, surface.config.height],
            pixels_per_point,
        };

        // Update the textures
        for (texture_id, image_delta) in textures_delta.set {
            renderer.egui.update_texture(
                &device_handle.device,
                &device_handle.queue,
                texture_id,
                &image_delta,
            );
        }

        let mut encoder = device_handle
            .device
            .create_command_encoder(&Default::default());
        let commands = renderer.egui.update_buffers(
            &device_handle.device,
            &device_handle.queue,
            &mut encoder,
            &primitives,
            &screen_descriptor,
        );
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: Default::default(),
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let mut render_pass = render_pass.forget_lifetime();
            renderer
                .egui
                .render(&mut render_pass, &primitives, &screen_descriptor);
            // renderer.line_renderer.draw(&mut render_pass); TODO: For now, I don't need rotations nor complex shapes, so I'll stick with egui's primitives
        }

        for texture_id in textures_delta.free {
            renderer.egui.free_texture(&texture_id);
        }

        device_handle
            .queue
            .submit(commands.into_iter().chain(once(encoder.finish())));
        // Queue the texture to be presented on the surface
        surface_texture.present();
    }
}
