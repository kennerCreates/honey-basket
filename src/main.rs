use iced::widget::shader::wgpu::RenderPassDescriptor;

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

impl App {
    fn update(&mut self, _message: Message) {}
    fn view(&self) -> iced::Element<'_, Message> {
        iced::widget::shader(ColorShader {
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
        device: &iced::widget::shader::wgpu::Device,
        queue: &iced::widget::shader::wgpu::Queue,
        format: iced::widget::shader::wgpu::TextureFormat,
        storage: &mut iced::widget::shader::Storage,
        bounds: &iced::Rectangle,
        viewport: &iced::widget::shader::Viewport,
    ) {
    }
    fn render(
        &self,
        encoder: &mut iced::widget::shader::wgpu::CommandEncoder,
        storage: &iced::widget::shader::Storage,
        target: &iced::widget::shader::wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        encoder.begin_render_pass(&RenderPassDescriptor){
            color_attachments: []
        }
    }
}

impl iced::widget::shader::Program<Message> for ColorShader {
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
