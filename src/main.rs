use iced::wgpu::{self, TextureUsages};
use iced::widget::shader;
use rand::Rng;
use std::time::{Duration, Instant};

#[derive(Default)]
struct App {
    latest_tick: Option<Instant>,
}

#[derive(Debug)]
enum Message {
    Tick(Instant),
}

#[derive(Debug)]
struct ColorShader {
    latest_tick: Option<Instant>,
}

#[derive(Debug)]
struct ColorPrimitive {
    latest_tick: Option<Instant>,
}

#[derive(Debug)]
#[allow(dead_code)]
enum SeedPattern {
    Random,
    Glider,
    Block,
    Blinker,
    Pulsar,
}

#[derive(Debug)]
struct ShaderPipelineInner {
    compute_pipeline: wgpu::ComputePipeline,
    _texture_a: wgpu::Texture,
    _texture_b: wgpu::Texture,
    _texture_view_a: wgpu::TextureView,
    _texture_view_b: wgpu::TextureView,
    bind_group_a: wgpu::BindGroup,
    bind_group_b: wgpu::BindGroup,
    display_bind_group_a: wgpu::BindGroup,
    display_bind_group_b: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    _sampler: wgpu::Sampler,
    ping_pong: bool,
    last_processed_tick: Option<Instant>,
}

#[derive(Debug)]
struct ShaderPipeline {
    format: wgpu::TextureFormat,
    inner: Option<ShaderPipelineInner>,
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::Tick(instant) => {
                self.latest_tick = Some(instant);
            }
        }
    }
    fn view(&self) -> iced::Element<'_, Message> {
        shader(ColorShader {
            latest_tick: self.latest_tick,
        })
        .width(iced::Fill)
        .height(iced::Fill)
        .into()
    }
}

fn subscription(_app: &App) -> iced::Subscription<Message> {
    iced::time::every(Duration::from_millis(100)).map(Message::Tick)
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title("game-of-life")
        .subscription(subscription)
        .run()
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
        ColorPrimitive {
            latest_tick: self.latest_tick,
        }
    }
}

impl shader::Pipeline for ShaderPipeline {
    fn new(_device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        Self {
            format,
            inner: None,
        }
    }
}

impl shader::Primitive for ColorPrimitive {
    type Pipeline = ShaderPipeline;

