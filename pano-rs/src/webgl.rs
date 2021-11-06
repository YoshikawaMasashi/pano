use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;

use js_sys::{ArrayBuffer, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

#[wasm_bindgen]
extern "C" {
    type Buffer;
}

#[wasm_bindgen(raw_module = "../../src/fs.js")]
extern "C" {
    #[wasm_bindgen(js_name = readFileSync, catch)]
    fn read_file(path: &str) -> Result<Buffer, JsValue>;

    #[wasm_bindgen(method, getter)]
    fn buffer(this: &Buffer) -> ArrayBuffer;

    #[wasm_bindgen(method, getter, js_name = byteOffset)]
    fn byte_offset(this: &Buffer) -> u32;

    #[wasm_bindgen(method, getter)]
    fn length(this: &Buffer) -> u32;
}

pub fn read_image(path: &Path) -> image::RgbaImage {
    let buffer = read_file(path.to_str().unwrap()).unwrap();
    let buffer: Vec<u8> = Uint8Array::new_with_byte_offset_and_length(
        &buffer.buffer(),
        buffer.byte_offset(),
        buffer.length(),
    )
    .to_vec();
    image::load(Cursor::new(buffer.as_slice()), image::ImageFormat::Png)
        .unwrap()
        .to_rgba8()
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let context = canvas
        .get_context("webgl2")?
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()?;
    context.clear_color(0.0, 0.0, 0.0, 1.0);

    let vert_shader = compile_shader(
        &context,
        WebGl2RenderingContext::VERTEX_SHADER,
        r##"#version 300 es
        const vec2[4] POSITIONS = vec2[](
            vec2(-1.0, -1.0),
            vec2(-1.0, 1.0),
            vec2(1.0, 1.0),
            vec2(1.0, -1.0)
        );
        const int[6] INDICES = int[](
            0, 1, 2,
            2, 3, 0
        );
        out vec2 fragment_position;
        void main(void) {
            vec2 position = POSITIONS[INDICES[gl_VertexID]];
            gl_Position = vec4(position, 0.0, 1.0);
            fragment_position = vec2(position.x, -position.y);
        }"##,
    )?;
    let frag_shader = compile_shader(
        &context,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        r##"#version 300 es
        #define PI 3.1415926535897932384626
        precision highp float;
        in vec2 fragment_position;
        out vec4 color;
        uniform sampler2D tex;
        void main(void) {
            float rotation_x = 0.0;
            float rotation_y = 0.0;
            vec3 pt = vec3(fragment_position.x, fragment_position.y, 1.0);
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
            float azimuth = sign(pt.x) * acos(pt.z / length(pt.xz)); // sign(pt.x) * acos(pt.z / cos(elevation));

            vec2 tex_coords = vec2(azimuth / PI, elevation / PI * 2.0);
            tex_coords = (tex_coords + 1.0) / 2.0;

            color = texture(tex, tex_coords);
        }
        "##,
    )?;
    let program = link_program(&context, &vert_shader, &frag_shader)?;
    let uniforms =
        get_uniform_locations(&context, &program, vec!["tex".to_string()]).unwrap();

    let image = read_image(Path::new("../pano-rs/panorama_image_transfer.png"));
    let tex_width = image.width();
    let tex_height = image.height();

    let texture = context.create_texture().unwrap();
    context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
    context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
        WebGl2RenderingContext::TEXTURE_2D,
        0,
        WebGl2RenderingContext::RGBA as i32,
        tex_width as i32,
        tex_height as i32,
        0,
        WebGl2RenderingContext::RGBA,
        WebGl2RenderingContext::UNSIGNED_BYTE,
        //Some(pixels.as_slice()),
        Some(image.as_raw().as_slice()),
    )?;
    context.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_MAG_FILTER,
        WebGl2RenderingContext::LINEAR as i32,
    );
    context.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_MIN_FILTER,
        WebGl2RenderingContext::LINEAR as i32,
    );
    context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

    context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
    context.use_program(Some(&program));
    context.active_texture(WebGl2RenderingContext::TEXTURE0);
    context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
    context.uniform1i(Some(&uniforms["tex"]), 0);
    context.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
    Ok(())
}

pub fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    context: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

pub fn get_uniform_locations(
    context: &WebGl2RenderingContext,
    program: &WebGlProgram,
    keys: Vec<String>,
) -> Result<HashMap<String, WebGlUniformLocation>, String> {
    let mut locations: HashMap<String, WebGlUniformLocation> = HashMap::new();
    for key in keys {
        locations.insert(
            key.clone(),
            context.get_uniform_location(program, &key).unwrap(),
        );
    }
    Ok(locations)
}
