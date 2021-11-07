extern crate console_error_panic_hook;

use std::panic;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlShader, WebGlTexture};
use yew::prelude::*;

use crate::file_io::{read_image, write_image};
use crate::webgl_utils::{compile_shader, get_uniform_locations, link_program};

const WORK_TEXTURE_WIDTH: usize = 3840;
const WORK_TEXTURE_HEIGHT: usize = 1920;

pub enum Msg {
    AddOne,
    MouseDownCanvas,
    MouseMoveCanvas { movement_x: f32, movement_y: f32 },
    MouseUpCanvas,
    RenderCanvas,
    KeyDown { key_code: u32 },
    ExportPng,
    ImportPng,
    SwitchEnableGrid,
}

pub struct ModelWebGL {
    context: WebGl2RenderingContext,
    work_texture: Arc<Mutex<WebGlTexture>>,
    show_panorama_vert_shader: WebGlShader,
    show_panorama_frag_shader: WebGlShader,
    draw_circle_vert_shader: WebGlShader,
    draw_circle_frag_shader: WebGlShader,
    alpha_grid_vert_shader: WebGlShader,
    alpha_grid_frag_shader: WebGlShader,
    grid_vert_shader: WebGlShader,
    grid_frag_shader: WebGlShader,
}

pub struct Model {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    value: i64,

    webgl: Option<Arc<RwLock<ModelWebGL>>>,

    rotation_x: f32,
    rotation_y: f32,
    mouse_on: bool,

    cubes_to_equirectangular_dialog_open: bool,
    enable_grid: bool,

    render_canvas_f: Arc<RwLock<Option<Closure<dyn FnMut()>>>>,
    key_down_f: Arc<RwLock<Option<Closure<dyn FnMut(web_sys::KeyboardEvent)>>>>,
    export_png_f: Arc<RwLock<Option<Closure<dyn FnMut()>>>>,
    import_png_f: Arc<RwLock<Option<Closure<dyn FnMut()>>>>,
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

            cubes_to_equirectangular_dialog_open: false,
            enable_grid: false,

            render_canvas_f: Arc::new(RwLock::new(None)),
            key_down_f: Arc::new(RwLock::new(None)),
            export_png_f: Arc::new(RwLock::new(None)),
            import_png_f: Arc::new(RwLock::new(None)),
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            let document = web_sys::window().unwrap().document().unwrap();

            let body = document.body().unwrap();
            let link = self.link.clone();
            *self.key_down_f.write().unwrap() = Some(Closure::wrap(Box::new(
                move |event: web_sys::KeyboardEvent| {
                    link.send_message(Msg::KeyDown {
                        key_code: event.key_code(),
                    });
                },
            )
                as Box<dyn FnMut(_)>));
            body.add_event_listener_with_callback(
                "keydown",
                self.key_down_f
                    .read()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unchecked_ref(),
            )
            .unwrap();

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

