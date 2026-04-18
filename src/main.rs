use iced::Element;
use iced::Length::Fill;
use iced::Rectangle;
use iced::Subscription;
use iced::application;
use iced::mouse;
use iced::time;
use iced::widget;
use rand::Rng;
use std::time::{Duration, Instant};
use wgpu::ComputePipeline;
use wgpu::TextureUsages;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Color, ColorTargetState,
    ColorWrites, CommandEncoder, CommandEncoderDescriptor, ComputePassDescriptor,
    ComputePipelineDescriptor, Device, Extent3d, FragmentState, LoadOp, Operations, Origin3d,
    Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, Sampler, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource,
    StoreOp, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDimension,
    TextureFormat, TextureView, TextureViewDescriptor, VertexState,
};
use widget::shader;

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
    WireLoop,
}

#[derive(Debug)]
#[allow(dead_code)]
enum SimulationType {
    GameOfLife,
    BriansBrain,
    Wireworld,
}

#[derive(Debug)]
struct ShaderPipelineInner {
    compute_pipeline: ComputePipeline,
    _texture_a: Texture,
    _texture_b: Texture,
    _texture_view_a: TextureView,
    _texture_view_b: TextureView,
    bind_group_a: BindGroup,
    bind_group_b: BindGroup,
    display_bind_group_a: BindGroup,
    display_bind_group_b: BindGroup,
    render_pipeline: RenderPipeline,
    _sampler: Sampler,
    ping_pong: bool,
    last_processed_tick: Option<Instant>,
}

#[derive(Debug)]
struct ShaderPipeline {
    format: TextureFormat,
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
    fn view(&self) -> Element<'_, Message> {
        shader(ColorShader {
            latest_tick: self.latest_tick,
        })
        .width(Fill)
        .height(Fill)
        .into()
    }
}

fn subscription(_app: &App) -> Subscription<Message> {
    time::every(Duration::from_millis(100)).map(Message::Tick)
}

fn main() -> iced::Result {
    application(App::default, App::update, App::view)
        .title("game-of-life")
        .subscription(subscription)
        .run()
}

impl shader::Program<Message> for ColorShader {
    type State = ();
    type Primitive = ColorPrimitive;

    fn draw(&self, _state: &(), _cursor: mouse::Cursor, _bounds: Rectangle) -> ColorPrimitive {
        ColorPrimitive {
            latest_tick: self.latest_tick,
        }
    }
}

impl shader::Pipeline for ShaderPipeline {
    fn new(_device: &Device, _queue: &Queue, format: TextureFormat) -> Self {
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
        device: &Device,
        queue: &Queue,
        bounds: &Rectangle,
        _viewport: &shader::Viewport,
    ) {
        if pipeline.inner.is_none() {
            let sim_type = SimulationType::Wireworld;
            let shader_source = match sim_type {
                SimulationType::GameOfLife => include_str!("game_of_life.wgsl"),
                SimulationType::BriansBrain => include_str!("brians_brain.wgsl"),
                SimulationType::Wireworld => include_str!("wireworld.wgsl"),
            };
            let shader_module = device.create_shader_module(ShaderModuleDescriptor {
                label: None,
                source: ShaderSource::Wgsl(shader_source.into()),
            });
            let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
                label: None,
                layout: None,
                module: &shader_module,
                entry_point: Some("main"),
                cache: None,
                compilation_options: Default::default(),
            });