    fn prepare(
        &self,
        pipeline: &mut ShaderPipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &iced::Rectangle,
        _viewport: &shader::Viewport,
    ) {
        if pipeline.inner.is_none() {
            let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });
            let compute_pipeline =
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: None,
                    layout: None,
                    module: &shader_module,
                    entry_point: Some("main"),
                    cache: None,
                    compilation_options: Default::default(),
                });

            let _texture_a = device.create_texture(&wgpu::TextureDescriptor {
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
            let _texture_b = device.create_texture(&wgpu::TextureDescriptor {
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
            let data = generate_seed(
                SeedPattern::Pulsar,
                bounds.width as i32,
                bounds.height as i32,
            );
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &_texture_b,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &data,
                wgpu::TexelCopyBufferLayout {
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

            let _texture_view_a = _texture_a.create_view(&wgpu::TextureViewDescriptor::default());
            let _texture_view_b = _texture_b.create_view(&wgpu::TextureViewDescriptor::default());

            let bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &compute_pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&_texture_view_a),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&_texture_view_b),
                    },
                ],
            });
            let bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &compute_pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&_texture_view_b),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&_texture_view_a),
                    },
                ],
            });

            let _sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
            let display_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("display.wgsl").into()),
            });
            let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: None,
                vertex: wgpu::VertexState {
                    module: &display_module,
                    entry_point: Some("vs"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &display_module,
                    entry_point: Some("fs"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: pipeline.format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview: None,
                cache: None,
            });

            let display_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &render_pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&_texture_view_a),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&_sampler),
                    },
                ],
            });
            let display_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &render_pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&_texture_view_b),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&_sampler),
                    },
                ],
            });
            let ping_pong = true;
            let last_processed_tick = None;

            pipeline.inner = Some(ShaderPipelineInner {
                compute_pipeline,
                _texture_a,
                _texture_b,
                _texture_view_a,
                _texture_view_b,
                bind_group_a,
                bind_group_b,
                display_bind_group_a,
                display_bind_group_b,
                render_pipeline,
                _sampler,
                ping_pong,
                last_processed_tick,
            });
        }
        let pipeline_ref = pipeline.inner.as_mut().unwrap();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            if pipeline_ref.last_processed_tick != self.latest_tick {
                let mut compute_pass =
                    encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
                compute_pass.set_pipeline(&pipeline_ref.compute_pipeline);
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
                pipeline_ref.last_processed_tick = self.latest_tick;
            }
        }
        queue.submit([encoder.finish()]);
    }
    fn render(
        &self,
        pipeline: &ShaderPipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        _clip_bounds: &iced::Rectangle<u32>,
    ) {
        let pipeline_ref = pipeline.inner.as_ref().unwrap();
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
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

fn generate_seed(pattern: SeedPattern, width: i32, height: i32) -> Vec<u8> {
    match pattern {
        SeedPattern::Random => {
            let mut seed: Vec<u8> = Vec::new();
            for _ in 0..((width * height) as i32) {
                let alive = rand::thread_rng().gen_bool(0.2) == true;
                seed.extend_from_slice(if alive {
                    &[255, 255, 0, 255]
                } else {
                    &[0, 0, 0, 255]
                });
            }
            seed
        }
        SeedPattern::Glider => {
            let mut seed: Vec<u8> = Vec::new();
            for _ in 0..((width * height) as i32) {
                seed.extend_from_slice(&[0, 0, 0, 255]);
            }
            let center_x = width / 2;
            let center_y = height / 2;
            let glider_positions = [
                (center_x + 1, center_y),
                (center_x + 2, center_y + 1),
                (center_x, center_y + 2),
                (center_x + 1, center_y + 2),
                (center_x + 2, center_y + 2),
            ];
            for (x, y) in glider_positions {
                let index = (y * width + x) as usize * 4;
                seed[index..index + 4].copy_from_slice(&[255, 255, 0, 255]);
            }
            seed
        }
        SeedPattern::Block => {
            let mut seed: Vec<u8> = Vec::new();
            for _ in 0..((width * height) as i32) {
                seed.extend_from_slice(&[0, 0, 0, 255]);
            }
            let center_x = width / 2;
            let center_y = height / 2;
            let block_positions = [
                (center_x, center_y),
                (center_x + 1, center_y),
                (center_x, center_y + 1),
                (center_x + 1, center_y + 1),
            ];
            for (x, y) in block_positions {
                let index = (y * width + x) as usize * 4;
                seed[index..index + 4].copy_from_slice(&[255, 255, 0, 255]);
            }
            seed
        }
        SeedPattern::Blinker => {
            let mut seed: Vec<u8> = Vec::new();
            for _ in 0..((width * height) as i32) {
                seed.extend_from_slice(&[0, 0, 0, 255]);
            }
            let center_x = width / 2;
            let center_y = height / 2;
            let blinker_positions = [
                (center_x, center_y),
                (center_x + 1, center_y),
                (center_x + 2, center_y),
            ];
            for (x, y) in blinker_positions {
                let index = (y * width + x) as usize * 4;
                seed[index..index + 4].copy_from_slice(&[255, 255, 0, 255]);
            }
            seed
        }
        SeedPattern::Pulsar => {
            let mut seed: Vec<u8> = Vec::new();
            for _ in 0..((width * height) as i32) {
                seed.extend_from_slice(&[0, 0, 0, 255]);
            }
            let center_x = width / 2;
            let center_y = height / 2;
            let pulsar_positions = [
                (center_x + 2, center_y),
                (center_x + 3, center_y),
                (center_x + 4, center_y),
                (center_x + 8, center_y),
                (center_x + 9, center_y),
                (center_x + 10, center_y),
                (center_x, center_y + 2),
                (center_x + 5, center_y + 2),
                (center_x + 7, center_y + 2),
                (center_x + 12, center_y + 2),
                (center_x, center_y + 3),
                (center_x + 5, center_y + 3),
                (center_x + 7, center_y + 3),
                (center_x + 12, center_y + 3),
                (center_x, center_y + 4),
                (center_x + 5, center_y + 4),
                (center_x + 7, center_y + 4),
                (center_x + 12, center_y + 4),
                (center_x + 2, center_y + 5),
                (center_x + 3, center_y + 5),
                (center_x + 4, center_y + 5),
                (center_x + 8, center_y + 5),
                (center_x + 9, center_y + 5),
                (center_x + 10, center_y + 5),
                (center_x + 2, center_y + 7),
                (center_x + 3, center_y + 7),
                (center_x + 4, center_y + 7),
                (center_x + 8, center_y + 7),
                (center_x + 9, center_y + 7),
                (center_x + 10, center_y + 7),
                (center_x, center_y + 8),
                (center_x + 5, center_y + 8),
                (center_x + 7, center_y + 8),
                (center_x + 12, center_y + 8),
                (center_x, center_y + 9),
                (center_x + 5, center_y + 9),
                (center_x + 7, center_y + 9),
                (center_x + 12, center_y + 9),
                (center_x, center_y + 10),
                (center_x + 5, center_y + 10),
                (center_x + 7, center_y + 10),
                (center_x + 12, center_y + 10),
                (center_x + 2, center_y + 12),
                (center_x + 3, center_y + 12),
                (center_x + 4, center_y + 12),
                (center_x + 8, center_y + 12),
                (center_x + 9, center_y + 12),
                (center_x + 10, center_y + 12),
            ];
            for (x, y) in pulsar_positions {
                let index = (y * width + x) as usize * 4;
                seed[index..index + 4].copy_from_slice(&[255, 255, 0, 255]);
            }
            seed
        }
    }
}