            // when development, we can use read_shader with path
            /*
            let show_panorama_vert_shader = crate::webgl_utils::read_shader(
                Path::new("./pano-rs/src/show_panorama.vert"),
                &context,
                WebGl2RenderingContext::VERTEX_SHADER,
            )
            .unwrap();
            */
            let show_panorama_vert_shader = compile_shader(
                &context,
                WebGl2RenderingContext::VERTEX_SHADER,
                include_str!("../shaders/show_panorama.vert"),
            )
            .unwrap();
            let show_panorama_frag_shader = compile_shader(
                &context,
                WebGl2RenderingContext::FRAGMENT_SHADER,
                include_str!("../shaders/show_panorama.frag"),
            )
            .unwrap();
            let draw_circle_vert_shader = compile_shader(
                &context,
                WebGl2RenderingContext::VERTEX_SHADER,
                include_str!("../shaders/draw_circle.vert"),
            )
            .unwrap();
            let draw_circle_frag_shader = compile_shader(
                &context,
                WebGl2RenderingContext::FRAGMENT_SHADER,
                include_str!("../shaders/draw_circle.frag"),
            )
            .unwrap();
            let alpha_grid_vert_shader = compile_shader(
                &context,
                WebGl2RenderingContext::VERTEX_SHADER,
                include_str!("../shaders/alpha_grid.vert"),
            )
            .unwrap();
            let alpha_grid_frag_shader = compile_shader(
                &context,
                WebGl2RenderingContext::FRAGMENT_SHADER,
                include_str!("../shaders/alpha_grid.frag"),
            )
            .unwrap();
            let grid_vert_shader = compile_shader(
                &context,
                WebGl2RenderingContext::VERTEX_SHADER,
                include_str!("../shaders/grid.vert"),
            )
            .unwrap();
            let grid_frag_shader = compile_shader(
                &context,
                WebGl2RenderingContext::FRAGMENT_SHADER,
                include_str!("../shaders/grid.frag"),
            )
            .unwrap();
            /*
            let grid_vert_shader = crate::webgl_utils::read_shader(
                Path::new("./pano-rs/src/grid.vert"),
                &context,
                WebGl2RenderingContext::VERTEX_SHADER,
            )
            .unwrap();
            let grid_frag_shader = crate::webgl_utils::read_shader(
                Path::new("./pano-rs/src/grid.frag"),
                &context,
                WebGl2RenderingContext::FRAGMENT_SHADER,
            )
            .unwrap();
            */

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

            self.webgl = Some(Arc::new(RwLock::new(ModelWebGL {
                context,
                work_texture: Arc::new(Mutex::new(work_texture)),
                show_panorama_vert_shader,
                show_panorama_frag_shader,
                draw_circle_vert_shader,
                draw_circle_frag_shader,
                alpha_grid_vert_shader,
                alpha_grid_frag_shader,
                grid_vert_shader,
                grid_frag_shader,
            })));

            self.webgl
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .show(self.rotation_x, self.rotation_y, self.enable_grid)
                .unwrap();

            let link = self.link.clone();
            *self.render_canvas_f.write().unwrap() = Some(Closure::wrap(Box::new(move || {
                link.send_message(Msg::RenderCanvas)
            })));
            request_animation_frame(self.render_canvas_f.read().unwrap().as_ref().unwrap());

            let link = self.link.clone();
            *self.export_png_f.write().unwrap() = Some(Closure::wrap(Box::new(move || {
                link.send_message(Msg::ExportPng)
            })));
            crate::wasm_bind::set_on_click_export_png(
                self.export_png_f
                    .read()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unchecked_ref(),
            );

