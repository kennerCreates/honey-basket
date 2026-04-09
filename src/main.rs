use iced::widget::shader;
use iced::widget::shader::wgpu::{self, TextureUsages};

use rand::Rng;

#[derive(Default)]
struct App {}

#[derive(Debug)]
enum Message {}

#[derive(Debug)]
struct ColorShader {}

#[derive(Debug)]
struct ColorPrimitive {}

#[derive(Debug)]
struct ShaderPipeline {
    pipeline: wgpu::ComputePipeline,
    texture_a: wgpu::Texture,
    texture_b: wgpu::Texture,
    texture_view_a: wgpu::TextureView,
    texture_view_b: wgpu::TextureView,
    bind_group_a: wgpu::BindGroup,
    bind_group_b: wgpu::BindGroup,
    display_bind_group_a: wgpu::BindGroup,
    display_bind_group_b: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    ping_pong: bool,
}

impl App {
    fn update(&mut self, _message: Message) {}
    fn view(&self) -> iced::Element<'_, Message> {
        shader(ColorShader {})
            .width(iced::Fill)
            .height(iced::Fill)
            .into()
    }
}

impl iced::widget::shader::Primitive for ColorPrimitive {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        storage: &mut shader::Storage,
        bounds: &iced::Rectangle,
        _viewport: &shader::Viewport,
    ) {
        if !storage.has::<ShaderPipeline>() {
            let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });
            let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: None,
                module: &shader_module,
                entry_point: "main",
            });

            let texture_a = device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: bounds.width as u32,
                    height: bounds.height as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let texture_b = device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: bounds.width as u32,
                    height: bounds.height as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let mut data: Vec<u8> = Vec::new();
            for _ in 0..((bounds.width * bounds.height) as i32) {
                let alive = rand::thread_rng().gen_bool(0.2) == true;
                data.extend_from_slice(if alive {
                    &[255, 255, 0, 255]
                } else {
                    &[0, 0, 0, 255]
                });
            }
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture_b,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * bounds.width as u32),
                    rows_per_image: Some(bounds.height as u32),
                },
                wgpu::Extent3d {
                    width: bounds.width as u32,
                    height: bounds.height as u32,
                    depth_or_array_layers: 1,
                },
            );

            let texture_view_a = texture_a.create_view(&wgpu::TextureViewDescriptor::default());
            let texture_view_b = texture_b.create_view(&wgpu::TextureViewDescriptor::default());

            let bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view_a),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&texture_view_b),
                    },
                ],
            });
            let bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view_b),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&texture_view_a),
                    },
                ],
            });

            let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
            let display_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("display.wgsl").into()),
            });
            let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: None,
                vertex: wgpu::VertexState {
                    module: &display_module,
                    entry_point: "vs",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &display_module,
                    entry_point: "fs",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview: None,
            });

            let display_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &render_pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view_a),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });
            let display_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &render_pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view_b),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });
            let ping_pong = true;

            storage.store(ShaderPipeline {
                pipeline,
                texture_a,
                texture_b,
                texture_view_a,
                texture_view_b,
                bind_group_a,
                bind_group_b,
                display_bind_group_a,
                display_bind_group_b,
                render_pipeline,
                sampler,
                ping_pong,
            });
        }
        let pipeline_ref = storage.get_mut::<ShaderPipeline>().unwrap();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            compute_pass.set_pipeline(&pipeline_ref.pipeline);
            if pipeline_ref.ping_pong {
                compute_pass.set_bind_group(0, &pipeline_ref.bind_group_b, &[]);
                pipeline_ref.ping_pong = false;
            } else {
                compute_pass.set_bind_group(0, &pipeline_ref.bind_group_a, &[]);
                pipeline_ref.ping_pong = true;
            }
            compute_pass.dispatch_workgroups(
                (bounds.width / 8.0) as u32,
                (bounds.height / 8.0) as u32,
                1,
            );
        }
        queue.submit([encoder.finish()]);
    }
    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        _clip_bounds: &iced::Rectangle<u32>,
    ) {
        let pipeline_ref = storage.get::<ShaderPipeline>().unwrap();
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        render_pass.set_pipeline(&pipeline_ref.render_pipeline);
        if pipeline_ref.ping_pong {
            render_pass.set_bind_group(0, &pipeline_ref.display_bind_group_b, &[]);
        } else {
            render_pass.set_bind_group(0, &pipeline_ref.display_bind_group_a, &[]);
        }
        render_pass.draw(0..3, 0..1);
    }
}

impl shader::Program<Message> for ColorShader {
    type State = ();
    type Primitive = ColorPrimitive;

    fn draw(
        &self,
        _state: &(),
        _cursor: iced::mouse::Cursor,
        _bounds: iced::Rectangle,
    ) -> ColorPrimitive {
        ColorPrimitive {}
    }
}

fn main() -> iced::Result {
    iced::application("honey-basket", App::update, App::view).run()
}
