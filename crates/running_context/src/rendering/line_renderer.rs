use glam::Vec2;
use wgpu::RenderPass;
use wgpu::util::DeviceExt;
use crate::rendering::contexts::DeviceHandle;

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
#[derive(Debug, Clone, Copy)]
struct Vertex {
    pos: Vec2,
}

impl Vertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ]
        }
    }
}

const VERTICES: [Vertex; 6] = [
    Vertex { pos: Vec2::new(-0.5, -0.5) }, // Bottom-left
    Vertex { pos: Vec2::new(0.5, -0.5) },  // Bottom-right
    Vertex { pos: Vec2::new(0.5, 0.5) },   // Top-right
    Vertex { pos: Vec2::new(0.5, 0.5) },   // Top-right
    Vertex { pos: Vec2::new(-0.5, 0.5) },  // Top-left
    Vertex { pos: Vec2::new(-0.5, -0.5) }, // Bottom-left
];
pub struct LineRender {
    pipeline: wgpu::RenderPipeline,
    vbo: wgpu::Buffer,
}

impl LineRender {
    pub fn build(handle: &DeviceHandle, format: wgpu::TextureFormat) -> Self{

        let vbo = handle.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("line vertex buffer"),
                contents: bytemuck::cast_slice(&VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let shader = handle.device.create_shader_module(wgpu::include_wgsl!("shaders/line.wgsl"));

        let pipeline_layout = handle.device
            .create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("line renderer"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[]
                });

        let primitive = wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        };

        let pipeline = handle.device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[
                        Vertex::layout()
                    ],
                },

                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[
                        Some(wgpu::ColorTargetState {
                            format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                    ],
                }),
                primitive,
                depth_stencil: None,
                multisample: Default::default(),
                multiview: None,
                cache: None,
            }
        );

        Self {
            pipeline,
            vbo
        }
    }

    pub fn draw(&self, render_pass: &mut RenderPass) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vbo.slice(..));
        render_pass.draw(0..VERTICES.len() as u32, 0..1);
    }
}