            let _texture_a = device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: bounds.width as u32,
                    height: bounds.height as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let _texture_b = device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: bounds.width as u32,
                    height: bounds.height as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let data = generate_seed(
                SeedPattern::WireLoop,
                bounds.width as i32,
                bounds.height as i32,
            );
            queue.write_texture(
                TexelCopyTextureInfo {
                    texture: &_texture_b,
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                },
                &data,
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * bounds.width as u32),
                    rows_per_image: Some(bounds.height as u32),
                },
                Extent3d {
                    width: bounds.width as u32,
                    height: bounds.height as u32,
                    depth_or_array_layers: 1,
                },
            );

            let _texture_view_a = _texture_a.create_view(&TextureViewDescriptor::default());
            let _texture_view_b = _texture_b.create_view(&TextureViewDescriptor::default());

            let bind_group_a = device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &compute_pipeline.get_bind_group_layout(0),
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&_texture_view_a),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&_texture_view_b),
                    },
                ],
            });
            let bind_group_b = device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &compute_pipeline.get_bind_group_layout(0),
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&_texture_view_b),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&_texture_view_a),
                    },
                ],
            });

            let _sampler = device.create_sampler(&SamplerDescriptor::default());
            let display_module = device.create_shader_module(ShaderModuleDescriptor {
                label: None,
                source: ShaderSource::Wgsl(include_str!("display.wgsl").into()),
            });
            let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: None,
                vertex: VertexState {
                    module: &display_module,
                    entry_point: Some("vs"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(FragmentState {
                    module: &display_module,
                    entry_point: Some("fs"),
                    compilation_options: Default::default(),
                    targets: &[Some(ColorTargetState {
                        format: pipeline.format,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview: None,
                cache: None,
            });

            let display_bind_group_a = device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &render_pipeline.get_bind_group_layout(0),
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&_texture_view_a),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&_sampler),
                    },
                ],
            });
            let display_bind_group_b = device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &render_pipeline.get_bind_group_layout(0),
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&_texture_view_b),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&_sampler),
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
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());
        {
            if pipeline_ref.last_processed_tick != self.latest_tick {
                let mut compute_pass =
                    encoder.begin_compute_pass(&ComputePassDescriptor::default());
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
        encoder: &mut CommandEncoder,
        target: &TextureView,
        _clip_bounds: &Rectangle<u32>,
    ) {
        let pipeline_ref = pipeline.inner.as_ref().unwrap();
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: StoreOp::Store,
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
        SeedPattern::WireLoop => {
            let mut seed: Vec<u8> = Vec::new();
            for _ in 0..((width * height) as i32) {
                seed.extend_from_slice(&[0, 0, 0, 255]);
            }
            let center_x = width / 2;
            let center_y = height / 2;
            let wire_positions = [
                (center_x, center_y),
                (center_x + 1, center_y),
                (center_x + 2, center_y),
                (center_x + 3, center_y),
                (center_x + 4, center_y),
                (center_x + 5, center_y),
                (center_x + 6, center_y),
                (center_x + 7, center_y),
                (center_x + 8, center_y),
                (center_x + 9, center_y),
                (center_x, center_y + 1),
                (center_x + 9, center_y + 1),
                (center_x, center_y + 2),
                (center_x + 9, center_y + 2),
                (center_x, center_y + 3),
                (center_x + 9, center_y + 3),
                (center_x, center_y),
                (center_x + 1, center_y + 4),
                (center_x + 2, center_y + 4),
                (center_x + 3, center_y + 4),
                (center_x + 4, center_y + 4),
                (center_x + 7, center_y + 4),
                (center_x + 8, center_y + 4),
                (center_x + 9, center_y + 4),
            ];
            for (x, y) in wire_positions {
                let index = (y * width + x) as usize * 4;
                seed[index..index + 4].copy_from_slice(&[255, 128, 0, 255]);
            }
            let head_positions = [(center_x + 5, center_y + 4)];
            for (x, y) in head_positions {
                let index = (y * width + x) as usize * 4;
                seed[index..index + 4].copy_from_slice(&[153, 153, 0, 255]);
            }
            let tail_positions = [(center_x + 6, center_y + 4)];
            for (x, y) in tail_positions {
                let index = (y * width + x) as usize * 4;
                seed[index..index + 4].copy_from_slice(&[76, 76, 0, 255]);
            }
            seed
        }
    }
}
