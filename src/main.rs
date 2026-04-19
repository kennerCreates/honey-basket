use bytemuck::{Pod, Zeroable};
use iced::Element;
use iced::Event;
use iced::Length::Fill;
use iced::Rectangle;
use iced::Subscription;
use iced::application;
use iced::mouse;
use iced::mouse::Button;
use iced::mouse::Cursor;
use iced::time;
use iced::widget;
use iced::widget::Action;
use std::time::{Duration, Instant};
use wgpu::ComputePipeline;
use wgpu::TextureUsages;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer, BufferDescriptor,
    BufferUsages, Color, ColorTargetState, ColorWrites, CommandEncoder, CommandEncoderDescriptor,
    ComputePassDescriptor, ComputePipelineDescriptor, Device, Extent3d, FragmentState, LoadOp,
    Operations, Origin3d, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, Sampler, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource,
    StoreOp, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDimension,
    TextureFormat, TextureView, TextureViewDescriptor, VertexState,
};
use widget::shader;

#[derive(Default)]
struct App {
    latest_tick: Option<Instant>,
    latest_paint: Option<(Instant, Vec<(u32, u32)>)>,
}

#[derive(Debug)]
enum Message {
    Tick(Instant),
    CellsPainted { drag_id: Instant, cells: Vec<(u32, u32)> },
}

const SIM_WIDTH: u32 = 640;
const SIM_HEIGHT: u32 = 360;

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
struct Uniforms {
    width: f32,
    height: f32,
    sim_width: f32,
    sim_height: f32,
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::Tick(instant) => {
                self.latest_tick = Some(instant);
            }
            Message::CellsPainted { drag_id, cells } => {
                match &mut self.latest_paint {
                    Some((existing_drag_id, existing_cells))  if *existing_drag_id == drag_id => {
                        existing_cells.extend(cells);
                    }
                    _ => {
                        self.latest_paint = Some((drag_id, cells));
                    }
                }
            }
        }
    }
    fn view(&self) -> Element<'_, Message> {
        shader(ColorShader {
            latest_tick: self.latest_tick,
            latest_paint: self.latest_paint.clone(),
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

fn cursor_to_cell(cursor: Cursor, bounds: Rectangle) -> Option<(u32, u32)> {
    let cursor_position = cursor.position_in(bounds);
    match cursor_position {
        Some(cursor_position) => {
            let sim_aspect = SIM_WIDTH as f32 / SIM_HEIGHT as f32;
            let widget_aspect = bounds.size().width / bounds.size().height;
            let scale_x = (sim_aspect / widget_aspect).min(1.0);
            let scale_y = (widget_aspect / sim_aspect).min(1.0);
            let widget_uv_x = cursor_position.x / bounds.size().width;
            let widget_uv_y = cursor_position.y / bounds.size().height;
            let normalized_x = (widget_uv_x - 0.5) / scale_x + 0.5;
            let normalized_y = (widget_uv_y - 0.5) / scale_y + 0.5;
            if normalized_x >= 1.0
                || normalized_x < 0.0
                || normalized_y >= 1.0
                || normalized_y < 0.0
            {
                None
            } else {
                let cell_x = (normalized_x * SIM_WIDTH as f32) as u32;
                let cell_y = (normalized_y * SIM_HEIGHT as f32) as u32;
                Some((cell_x, cell_y))
            }
        }
        None => None,
    }

}
#[derive(Debug)]
struct ColorShader {
    latest_tick: Option<Instant>,
    latest_paint: Option<(Instant, Vec<(u32, u32)>)>,
}

#[derive(Debug)]
struct ColorPrimitive {
    latest_tick: Option<Instant>,
    latest_paint: Option<(Instant, Vec<(u32, u32)>)>,
}

#[derive(Default)]
struct DragState {
    is_dragging: bool,
    last_cell: Option<(u32, u32)>,
    drag_id: Option<Instant>,
}

fn plot_drag_line(a: (u32, u32), b: (u32, u32)) -> Vec<(u32, u32)> {
    let (mut x0, mut y0) = (a.0 as i32, a.1 as i32);
    let (x1, y1) = (b.0 as i32, b.1 as i32);
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;
    let mut cells: Vec<(u32, u32)> = Vec::new();
    loop {
        cells.push ((x0 as u32, y0 as u32));
        let e2 = 2 * error;
        if e2 >= dy {
            if x0 == x1 {
                break;
            }
            error += dy;
            x0 += sx;
        }
        if e2 <= dx {
            if y0 == y1 {
                break;
            }
            error += dx;
            y0 += sy;
        }
    }
    cells
}

#[test]
fn test_plot_drag_line() {
    let cells = plot_drag_line((0, 0), (5, 3));
    assert_eq!(cells, vec![(0, 0), (1, 1), (2, 1), (3, 2), (4, 2), (5, 3)]);
}
#[test]
fn test_first_cell_plot_drag_line() {
    let cells = plot_drag_line((0, 0), (5, 3));
    assert_eq!(cells.first(), Some(&(0, 0)));
}
#[test]
fn test_last_cell_plot_drag_line() {
    let cells = plot_drag_line((0, 0), (5, 3));
    assert_eq!(cells.last(), Some(&(5, 3)));
}
#[test]
fn test_cells_are_adjacent_plot_drag_line() {
    let cells = plot_drag_line((0, 0), (5, 3));
    for pair in cells.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        assert!((b.0 as i32 - a.0 as i32).abs() <= 1);
        assert!((b.1 as i32 - a.1 as i32).abs() <= 1);
    }
}
#[test]
fn test_length_plot_drag_line() {
    let cells = plot_drag_line((0, 0), (5, 3));
    assert_eq!(cells.len(), 6);
}
#[test]
fn test_single_cell_plot_drag_line() {
    let cells = plot_drag_line((3, 3), (3, 3));
    assert_eq!(cells, vec![(3, 3)]);
}
#[test]
fn test_line_reverse_plot_drag_line() {
    let forward = plot_drag_line((0, 0), (5, 3));
    let reverse = plot_drag_line((5, 3), (0, 0));
    let mut reverse_reversed = reverse.clone();
    reverse_reversed.reverse();
    assert_eq!(forward, reverse_reversed);
}


impl shader::Program<Message> for ColorShader {
    type State = DragState;
    type Primitive = ColorPrimitive;

    fn draw(&self, _state: &Self::State, _cursor: Cursor, _bounds: Rectangle) -> ColorPrimitive {
        ColorPrimitive {
            latest_tick: self.latest_tick,
            latest_paint: self.latest_paint.clone(),
        }
    }
    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<Action<Message>> {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(Button::Left)) => {
                if let Some(cell) = cursor_to_cell(cursor, bounds) {
                    let id = Instant::now();
                    state.drag_id = Some(id);
                    state.is_dragging = true;
                    state.last_cell = Some(cell);
                    return Some(Action::publish(Message::CellsPainted { drag_id: id, cells: vec![cell] }));
                }
                None
            }
            Event::Mouse(mouse::Event::ButtonReleased(Button::Left)) => {
                state.is_dragging = false;
                state.last_cell = None;
                state.drag_id = None;
                None
            }
            Event::Mouse(mouse::Event::CursorMoved{..}) => {
                if !state.is_dragging {
                    return None;
                }
                let Some(new) = cursor_to_cell(cursor, bounds) else {
                    state.last_cell = None;
                    return None;
                };
                let cells = match state.last_cell {
                    Some(last) if new != last => plot_drag_line(last, new),
                    Some(_) => return None,
                    None => vec![new],
                };
                state.last_cell = Some(new);
                Some(Action::publish(Message::CellsPainted { drag_id: state.drag_id.unwrap(), cells }))
            }
            _ => None,
        }
    }
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
    uniform_buffer: Buffer,
    display_bind_group_a: BindGroup,
    display_bind_group_b: BindGroup,
    render_pipeline: RenderPipeline,
    _sampler: Sampler,
    ping_pong: bool,
    last_processed_tick: Option<Instant>,
    last_applied_drag_id: Option<Instant>,
    last_applied_count: usize,
}

