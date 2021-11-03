#[macro_use]
extern crate glium;

use glium::{glutin, Surface};


fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let cb = glutin::ContextBuilder::new();

    let size = glutin::dpi::PhysicalSize {
        width: 3840,
        height: 1920,
    };
    let context = cb.build_headless(&event_loop, size).unwrap();
    let headless = glium::backend::glutin::headless::Headless::new(context).unwrap();
    
    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }

    implement_vertex!(Vertex, position);

    let vertex1 = Vertex { position: [-1.0, -1.0] };
    let vertex2 = Vertex { position: [-1.0,  1.0] };
    let vertex3 = Vertex { position: [1.0, 1.0] };
    let vertex4 = Vertex { position: [1.0, -1.0] };
    let shape = vec![vertex1, vertex2, vertex3, vertex4];

    let vertex_buffer = glium::VertexBuffer::new(&headless, &shape).unwrap();
    let index_buffer = glium::IndexBuffer::new(&headless, glium::index::PrimitiveType::TrianglesList,
        &[0u16, 1, 2, 2, 3, 0]).unwrap();

    let output_texture = glium::texture::Texture2d::empty(&headless, 3840, 1920).unwrap();
    
    let vertex_shader_src = r#"
        #version 140

        in vec2 position;
        out vec2 v_tex_coords;

        void main() {
            v_tex_coords = position;
            gl_Position = vec4(position, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140
        #define PI 3.1415926535897932384626

        in vec2 v_tex_coords;
        out vec4 color;
        
        uniform float scale;
        uniform vec3 position;
        uniform vec4 circle_color;

        void main() {
            float azimuth = v_tex_coords.x * PI;
            float elevation = v_tex_coords.y * PI / 2.0;
            
            vec3 pt;
            pt.x = cos(elevation) * sin(azimuth);
            pt.y = sin(elevation);
            pt.z = cos(elevation) * cos(azimuth);

            vec3 rotation_eular = -position / 180.0 * PI;
            mat3 rotation_x = mat3(
                vec3(1, 0.0, 0.0),
                vec3(0.0, cos(rotation_eular.x), -sin(rotation_eular.x)),
                vec3(0.0, sin(rotation_eular.x), cos(rotation_eular.x))
            );
            mat3 rotation_y = mat3(
                vec3(cos(rotation_eular.y), 0.0, sin(rotation_eular.y)),
                vec3(0.0, 1.0, 0.0),
                vec3(-sin(rotation_eular.y), 0.0, cos(rotation_eular.y))
            );
            mat3 rotation_z = mat3(
                vec3(cos(rotation_eular.z), -sin(rotation_eular.z), 0.0),
                vec3(sin(rotation_eular.z), cos(rotation_eular.z), 0.0),
                vec3(0.0, 0.0, 1.0)
            );
            mat3 rotation = rotation_x * rotation_y * rotation_z;
            pt = rotation * pt;

            if(pt.z >= 0.0) {
                vec2 plane_pos = vec2(pt.x / pt.z, pt.y / pt.z);
                if (sqrt(plane_pos.x * plane_pos.x + plane_pos.y * plane_pos.y) <= scale) {
                    color = circle_color;
                } else {
                    color = vec4(0.0, 0.0, 0.0, 0.0);
                }
            } else {
                color = vec4(0.0, 0.0, 0.0, 0.0);
            }
        }
    "#;
    
    let program = glium::Program::from_source(&headless, vertex_shader_src, fragment_shader_src,
        None).unwrap();
    
    let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&headless, &output_texture).unwrap();
    let target = headless.draw();

    framebuffer.clear_color(0.98, 0.98, 0.95, 1.0);
    
    let mut draw_params: glium::draw_parameters::DrawParameters<'_> = Default::default();
    draw_params.blend = glium::Blend::alpha_blending();

    let uniforms = uniform! {
        position: [60.0 as f32, 185.0 as f32, 10.0 as f32],
        scale: 0.3 as f32,
        circle_color: [0.0 as f32, 0.0 as f32, 1.0 as f32, 1.0 as f32],
    };
    framebuffer.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &draw_params).unwrap();
    
    let uniforms = uniform! {
        position: [90.0 as f32, 0.0 as f32, 3.0 as f32],
        scale: 0.1 as f32,
        circle_color: [0.0 as f32, 0.0 as f32, 1.0 as f32, 1.0 as f32],
    };
    framebuffer.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &draw_params).unwrap();
    
    let uniforms = uniform! {
        position: [-30.0 as f32, 20.0 as f32, 1.0 as f32],
        scale: 0.2 as f32,
        circle_color: [0.0 as f32, 0.0 as f32, 1.0 as f32, 1.0 as f32],
    };
    framebuffer.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &draw_params).unwrap();
    
    let uniforms = uniform! {
        position: [-20.0 as f32, 120.0 as f32, 50.0 as f32],
        scale: 0.4 as f32,
        circle_color: [0.0 as f32, 0.0 as f32, 1.0 as f32, 1.0 as f32],
    };
    framebuffer.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &draw_params).unwrap();

    target.finish().unwrap();
    
    let image: glium::texture::RawImage2d<u8> = output_texture.read();
    let image = image::ImageBuffer::from_raw(image.width, image.height, image.data.into_owned()).unwrap();
    let image = image::DynamicImage::ImageRgba8(image).flipv();
    image.save("panorama_circle.png").unwrap();
}