            let link = self.link.clone();
            *self.import_png_f.write().unwrap() = Some(Closure::wrap(Box::new(move || {
                link.send_message(Msg::ImportPng)
            })));
            crate::wasm_bind::set_on_click_import_png(
                self.import_png_f
                    .read()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unchecked_ref(),
            );
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddOne => {
                self.value += 1;
                self.webgl
                    .as_ref()
                    .unwrap()
                    .read()
                    .unwrap()
                    .draw_circle(
                        0.05,
                        (-self.rotation_x, self.rotation_y, 0.0),
                        (1.0, 0.5, 0.5, 1.0),
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
                self.webgl
                    .as_ref()
                    .unwrap()
                    .read()
                    .unwrap()
                    .show(self.rotation_x, self.rotation_y, self.enable_grid)
                    .unwrap();
                request_animation_frame(self.render_canvas_f.read().unwrap().as_ref().unwrap());
                false
            }
            Msg::KeyDown { key_code } => {
                // crate::console_log!("key down {}", key_code);
                if key_code == 54 {
                    // '6' key
                    self.cubes_to_equirectangular_dialog_open =
                        !self.cubes_to_equirectangular_dialog_open;
                    true
                } else {
                    false
                }
            }
            Msg::ExportPng => {
                let dialog_promise: js_sys::Promise =
                    crate::wasm_bind::show_save_png_dialog().unwrap().into();
                let webgl = self.webgl.as_ref().unwrap().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let path_or_undefined = wasm_bindgen_futures::JsFuture::from(dialog_promise)
                        .await
                        .unwrap();
                    if let Some(path) = path_or_undefined.as_string() {
                        webgl.read().unwrap().save_png(Path::new(&path)).unwrap();
                    }
                });
                false
            }
            Msg::ImportPng => {
                let dialog_promise: js_sys::Promise =
                    crate::wasm_bind::show_open_png_dialog().unwrap().into();
                let webgl = self.webgl.as_ref().unwrap().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let path_or_undefined = wasm_bindgen_futures::JsFuture::from(dialog_promise)
                        .await
                        .unwrap();
                    if let Some(path) = path_or_undefined.as_string() {
                        webgl
                            .read()
                            .unwrap()
                            .import_png_to_work_texture(Path::new(&path))
                            .unwrap();
                    }
                });
                false
            }
            Msg::SwitchEnableGrid => {
                self.enable_grid = !self.enable_grid;
                true
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
                <button onclick=self.link.callback(|_| Msg::AddOne)>{ "円を追加" }</button>
                <p>{"円の数"}{ self.value }</p>
                <button onclick=self.link.callback(|_| Msg::SwitchEnableGrid)>{ "グリッド" }</button>
                <canvas
                    id="canvas"
                    height="960"
                    width="960"
                    onmousedown=self.link.callback(|_| Msg::MouseDownCanvas)
                    onmouseup=self.link.callback(|_| Msg::MouseUpCanvas)
                    onmouseout=self.link.callback(|_| Msg::MouseUpCanvas)
                    onmousemove=self.link.callback(|e: web_sys::MouseEvent| Msg::MouseMoveCanvas{movement_x: e.movement_x() as f32, movement_y: e.movement_y() as f32})
                ></canvas>
                {
                    if self.cubes_to_equirectangular_dialog_open {
                        html! {
                           <div id="overlay"></div>
                        }
                    } else {
                        html! {
                        }
                    }
                }
                <div id="centerpoint">
                    <dialog
                        id="6cubes to equirectangular dialog"
                        open=self.cubes_to_equirectangular_dialog_open
                    >
                        {"6 cubes to equirectangular"}
                        <br />
                        {"6cubes images: front.png, back.png, left.png, right.png, top.png, botton.pngが入ったディレクトリを指定してください"}
                        <br />
                        <input/>
                        <button>{ "ファイルを選択" }</button>
                    </dialog>
                </div>
            </div>
        }
    }
}