#[derive(Debug)]
struct ShaderPipeline {
    format: TextureFormat,
    inner: Option<ShaderPipelineInner>,
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
            let sim_type = SimulationType::GameOfLife;
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
                    width: SIM_WIDTH,
                    height: SIM_HEIGHT,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let _texture_b = device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: SIM_WIDTH,
                    height: SIM_HEIGHT,
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
            let data: Vec<u8> = vec![0u8, 0, 0, 255].repeat((SIM_WIDTH * SIM_HEIGHT) as usize);
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
                    bytes_per_row: Some(4 * SIM_WIDTH),
                    rows_per_image: Some(SIM_HEIGHT),
                },
                Extent3d {
                    width: SIM_WIDTH,
                    height: SIM_HEIGHT,
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
            let uniform_buffer = device.create_buffer(&BufferDescriptor {
                label: None,
                size: std::mem::size_of::<Uniforms>() as u64,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                mapped_at_creation: false,
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
                    BindGroupEntry {
                        binding: 2,
                        resource: uniform_buffer.as_entire_binding(),
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
                    BindGroupEntry {
                        binding: 2,
                        resource: uniform_buffer.as_entire_binding(),
                    },
                ],
            });
            let ping_pong = true;
            let last_processed_tick = None;
            let last_applied_drag_id = None;
            let last_applied_count = 0;

            pipeline.inner = Some(ShaderPipelineInner {
                compute_pipeline,
                _texture_a,
                _texture_b,
                _texture_view_a,
                _texture_view_b,
                bind_group_a,
                bind_group_b,
                uniform_buffer,
                display_bind_group_a,
                display_bind_group_b,
                render_pipeline,
                _sampler,
                ping_pong,
                last_processed_tick,
                last_applied_drag_id,
                last_applied_count,
            });
        }
        let pipeline_ref = pipeline.inner.as_mut().unwrap();
        let uniforms = Uniforms {
            width: bounds.width,
            height: bounds.height,
            sim_width: SIM_WIDTH as f32,
            sim_height: SIM_HEIGHT as f32,
        };
        queue.write_buffer(
            &pipeline_ref.uniform_buffer,
            0,
            bytemuck::bytes_of(&uniforms),
        );
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());
        {   if let Some((_drag_id, cells)) = &self.latest_paint {
            if pipeline_ref.last_applied_drag_id != Some(*_drag_id){
                pipeline_ref.last_applied_drag_id = Some(*_drag_id);
                pipeline_ref.last_applied_count = 0;
            }
            for (cell_x, cell_y) in &cells[pipeline_ref.last_applied_count..] {
                let texture_to_write = if pipeline_ref.ping_pong {
                    &pipeline_ref._texture_b
                } else {
                    &pipeline_ref._texture_a
                };
                queue.write_texture(
                    TexelCopyTextureInfo {
                        texture: texture_to_write,
                        mip_level: 0,
                        origin: Origin3d { x: *cell_x, y: *cell_y, z: 0 },
                        aspect: TextureAspect::All,
                    },
                    &[255, 255, 0, 255],
                    TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(4),
                        rows_per_image: Some(1),
                    },
                    Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: 1,
                    },
                );
                pipeline_ref.last_applied_count = cells.len();
            }
        }
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
                    (SIM_WIDTH as f32 / 8.0) as u32,
                    (SIM_HEIGHT as f32 / 8.0) as u32,
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
