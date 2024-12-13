use std::error::Error;

use gl_app::{App, BufferData};
use glutin::config::ConfigTemplateBuilder;
use winit::{event_loop::EventLoop, window::Window};

#[rustfmt::skip]
static VERTEX_DATA: [f32; 15] = [
    -0.5, -0.5,  1.0,  0.0,  0.0,
     0.0,  0.5,  0.0,  1.0,  0.0,
     0.5, -0.5,  0.0,  0.0,  1.0,
];
fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;

    let template = ConfigTemplateBuilder::new().with_alpha_size(8);

    let window_attributes = Window::default_attributes()
        .with_transparent(true)
        .with_title("Triangle");

    let mut app = App::new(template, window_attributes)
        .with_shaders(
            concat!(include_str!("triangle.vs"), "\0"),
            concat!(include_str!("triangle.fs"), "\0"),
        )
        .with_buffer_data(
            BufferData::new()
                .with_vertices(VERTEX_DATA.to_vec())
                .with_attrib("position", 2, 5, 0)
                .with_attrib("color", 3, 5, 2),
        );
    event_loop.run_app(&mut app)?;

    app.exit_state
}
