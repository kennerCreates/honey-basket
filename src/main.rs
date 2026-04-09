use iced::widget::shader;
use iced::widget::shader::wgpu::{self, ShaderModuleDescriptor};

#[derive(Default)]
struct App {}

#[derive(Debug)]
enum Message {}

#[derive(Debug)]
struct ColorShader {
    r: f64,
    g: f64,
    b: f64,
}

#[derive(Debug)]
struct ColorPrimitive {
    r: f64,
    g: f64,
    b: f64,
}

#[derive(Debug)]
struct ShaderPipeline {
    pipeline: wgpu::ComputePipeline,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
}

impl App {
    fn update(&mut self, _message: Message) {}
    fn view(&self) -> iced::Element<'_, Message> {
        shader(ColorShader {
            r: 1.0,
            g: 0.0,
            b: 1.0,
        })
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
        viewport: &shader::Viewport,
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
            storage.store(ShaderPipeline { pipeline })
        }
    }
    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: self.r,
                        g: self.g,
                        b: self.b,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
    }
}

impl shader::Program<Message> for ColorShader {
    type State = ();
    type Primitive = ColorPrimitive;

    fn draw(
        &self,
        state: &(),
        cursor: iced::mouse::Cursor,
        bounds: iced::Rectangle,
    ) -> ColorPrimitive {
        ColorPrimitive {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}

fn main() -> iced::Result {
    iced::application("honey-basket", App::update, App::view).run()
}
