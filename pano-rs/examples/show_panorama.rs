#[macro_use]
extern crate glium;

use std::io::Cursor;

fn main() {
    #[allow(unused_imports)]
    use glium::{glutin, Surface};

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(840.0, 840.0));
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let image = image::load(
        Cursor::new(include_bytes!("../panorama_image_transfer.png")),
        image::ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();
    let image_dimensions = image.dimensions();
    let image =
        glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }

    implement_vertex!(Vertex, position);

    let vertex1 = Vertex {
        position: [-1.0, -1.0],
    };
    let vertex2 = Vertex {
        position: [-1.0, 1.0],
    };
    let vertex3 = Vertex {
        position: [1.0, 1.0],
    };
    let vertex4 = Vertex {
        position: [1.0, -1.0],
    };
    let shape = vec![vertex1, vertex2, vertex3, vertex4];

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let index_buffer = glium::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &[0u16, 1, 2, 2, 3, 0],
    )
    .unwrap();

    let vertex_shader_src = r#"
        #version 140

        in vec2 position;
        out vec2 v_tex_coords;

        void main() {
            gl_Position = vec4(position, 0.0, 1.0);
            v_tex_coords = position;
        }
    "#;

    let fragment_shader_src = r#"
        #version 140
        #define PI 3.1415926535897932384626

        in vec2 v_tex_coords;
        out vec4 color;

        uniform sampler2D tex;
        uniform float rotation_x;
        uniform float rotation_y;

        void main() {
            vec3 pt = vec3(v_tex_coords.x, v_tex_coords.y, 1.0);
            pt = normalize(pt);
            
            float rotation_x_ = rotation_x / 180.0 * PI;
            float rotation_y_ = rotation_y / 180.0 * PI;
            mat3 rotation_x_mat = mat3(
                vec3(1, 0.0, 0.0),
                vec3(0.0, cos(rotation_x_), -sin(rotation_x_)),
                vec3(0.0, sin(rotation_x_), cos(rotation_x_))
            );
            mat3 rotation_y_mat = mat3(
                vec3(cos(rotation_y_), 0.0, sin(rotation_y_)),
                vec3(0.0, 1.0, 0.0),
                vec3(-sin(rotation_y_), 0.0, cos(rotation_y_))
            );
            mat3 rotation = rotation_y_mat * rotation_x_mat;
            pt = rotation * pt;

            float elevation = asin(pt.y);
            float azimuth = sign(pt.x) * acos(pt.z / cos(elevation));

            vec2 tex_coords = vec2(azimuth / PI, elevation / PI * 2.0);
            tex_coords = (tex_coords + 1.0) / 2.0;

            color = texture(tex, tex_coords);
        }
    "#;

    let program =
        glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None)
            .unwrap();

    let mut rotation_x: f32 = 0.0;
    let mut rotation_y: f32 = 0.0;
    let mut prev_rotation_x: f32 = 0.0;
    let mut prev_rotation_y: f32 = 0.0;
    let mut mouse_is_down: bool = false;
    let mut mouse_move_x: f32 = 0.0;
    let mut mouse_move_y: f32 = 0.0;

    event_loop.run(move |event, _, control_flow| {
        let next_frame_time =
            std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                glutin::event::WindowEvent::MouseInput { state, button, .. } => {
                    if button == glutin::event::MouseButton::Left {
                        if state == glutin::event::ElementState::Pressed {
                            mouse_is_down = true;
                        } else if state == glutin::event::ElementState::Released {
                            mouse_is_down = false;
                            rotation_x = prev_rotation_x - mouse_move_y;
                            rotation_y = prev_rotation_y - mouse_move_x;

                            rotation_x = (rotation_x + 180.0) % 360.0 - 180.0;
                            if rotation_x > 90.0 {
                                rotation_x = 180.0 - rotation_x;
                                rotation_y = rotation_y + 180.0;
                            }
                            if rotation_x < -90.0 {
                                rotation_x = -180.0 - rotation_x;
                                rotation_y = rotation_y + 180.0;
                            }

                            prev_rotation_x = rotation_x;
                            prev_rotation_y = rotation_y;

                            mouse_move_x = 0.0;
                            mouse_move_y = 0.0;
                        }
                    }
                }
                _ => return,
            },
            glutin::event::Event::DeviceEvent { event, .. } => match event {
                glutin::event::DeviceEvent::MouseMotion { delta } => {
                    if mouse_is_down {
                        mouse_move_x += 0.5 * delta.0 as f32;
                        mouse_move_y += 0.5 * delta.1 as f32;
                        rotation_x = prev_rotation_x - mouse_move_y;
                        rotation_y = prev_rotation_y - mouse_move_x;
                    }
                }
                _ => return,
            },
            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        let uniforms = uniform! {
            tex: &texture,
            rotation_x: rotation_x,
            rotation_y: rotation_y,
        };

        target
            .draw(
                &vertex_buffer,
                &index_buffer,
                &program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
        target.finish().unwrap();
    });
}
