use std::{error::Error, time::SystemTime};

use gl_app::{App, BufferData};
use glutin::config::ConfigTemplateBuilder;
use std::sync::LazyLock;
use winit::{event_loop::EventLoop, window::Window};

#[rustfmt::skip]
static VERTEX_DATA: [f32; 9] = [
     0.5, -0.5, 0.0,
    -0.5, -0.5, 0.0,
     0.0,  0.5, 0.0,
];

static START: LazyLock<SystemTime> = LazyLock::new(|| SystemTime::now());

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;

    let template = ConfigTemplateBuilder::new().with_alpha_size(8);

    let window_attributes = Window::default_attributes()
        .with_transparent(true)
        .with_title("Blinking Triangle");

    let mut app = App::new(template, window_attributes)
        .with_shaders(
            concat!(include_str!("blinking-triangle.vs"), "\0"),
            concat!(include_str!("blinking-triangle.fs"), "\0"),
        )
        .with_buffer_data(
            BufferData::new()
                .with_vertices(VERTEX_DATA.to_vec())
                .with_input(0, 3, 3, 0),
        )
        .with_draw_callback(Box::new(|renderer| {
            let elapsed = START.elapsed().unwrap().as_secs_f32();
            let green = ((elapsed * 4.0).sin() / 2.0 + 0.5) as f32;
            renderer.set_vec4f("outColor", 0.1, (green * 0.9) + 0.1, 0.1, 0.9);
        }));
    event_loop.run_app(&mut app)?;

    app.exit_state
}
