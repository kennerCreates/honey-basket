use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;
use winit::window::Window;

struct App {
    window: Option<Arc<Window>>,
    surface: Option<wgpu::Surface<'static>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    instance: Option<wgpu::Instance>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        self.instance = Some(instance);
        self.window = Some(window);
        let surface = self
            .instance
            .as_ref()
            .unwrap()
            .create_surface(self.window.clone().unwrap())
            .unwrap();
        let adapter = pollster::block_on(
            self.instance
                .as_ref()
                .unwrap()
                .request_adapter(&wgpu::RequestAdapterOptions::default()),
        )
        .unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();
        let size = self.window.as_ref().unwrap().inner_size();
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface.configure(&device, &config);
        self.surface = Some(surface);
        self.device = Some(device);
        self.queue = Some(queue);
    }
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if WindowEvent::CloseRequested == event {
            self.window = None;
            event_loop.exit();
        }
        if WindowEvent::RedrawRequested == event {
            let surface_texture = self
                .surface
                .as_ref()
                .unwrap()
                .get_current_texture()
                .unwrap();
            let mut recorder = self
                .device
                .as_ref()
                .unwrap()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            let view = surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let render_pass = recorder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 0.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            drop(render_pass);
            self.queue.as_ref().unwrap().submit([recorder.finish()]);
            surface_texture.present();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App {
        window: None,
        surface: None,
        device: None,
        queue: None,
        instance: None,
    };
    let _ = event_loop.run_app(&mut app);
}
