#[macro_use]
extern crate glium;

use std::io::Cursor;

use glium::backend::Facade;
use glium::{glutin, Surface};

fn load_png_to_texture<F: ?Sized>(binary_data: &[u8], facade: &F) -> glium::texture::SrgbTexture2d
where
    F: Facade,
{
    let image = image::load(Cursor::new(binary_data), image::ImageFormat::Png)
        .unwrap()
        .to_rgba8();
    let image_dimensions = image.dimensions();
    let image =
        glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let texture = glium::texture::SrgbTexture2d::new(facade, image).unwrap();

    texture
}

fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let cb = glutin::ContextBuilder::new();

    let size = glutin::dpi::PhysicalSize {
        width: 3840,
        height: 1920,
    };
    let context = cb.build_headless(&event_loop, size).unwrap();
    let headless = glium::backend::glutin::headless::Headless::new(context).unwrap();

    println!("image loading");
    let front_texture = load_png_to_texture(include_bytes!("./6cubes_image/front.png"), &headless);
    println!("front");
    let back_texture = load_png_to_texture(include_bytes!("./6cubes_image/back.png"), &headless);
    println!("back");
    let left_texture = load_png_to_texture(include_bytes!("./6cubes_image/left.png"), &headless);
    println!("left");
    let right_texture = load_png_to_texture(include_bytes!("./6cubes_image/right.png"), &headless);
    println!("right");
    let top_texture = load_png_to_texture(include_bytes!("./6cubes_image/top.png"), &headless);
    println!("top");
    let bottom_texture =
        load_png_to_texture(include_bytes!("./6cubes_image/bottom.png"), &headless);
    println!("bottom");
    println!("image load done");

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

    let vertex_buffer = glium::VertexBuffer::new(&headless, &shape).unwrap();
    let index_buffer = glium::IndexBuffer::new(
        &headless,
        glium::index::PrimitiveType::TrianglesList,
        &[0u16, 1, 2, 2, 3, 0],
    )
    .unwrap();

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

        uniform sampler2D front;
        uniform sampler2D back;
        uniform sampler2D left;
        uniform sampler2D right;
        uniform sampler2D top;
        uniform sampler2D bottom;

        void main() {
            float azimuth = v_tex_coords.x * PI;
            float elevation = v_tex_coords.y * PI / 2.0;
            
            vec3 pt;
            pt.x = cos(elevation) * sin(azimuth);
            pt.y = sin(elevation);
            pt.z = cos(elevation) * cos(azimuth);
            
            if ((abs(pt.x) >= abs(pt.y)) && (abs(pt.x) >= abs(pt.z))) {{
                if (pt.x <= 0.0) {{
                    color = texture(left, vec2(((-pt.z/pt.x)+1.0)/2.0,((-pt.y/pt.x)+1.0)/2.0));
                }} else {{
                    color = texture(right, vec2(((-pt.z/pt.x)+1.0)/2.0,((pt.y/pt.x)+1.0)/2.0));
                }}
            }} else if (abs(pt.y) >= abs(pt.z)) {{
                if (pt.y <= 0.0) {{
                    color = texture(bottom, vec2(((-pt.x/pt.y)+1.0)/2.0,((-pt.z/pt.y)+1.0)/2.0));
                }} else {{
                    color = texture(top, vec2(((pt.x/pt.y)+1.0)/2.0,((-pt.z/pt.y)+1.0)/2.0));
                }}
            }} else {{
                if (pt.z <= 0.0) {{
                    color = texture(back, vec2(((pt.x/pt.z)+1.0)/2.0,((-pt.y/pt.z)+1.0)/2.0));
                }} else {{
                    color = texture(front, vec2(((pt.x/pt.z)+1.0)/2.0,((pt.y/pt.z)+1.0)/2.0));
                }}
            }}
        }
    "#;

    let program =
        glium::Program::from_source(&headless, vertex_shader_src, fragment_shader_src, None)
            .unwrap();

    let mut framebuffer =
        glium::framebuffer::SimpleFrameBuffer::new(&headless, &output_texture).unwrap();
    let target = headless.draw();

    let uniforms = uniform! {
        front: &front_texture,
        back: &back_texture,
        left: &left_texture,
        right: &right_texture,
        top: &top_texture,
        bottom: &bottom_texture,
    };

    framebuffer
        .draw(
            &vertex_buffer,
            &index_buffer,
            &program,
            &uniforms,
            &Default::default(),
        )
        .unwrap();

    target.finish().unwrap();

    let image: glium::texture::RawImage2d<u8> = output_texture.read();
    let image =
        image::ImageBuffer::from_raw(image.width, image.height, image.data.into_owned()).unwrap();
    let image = image::DynamicImage::ImageRgba8(image).flipv();
    image.save("equirectangular.png").unwrap();
}
