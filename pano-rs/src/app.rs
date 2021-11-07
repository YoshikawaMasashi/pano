extern crate console_error_panic_hook;

use std::panic;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlShader, WebGlTexture};
use yew::prelude::*;

use crate::file_io::{read_image, write_image};
use crate::webgl_utils::{get_uniform_locations, link_program, read_shader};

const WORK_TEXTURE_WIDTH: usize = 3840;
const WORK_TEXTURE_HEIGHT: usize = 1920;

pub enum Msg {
    AddOne,
    MouseDownCanvas,
    MouseMoveCanvas { movement_x: f32, movement_y: f32 },
    MouseUpCanvas,
    RenderCanvas,
}

pub struct ModelWebGL {
    context: WebGl2RenderingContext,
    work_texture: Arc<Mutex<WebGlTexture>>,
    show_panorama_vert_shader: WebGlShader,
    show_panorama_frag_shader: WebGlShader,
    draw_circle_vert_shader: WebGlShader,
    draw_circle_frag_shader: WebGlShader,
}

pub struct Model {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    value: i64,

    webgl: Option<ModelWebGL>,

    rotation_x: f32,
    rotation_y: f32,
    mouse_on: bool,

    render_canvas_f: Arc<RwLock<Option<Closure<dyn FnMut()>>>>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            value: 0,
            webgl: None,
            rotation_x: 0.0,
            rotation_y: 0.0,
            mouse_on: false,
            render_canvas_f: Arc::new(RwLock::new(None)),
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            let document = web_sys::window().unwrap().document().unwrap();
            let canvas = document.get_element_by_id("canvas").unwrap();
            let canvas: web_sys::HtmlCanvasElement =
                canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

            let context = canvas
                .get_context("webgl2")
                .unwrap()
                .unwrap()
                .dyn_into::<WebGl2RenderingContext>()
                .unwrap();
            context.clear_color(0.0, 0.0, 0.0, 1.0);
            context.enable(WebGl2RenderingContext::BLEND);

            let show_panorama_vert_shader = read_shader(
                Path::new("./pano-rs/src/show_panorama.vert"),
                &context,
                WebGl2RenderingContext::VERTEX_SHADER,
            )
            .unwrap();
            let show_panorama_frag_shader = read_shader(
                Path::new("./pano-rs/src/show_panorama.frag"),
                &context,
                WebGl2RenderingContext::FRAGMENT_SHADER,
            )
            .unwrap();
            let draw_circle_vert_shader = read_shader(
                Path::new("./pano-rs/src/draw_circle.vert"),
                &context,
                WebGl2RenderingContext::VERTEX_SHADER,
            )
            .unwrap();
            let draw_circle_frag_shader = read_shader(
                Path::new("./pano-rs/src/draw_circle.frag"),
                &context,
                WebGl2RenderingContext::FRAGMENT_SHADER,
            )
            .unwrap();

            let work_texture = context.create_texture().unwrap();

            context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&work_texture));
            context
                .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                    WebGl2RenderingContext::TEXTURE_2D,
                    0,
                    WebGl2RenderingContext::RGBA as i32,
                    WORK_TEXTURE_WIDTH as i32,
                    WORK_TEXTURE_HEIGHT as i32,
                    0,
                    WebGl2RenderingContext::RGBA,
                    WebGl2RenderingContext::UNSIGNED_BYTE,
                    None,
                )
                .unwrap();
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

            self.webgl = Some(ModelWebGL {
                context,
                work_texture: Arc::new(Mutex::new(work_texture)),
                show_panorama_vert_shader,
                show_panorama_frag_shader,
                draw_circle_vert_shader,
                draw_circle_frag_shader,
            });

            self.read_image_to_work_texture(Path::new("./pano-rs/panorama_image_transfer.png"))
                .unwrap();
            self.show().unwrap();
            self.save(Path::new("./panorama_image_transfer.png"))
                .unwrap();

            let link = self.link.clone();
            *self.render_canvas_f.write().unwrap() = Some(Closure::wrap(Box::new(move || {
                link.send_message(Msg::RenderCanvas)
            })));
            request_animation_frame(self.render_canvas_f.read().unwrap().as_ref().unwrap());
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddOne => {
                self.value += 1;
                self.draw_circle(
                    0.05,
                    (-self.rotation_x, self.rotation_y, 0.0),
                    (0.7, 0.7, 0.7, 1.0),
                )
                .unwrap();
                // the value has changed so we need to
                // re-render for it to appear on the page
                true
            }
            Msg::MouseDownCanvas => {
                self.mouse_on = true;
                false
            }
            Msg::MouseMoveCanvas {
                movement_x,
                movement_y,
            } => {
                if self.mouse_on {
                    self.rotation_y += 0.3 * movement_x;
                    self.rotation_x -= 0.3 * movement_y;
                }
                false
            }
            Msg::MouseUpCanvas => {
                self.mouse_on = false;
                self.modify_rotation();
                false
            }
            Msg::RenderCanvas => {
                self.show().unwrap();
                request_animation_frame(self.render_canvas_f.read().unwrap().as_ref().unwrap());
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <p>{"Hello World!"}</p>
                <canvas
                    id="canvas"
                    height="960"
                    width="960"
                    onmousedown=self.link.callback(|_| Msg::MouseDownCanvas)
                    onmouseup=self.link.callback(|_| Msg::MouseUpCanvas)
                    onmouseout=self.link.callback(|_| Msg::MouseUpCanvas)
                    onmousemove=self.link.callback(|e: web_sys::MouseEvent| Msg::MouseMoveCanvas{movement_x: e.movement_x() as f32, movement_y: e.movement_y() as f32})
                ></canvas>
                <button onclick=self.link.callback(|_| Msg::AddOne)>{ "+1" }</button>
                <p>{ self.value }</p>
            </div>
        }
    }
}

