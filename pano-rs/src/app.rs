extern crate console_error_panic_hook;

use std::panic;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlShader, WebGlTexture};

use crate::file_io::{read_image, write_image};
use crate::webgl_utils::{get_uniform_locations, link_program, read_shader};
use crate::yew_test;

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
        context.enable(WebGl2RenderingContext::BLEND);

        let show_panorama_vert_shader = read_shader(
            Path::new("./pano-rs/src/show_panorama.vert"),
            &context,
            WebGl2RenderingContext::VERTEX_SHADER,
        )?;
        let show_panorama_frag_shader = read_shader(
            Path::new("./pano-rs/src/show_panorama.frag"),
            &context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
        )?;
        let draw_circle_vert_shader = read_shader(
            Path::new("./pano-rs/src/draw_circle.vert"),
            &context,
            WebGl2RenderingContext::VERTEX_SHADER,
        )?;
        let draw_circle_frag_shader = read_shader(
            Path::new("./pano-rs/src/draw_circle.frag"),
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
            0,
        );
        context.viewport(0, 0, WORK_TEXTURE_WIDTH as i32, WORK_TEXTURE_HEIGHT as i32);

        let program = link_program(
            &context,
            &self.draw_circle_vert_shader,
            &self.draw_circle_frag_shader,
        )?;
        let uniforms = get_uniform_locations(
            &context,
            &program,
            vec![
                "scale".to_string(),
                "position".to_string(),
                "circle_color".to_string(),
            ],
        )?;
        context.use_program(Some(&program));

        context.uniform1f(Some(&uniforms["scale"]), 0.2);
        context.uniform3f(Some(&uniforms["position"]), 0.0, 0.0, 0.0);
        context.uniform4f(Some(&uniforms["circle_color"]), 0.5, 0.5, 0.5, 1.0);
        context.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );
        context.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);

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

    pub fn save(&self, path: &Path) -> Result<(), JsValue> {
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
            0,
        );
        context.viewport(0, 0, WORK_TEXTURE_WIDTH as i32, WORK_TEXTURE_HEIGHT as i32);
    
        let mut data: Vec<u8> = vec![0; WORK_TEXTURE_WIDTH * WORK_TEXTURE_HEIGHT * 4];
        context.read_pixels_with_opt_u8_array(
            0,
            0,
            WORK_TEXTURE_WIDTH as i32,
            WORK_TEXTURE_HEIGHT as i32,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            Some(data.as_mut_slice()),
        )?;
        context.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);

        let data =
            image::RgbaImage::from_vec(WORK_TEXTURE_WIDTH as u32, WORK_TEXTURE_HEIGHT as u32, data)
                .unwrap();
        write_image(path, data);
        Ok(())
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
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    yew::start_app::<yew_test::Model>();

    let app = App::new()?;
    app.read_image_to_work_texture(Path::new("./pano-rs/panorama_image_transfer.png"))?;
    app.draw_circle()?;
    app.show()?;
    app.save(Path::new("./panorama_image_transfer.png"))?;

    let app = Arc::new(RwLock::new(app));

    let f = Arc::new(RwLock::new(None));
    let g = f.clone();
    let mouse_on = Arc::new(RwLock::new(false));

    {
        let app = app.clone();
        *g.write().unwrap() = Some(Closure::wrap(Box::new(move || {
            app.read().unwrap().show().unwrap();
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
        let app = app.clone();
        let mouse_on = mouse_on.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            if *mouse_on.read().unwrap() {
                app.write()
                    .unwrap()
                    .increase_rotation_y(0.3 * event.movement_x() as f32);
                app.write()
                    .unwrap()
                    .increase_rotation_x(-0.3 * event.movement_y() as f32);
            }
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let app = app.clone();
        let mouse_on = mouse_on.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
            *mouse_on.write().unwrap() = false;
            app.write().unwrap().modify_rotation();
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let app = app.clone();
        let mouse_on = mouse_on.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
            *mouse_on.write().unwrap() = false;
            app.write().unwrap().modify_rotation();
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mouseout", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    request_animation_frame(g.read().unwrap().as_ref().unwrap());

    Ok(())
}