impl Model {
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

impl ModelWebGL {
    pub fn import_png_to_work_texture(&self, path: &Path) -> Result<(), JsValue> {
        let image = read_image(path);
        assert_eq!(image.width(), WORK_TEXTURE_WIDTH as u32);
        assert_eq!(image.height(), WORK_TEXTURE_HEIGHT as u32);

        let work_texture = self.work_texture.lock().unwrap();
        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&work_texture));
        self.context
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
        self.context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );
        self.context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );
        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        Ok(())
    }

    pub fn draw_circle(
        &self,
        scale: f32,
        position: (f32, f32, f32),
        circle_color: (f32, f32, f32, f32),
    ) -> Result<(), JsValue> {
        let frame_buffer = self.context.create_framebuffer().unwrap();
        self.context
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&frame_buffer));

        let work_texture = self.work_texture.lock().unwrap();
        self.context.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&work_texture),
            0,
        );
        self.context
            .viewport(0, 0, WORK_TEXTURE_WIDTH as i32, WORK_TEXTURE_HEIGHT as i32);

        let program = link_program(
            &self.context,
            &self.draw_circle_vert_shader,
            &self.draw_circle_frag_shader,
        )?;
        let uniforms = get_uniform_locations(
            &self.context,
            &program,
            vec![
                "scale".to_string(),
                "position".to_string(),
                "circle_color".to_string(),
            ],
        )?;
        self.context.use_program(Some(&program));

        self.context.uniform1f(Some(&uniforms["scale"]), scale);
        self.context.uniform3f(
            Some(&uniforms["position"]),
            position.0,
            position.1,
            position.2,
        );
        self.context.uniform4f(
            Some(&uniforms["circle_color"]),
            circle_color.0,
            circle_color.1,
            circle_color.2,
            circle_color.3,
        );
        self.context.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );
        self.context
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);

        self.context
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);

        Ok(())
    }

    pub fn show(&self, rotation_x: f32, rotation_y: f32, enable_grid: bool) -> Result<(), JsValue> {
        self.show_alpha_grid(rotation_x, rotation_y)?;
        self.show_texture(rotation_x, rotation_y)?;
        if enable_grid {
            self.show_grid(rotation_x, rotation_y)?;
        }
        Ok(())
    }

    pub fn show_texture(&self, rotation_x: f32, rotation_y: f32) -> Result<(), JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

        self.context
            .viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
        let program = link_program(
            &self.context,
            &self.show_panorama_vert_shader,
            &self.show_panorama_frag_shader,
        )?;
        let uniforms = get_uniform_locations(
            &self.context,
            &program,
            vec![
                "tex".to_string(),
                "rotation_x".to_string(),
                "rotation_y".to_string(),
            ],
        )?;
        self.context.use_program(Some(&program));

        let work_texture = self.work_texture.lock().unwrap();
        self.context
            .active_texture(WebGl2RenderingContext::TEXTURE0);
        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&work_texture));
        self.context.uniform1i(Some(&uniforms["tex"]), 0);
        self.context
            .uniform1f(Some(&uniforms["rotation_x"]), rotation_x);
        self.context
            .uniform1f(Some(&uniforms["rotation_y"]), rotation_y);

        self.context.enable(WebGl2RenderingContext::BLEND);
        self.context.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );
        self.context
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        Ok(())
    }

    pub fn show_alpha_grid(&self, rotation_x: f32, rotation_y: f32) -> Result<(), JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

        self.context
            .viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
        let program = link_program(
            &self.context,
            &self.alpha_grid_vert_shader,
            &self.alpha_grid_frag_shader,
        )?;
        let uniforms = get_uniform_locations(
            &self.context,
            &program,
            vec!["rotation_x".to_string(), "rotation_y".to_string()],
        )?;
        self.context.use_program(Some(&program));

        self.context.enable(WebGl2RenderingContext::BLEND);
        self.context.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );

        self.context
            .uniform1f(Some(&uniforms["rotation_x"]), rotation_x);
        self.context
            .uniform1f(Some(&uniforms["rotation_y"]), rotation_y);
        self.context
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);

        Ok(())
    }

    pub fn show_grid(&self, rotation_x: f32, rotation_y: f32) -> Result<(), JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

        self.context
            .viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
        let program = link_program(
            &self.context,
            &self.grid_vert_shader,
            &self.grid_frag_shader,
        )?;
        let uniforms = get_uniform_locations(
            &self.context,
            &program,
            vec!["rotation_x".to_string(), "rotation_y".to_string()],
        )?;
        self.context.use_program(Some(&program));

        self.context
            .uniform1f(Some(&uniforms["rotation_x"]), rotation_x);
        self.context
            .uniform1f(Some(&uniforms["rotation_y"]), rotation_y);
        self.context
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);

        Ok(())
    }

    pub fn save_png(&self, path: &Path) -> Result<(), JsValue> {
        let frame_buffer = self.context.create_framebuffer().unwrap();
        self.context
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&frame_buffer));

        let work_texture = self.work_texture.lock().unwrap();
        self.context.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&work_texture),
            0,
        );
        self.context
            .viewport(0, 0, WORK_TEXTURE_WIDTH as i32, WORK_TEXTURE_HEIGHT as i32);

        let mut data: Vec<u8> = vec![0; WORK_TEXTURE_WIDTH * WORK_TEXTURE_HEIGHT * 4];
        self.context.read_pixels_with_opt_u8_array(
            0,
            0,
            WORK_TEXTURE_WIDTH as i32,
            WORK_TEXTURE_HEIGHT as i32,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            Some(data.as_mut_slice()),
        )?;
        self.context
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);

        let data =
            image::RgbaImage::from_vec(WORK_TEXTURE_WIDTH as u32, WORK_TEXTURE_HEIGHT as u32, data)
                .unwrap();
        write_image(path, data);
        Ok(())
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