impl Model {
    pub fn read_image_to_work_texture(&self, path: &Path) -> Result<(), JsValue> {
        let webgl = self.webgl.as_ref().unwrap();
        let image = read_image(path);
        assert_eq!(image.width(), WORK_TEXTURE_WIDTH as u32);
        assert_eq!(image.height(), WORK_TEXTURE_HEIGHT as u32);

        let work_texture = webgl.work_texture.lock().unwrap();
        webgl
            .context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&work_texture));
        webgl
            .context
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
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
        webgl.context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );
        webgl.context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );
        webgl
            .context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        Ok(())
    }

    pub fn draw_circle(
        &self,
        scale: f32,
        position: (f32, f32, f32),
        circle_color: (f32, f32, f32, f32),
    ) -> Result<(), JsValue> {
        let webgl = self.webgl.as_ref().unwrap();
        let frame_buffer = webgl.context.create_framebuffer().unwrap();
        webgl
            .context
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&frame_buffer));

        let work_texture = webgl.work_texture.lock().unwrap();
        webgl.context.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&work_texture),
            0,
        );
        webgl
            .context
            .viewport(0, 0, WORK_TEXTURE_WIDTH as i32, WORK_TEXTURE_HEIGHT as i32);

        let program = link_program(
            &webgl.context,
            &webgl.draw_circle_vert_shader,
            &webgl.draw_circle_frag_shader,
        )?;
        let uniforms = get_uniform_locations(
            &webgl.context,
            &program,
            vec![
                "scale".to_string(),
                "position".to_string(),
                "circle_color".to_string(),
            ],
        )?;
        webgl.context.use_program(Some(&program));

        webgl.context.uniform1f(Some(&uniforms["scale"]), scale);
        webgl.context.uniform3f(
            Some(&uniforms["position"]),
            position.0,
            position.1,
            position.2,
        );
        webgl.context.uniform4f(
            Some(&uniforms["circle_color"]),
            circle_color.0,
            circle_color.1,
            circle_color.2,
            circle_color.3,
        );
        webgl.context.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );
        webgl
            .context
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);

        webgl
            .context
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);

        Ok(())
    }

    pub fn show(&self) -> Result<(), JsValue> {
        let webgl = self.webgl.as_ref().unwrap();

        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

        webgl.context.clear_color(0.0, 0.0, 0.0, 1.0);
        webgl
            .context
            .viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
        let program = link_program(
            &webgl.context,
            &webgl.show_panorama_vert_shader,
            &webgl.show_panorama_frag_shader,
        )?;
        let uniforms = get_uniform_locations(
            &webgl.context,
            &program,
            vec![
                "tex".to_string(),
                "rotation_x".to_string(),
                "rotation_y".to_string(),
            ],
        )?;
        webgl.context.use_program(Some(&program));

        let work_texture = webgl.work_texture.lock().unwrap();
        webgl
            .context
            .clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        webgl
            .context
            .active_texture(WebGl2RenderingContext::TEXTURE0);
        webgl
            .context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&work_texture));
        webgl.context.uniform1i(Some(&uniforms["tex"]), 0);
        webgl
            .context
            .uniform1f(Some(&uniforms["rotation_x"]), self.rotation_x);
        webgl
            .context
            .uniform1f(Some(&uniforms["rotation_y"]), self.rotation_y);
        webgl
            .context
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
        webgl
            .context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        Ok(())
    }

    pub fn save(&self, path: &Path) -> Result<(), JsValue> {
        let webgl = self.webgl.as_ref().unwrap();
        let frame_buffer = webgl.context.create_framebuffer().unwrap();
        webgl
            .context
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&frame_buffer));

        let work_texture = webgl.work_texture.lock().unwrap();
        webgl.context.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&work_texture),
            0,
        );
        webgl
            .context
            .viewport(0, 0, WORK_TEXTURE_WIDTH as i32, WORK_TEXTURE_HEIGHT as i32);

        let mut data: Vec<u8> = vec![0; WORK_TEXTURE_WIDTH * WORK_TEXTURE_HEIGHT * 4];
        webgl.context.read_pixels_with_opt_u8_array(
            0,
            0,
            WORK_TEXTURE_WIDTH as i32,
            WORK_TEXTURE_HEIGHT as i32,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            Some(data.as_mut_slice()),
        )?;
        webgl
            .context
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);

        let data =
            image::RgbaImage::from_vec(WORK_TEXTURE_WIDTH as u32, WORK_TEXTURE_HEIGHT as u32, data)
                .unwrap();
        write_image(path, data);
        Ok(())
    }

    fn modify_rotation(&mut self) {
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

    yew::start_app::<Model>();

    Ok(())
}
