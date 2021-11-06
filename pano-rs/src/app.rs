extern crate console_error_panic_hook;

use std::panic;
use std::path::Path;
use std::sync::{Arc, Mutex};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlShader, WebGlTexture};

use crate::file_io::read_image;
use crate::webgl_utils::{get_uniform_locations, link_program, read_shader};

const WORK_TEXTURE_WIDTH: usize = 3840;
const WORK_TEXTURE_HEIGHT: usize = 1920;

pub struct App {
    work_texture: Arc<Mutex<WebGlTexture>>,
    show_panorama_vert_shader: WebGlShader,
    show_panorama_frag_shader: WebGlShader,
    draw_circle_vert_shader: WebGlShader,
    draw_circle_frag_shader: WebGlShader,
    rotation_x: f32,
    rotation_y: f32,
}

impl App {
    pub fn new() -> Result<Self, JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

        let context = canvas
            .get_context("webgl2")?
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()?;
        context.clear_color(0.0, 0.0, 0.0, 1.0);

        let show_panorama_vert_shader = read_shader(
            Path::new("../pano-rs/src/show_panorama.vert"),
            &context,
            WebGl2RenderingContext::VERTEX_SHADER,
        )?;
        let show_panorama_frag_shader = read_shader(
            Path::new("../pano-rs/src/show_panorama.frag"),
            &context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
        )?;
        let draw_circle_vert_shader = read_shader(
            Path::new("../pano-rs/src/draw_circle.vert"),
            &context,
            WebGl2RenderingContext::VERTEX_SHADER,
        )?;
        let draw_circle_frag_shader = read_shader(
            Path::new("../pano-rs/src/draw_circle.frag"),
            &context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
        )?;

        let work_texture = context.create_texture().unwrap();

        context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&work_texture));
        context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            WebGl2RenderingContext::RGBA as i32,
            WORK_TEXTURE_WIDTH as i32,
            WORK_TEXTURE_HEIGHT as i32,
            0,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            None,
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

        Ok(App {
            work_texture: Arc::new(Mutex::new(work_texture)),
            show_panorama_vert_shader,
            show_panorama_frag_shader,
            draw_circle_vert_shader,
            draw_circle_frag_shader,
            rotation_x: 0.0,
            rotation_y: 0.0,
        })
    }

    pub fn read_image_to_work_texture(&self, path: &Path) -> Result<(), JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

        let context = canvas
            .get_context("webgl2")?
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()?;

        let image = read_image(path);
        assert_eq!(image.width(), WORK_TEXTURE_WIDTH as u32);
        assert_eq!(image.height(), WORK_TEXTURE_HEIGHT as u32);

        let work_texture = self.work_texture.lock().unwrap();
        context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&work_texture));
        context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            WebGl2RenderingContext::RGBA as i32,
            WORK_TEXTURE_WIDTH as i32,
            WORK_TEXTURE_HEIGHT as i32,
            0,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
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

        Ok(())
    }

    pub fn draw_circle(&self) -> Result<(), JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

        let context = canvas
            .get_context("webgl2")?
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()?;
        
        let frame_buffer = context.create_framebuffer().unwrap();
        context.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&frame_buffer));

        let work_texture = self.work_texture.lock().unwrap();
        context.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&work_texture),
            0
        );
        context.viewport(0, 0, WORK_TEXTURE_WIDTH as i32, WORK_TEXTURE_HEIGHT as i32);

        context.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);

        Ok(())
    }

    pub fn show(&self) -> Result<(), JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

        let context = canvas
            .get_context("webgl2")?
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()?;
        context.clear_color(0.0, 0.0, 0.0, 1.0);
        context.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
        let program = link_program(
            &context,
            &self.show_panorama_vert_shader,
            &self.show_panorama_frag_shader,
        )?;
        let uniforms = get_uniform_locations(
            &context,
            &program,
            vec![
                "tex".to_string(),
                "rotation_x".to_string(),
                "rotation_y".to_string(),
            ],
        )?;
        context.use_program(Some(&program));

        let work_texture = self.work_texture.lock().unwrap();
        context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        context.active_texture(WebGl2RenderingContext::TEXTURE0);
        context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&work_texture));
        context.uniform1i(Some(&uniforms["tex"]), 0);
        context.uniform1f(Some(&uniforms["rotation_x"]), self.rotation_x);
        context.uniform1f(Some(&uniforms["rotation_y"]), self.rotation_y);
        context.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
        context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        Ok(())
    }
}

#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let app = App::new()?;
    app.read_image_to_work_texture(Path::new("../pano-rs/panorama_image_transfer.png"))?;
    app.draw_circle()?;
    app.show()?;

    Ok(())
}
