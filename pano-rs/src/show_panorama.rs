extern crate console_error_panic_hook;

use std::panic;
use std::path::Path;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlShader, WebGlTexture};

use crate::file_io::read_image;
use crate::webgl_utils::{get_uniform_locations, link_program, read_shader};

struct PanoramaShower {
    texture: WebGlTexture,
    vert_shader: WebGlShader,
    frag_shader: WebGlShader,
    rotation_x: f32,
    rotation_y: f32,
}

impl PanoramaShower {
    pub fn new() -> Result<Self, JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

        let context = canvas
            .get_context("webgl2")?
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()?;
        context.clear_color(0.0, 0.0, 0.0, 1.0);

        let vert_shader = read_shader(
            Path::new("../pano-rs/src/show_panorama.vert"),
            &context,
            WebGl2RenderingContext::VERTEX_SHADER,
        )?;
        let frag_shader = read_shader(
            Path::new("../pano-rs/src/show_panorama.frag"),
            &context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
        )?;

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

        Ok(PanoramaShower {
            texture,
            rotation_x: 0.0,
            rotation_y: 0.0,
            vert_shader,
            frag_shader,
        })
    }

    pub fn draw(&self) -> Result<(), JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

        let context = canvas
            .get_context("webgl2")?
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()?;
        context.clear_color(0.0, 0.0, 0.0, 1.0);
        let program = link_program(&context, &self.vert_shader, &self.frag_shader)?;
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

        context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        context.active_texture(WebGl2RenderingContext::TEXTURE0);
        context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
        context.uniform1i(Some(&uniforms["tex"]), 0);
        context.uniform1f(Some(&uniforms["rotation_x"]), self.rotation_x);
        context.uniform1f(Some(&uniforms["rotation_y"]), self.rotation_y);
        context.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
        context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_rotation_x(&self) -> f32 {
        self.rotation_x
    }

    #[allow(dead_code)]
    pub fn get_rotation_y(&self) -> f32 {
        self.rotation_y
    }

    #[allow(dead_code)]
    pub fn set_rotation_x(&mut self, rotation: f32) {
        self.rotation_x = rotation;
    }

    #[allow(dead_code)]
    pub fn set_rotation_y(&mut self, rotation: f32) {
        self.rotation_y = rotation;
    }

    pub fn increase_rotation_x(&mut self, rotation: f32) {
        self.rotation_x += rotation;
    }

    pub fn increase_rotation_y(&mut self, rotation: f32) {
        self.rotation_y += rotation;
    }

    pub fn modify_rotation(&mut self) {
        let mut rotation_x = self.rotation_x;
        let mut rotation_y = self.rotation_y;

        rotation_x = (rotation_x + 180.0) % 360.0 - 180.0;
        if rotation_x > 90.0 {
            rotation_x = 180.0 - rotation_x;
            rotation_y = rotation_y + 180.0;
        }
        if rotation_x < -90.0 {
            rotation_x = -180.0 - rotation_x;
            rotation_y = rotation_y + 180.0;
        }

        self.rotation_x = rotation_x;
        self.rotation_y = rotation_y;
    }
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window().unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

use std::sync::{Arc, RwLock};

#[wasm_bindgen]
pub fn show_panorama() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let shower = Arc::new(RwLock::new(PanoramaShower::new()?));
    shower.read().unwrap().draw().unwrap();

    let f = Arc::new(RwLock::new(None));
    let g = f.clone();
    let mouse_on = Arc::new(RwLock::new(false));

    {
        let shower = shower.clone();
        *g.write().unwrap() = Some(Closure::wrap(Box::new(move || {
            // shower.write().unwrap().increase_rotation_y(1.0);
            shower.read().unwrap().draw().unwrap();
            request_animation_frame(f.read().unwrap().as_ref().unwrap());
        }) as Box<dyn FnMut()>));
    }

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    {
        let mouse_on = mouse_on.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
            *mouse_on.write().unwrap() = true;
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let shower = shower.clone();
        let mouse_on = mouse_on.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            if *mouse_on.read().unwrap() {
                shower
                    .write()
                    .unwrap()
                    .increase_rotation_y(0.3 * event.movement_x() as f32);
                shower
                    .write()
                    .unwrap()
                    .increase_rotation_x(-0.3 * event.movement_y() as f32);
            }
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let shower = shower.clone();
        let mouse_on = mouse_on.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
            *mouse_on.write().unwrap() = false;
            shower.write().unwrap().modify_rotation();
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let shower = shower.clone();
        let mouse_on = mouse_on.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
            *mouse_on.write().unwrap() = false;
            shower.write().unwrap().modify_rotation();
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mouseout", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    request_animation_frame(g.read().unwrap().as_ref().unwrap());
    Ok(())
}
