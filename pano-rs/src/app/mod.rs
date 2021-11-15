extern crate console_error_panic_hook;

mod cubes_to_equirectangular_dialog;
mod image_transfer_dialog;

use std::panic;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlDivElement, WebGl2RenderingContext, WebGlShader, WebGlTexture};
use yew::prelude::*;
use yew::{html, ChangeData, Html, InputData};

use crate::file_io::{read_image, write_image};
use crate::webgl_utils::{compile_shader, get_uniform_locations, link_program};
use cubes_to_equirectangular_dialog::CubesToEquirectangularDialog;
use image_transfer_dialog::ImageTransferDialog;

const WORK_TEXTURE_WIDTH: usize = 3840;
const WORK_TEXTURE_HEIGHT: usize = 1920;

#[derive(PartialEq, Eq)]
pub enum Dialog {
    None,
    CubesToEquirectangular,
    ImageTransfer,
}

impl Dialog {
    fn open(&self) -> bool {
        match self {
            Dialog::None => false,
            _ => true,
        }
    }
    fn cubes_to_equirectangular_dialog_open(&self) -> bool {
        match self {
            Dialog::CubesToEquirectangular => true,
            _ => false,
        }
    }
    fn image_transfer_dialog_open(&self) -> bool {
        match self {
            Dialog::ImageTransfer => true,
            _ => false,
        }
    }
}

pub enum Msg {
    MouseDownCanvas {
        button: i16,
        offset_x: f32,
        offset_y: f32,
    },
    MouseMoveCanvas {
        movement_x: f32,
        movement_y: f32,
        offset_x: f32,
        offset_y: f32,
    },
    MouseUpCanvas,
    RenderCanvas,
    KeyDown {
        key_code: u32,
    },
    ExportPng,
    ImportPng,
    SwitchEnableGrid,
    ChangeMainCanvasSize {
        height: f32,
        width: f32,
    },
    ChangeFOV {
        fov: f32,
    },
}

pub struct Model {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    webgl: Option<Arc<RwLock<ModelWebGL>>>,

    rotation_x: f32,
    rotation_y: f32,
    scroll_mouse_on: bool,
    left_mouse_on: bool,
    prev_mouse_point: Option<(f32, f32)>,

    dialog: Dialog,
    enable_grid: bool,

    app_height: f32,
    app_width: f32,
    main_canvas_size: f32,
    fov: f32,
    yew_root_ref: NodeRef,

    render_canvas_f: Arc<RwLock<Option<Closure<dyn FnMut()>>>>,
    key_down_f: Arc<RwLock<Option<Closure<dyn FnMut(web_sys::KeyboardEvent)>>>>,
    export_png_f: Arc<RwLock<Option<Closure<dyn FnMut()>>>>,
    import_png_f: Arc<RwLock<Option<Closure<dyn FnMut()>>>>,
    resize_f: Arc<RwLock<Option<Closure<dyn FnMut()>>>>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            webgl: None,
            rotation_x: 0.0,
            rotation_y: 0.0,
            scroll_mouse_on: false,
            left_mouse_on: false,
            prev_mouse_point: None,

            dialog: Dialog::None,
            enable_grid: false,

            app_height: 960.0,
            app_width: 960.0,
            main_canvas_size: 950.0,
            fov: 60.0,
            yew_root_ref: NodeRef::default(),

            render_canvas_f: Arc::new(RwLock::new(None)),
            key_down_f: Arc::new(RwLock::new(None)),
            export_png_f: Arc::new(RwLock::new(None)),
            import_png_f: Arc::new(RwLock::new(None)),
            resize_f: Arc::new(RwLock::new(None)),
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();

            let link = self.link.clone();
            let yew_root_ref = self.yew_root_ref.clone();
            *self.resize_f.write().unwrap() = Some(Closure::wrap(Box::new(move || {
                if let Some(main_canvas_wrapper) = yew_root_ref.cast::<HtmlDivElement>() {
                    link.send_message(Msg::ChangeMainCanvasSize {
                        height: main_canvas_wrapper.offset_height() as f32,
                        width: main_canvas_wrapper.offset_width() as f32,
                    });
                }
            })
                as Box<dyn FnMut()>));
            window
                .add_event_listener_with_callback(
                    "resize",
                    self.resize_f
                        .read()
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .unchecked_ref(),
                )
                .unwrap();

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

            let canvas = document.get_element_by_id("main_canvas").unwrap();
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
                Path::new("./pano-rs/src/shaders/show_panorama.vert"),
                &context,
                WebGl2RenderingContext::VERTEX_SHADER,
            )
            .unwrap();
            */
            let all_view_vert_shader = compile_shader(
                &context,
                WebGl2RenderingContext::VERTEX_SHADER,
                include_str!("../shaders/all_view.vert"),
            )
            .unwrap();
            let drawing_canvas_vert_shader = compile_shader(
                &context,
                WebGl2RenderingContext::VERTEX_SHADER,
                include_str!("../shaders/drawing_canvas.vert"),
            )
            .unwrap();
            let brush_vert_shader = crate::webgl_utils::read_shader(
                Path::new("./pano-rs/src/shaders/brush.vert"),
                &context,
                WebGl2RenderingContext::VERTEX_SHADER,
            )
            .unwrap();
            /*
            let brush_vert_shader = compile_shader(
                &context,
                WebGl2RenderingContext::VERTEX_SHADER,
                include_str!("../shaders/brush.vert"),
            )
            .unwrap();
            */

            let show_panorama_frag_shader = compile_shader(
                &context,
                WebGl2RenderingContext::FRAGMENT_SHADER,
                include_str!("../shaders/show_panorama.frag"),
            )
            .unwrap();
            let draw_circle_frag_shader = compile_shader(
                &context,
                WebGl2RenderingContext::FRAGMENT_SHADER,
                include_str!("../shaders/draw_circle.frag"),
            )
            .unwrap();
            let alpha_grid_frag_shader = compile_shader(
                &context,
                WebGl2RenderingContext::FRAGMENT_SHADER,
                include_str!("../shaders/alpha_grid.frag"),
            )
            .unwrap();
            let grid_frag_shader = compile_shader(
                &context,
                WebGl2RenderingContext::FRAGMENT_SHADER,
                include_str!("../shaders/grid.frag"),
            )
            .unwrap();
            let brush_frag_shader = crate::webgl_utils::read_shader(
                Path::new("./pano-rs/src/shaders/brush.frag"),
                &context,
                WebGl2RenderingContext::FRAGMENT_SHADER,
            )
            .unwrap();
            let panorama_rotation_frag_shader = crate::webgl_utils::read_shader(
                Path::new("./pano-rs/src/shaders/panorama_rotation.frag"),
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

            self.webgl = Some(Arc::new(RwLock::new(ModelWebGL {
                context,
                work_texture: Arc::new(Mutex::new(work_texture)),
                brush_texture: Arc::new(Mutex::new(None)),

                all_view_vert_shader,
                drawing_canvas_vert_shader,
                brush_vert_shader,
                brush_dist: 0.0,

                show_panorama_frag_shader,
                draw_circle_frag_shader,
                alpha_grid_frag_shader,
                grid_frag_shader,
                brush_frag_shader,
                panorama_rotation_frag_shader,
            })));

            self.webgl
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .show(self.rotation_x, self.rotation_y, self.fov, self.enable_grid)
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
            Msg::MouseDownCanvas {
                button,
                offset_x,
                offset_y,
            } => {
                if button == 0 {
                    self.left_mouse_on = true;
                    self.prev_mouse_point = Some((offset_x, offset_y));
                    self.webgl
                        .as_ref()
                        .unwrap()
                        .write()
                        .unwrap()
                        .start_brush()
                        .unwrap();
                } else if button == 1 {
                    self.scroll_mouse_on = true;
                }
                false
            }
            Msg::MouseMoveCanvas {
                movement_x,
                movement_y,
                offset_x,
                offset_y,
            } => {
                if self.scroll_mouse_on {
                    self.rotation_y += 0.3 * movement_x;
                    self.rotation_x -= 0.3 * movement_y;
                }
                if self.left_mouse_on {
                    let prev_mouse_point = self.prev_mouse_point.unwrap();
                    let factor = (self.fov / 2.0 / 180.0 * std::f32::consts::PI).tan();
                    self.webgl
                        .as_ref()
                        .unwrap()
                        .write()
                        .unwrap()
                        .draw_brush(
                            (
                                (2.0 * prev_mouse_point.0 / self.main_canvas_size as f32 - 1.0)
                                    * factor,
                                (1.0 - 2.0 * (prev_mouse_point.1 / self.main_canvas_size as f32))
                                    * factor,
                                1.0,
                            ),
                            (
                                (2.0 * offset_x / self.main_canvas_size as f32 - 1.0) * factor,
                                (1.0 - 2.0 * offset_y / self.main_canvas_size as f32) * factor,
                                1.0,
                            ),
                        )
                        .unwrap();
                    self.prev_mouse_point = Some((offset_x, offset_y));
                }
                false
            }
            Msg::MouseUpCanvas => {
                if self.left_mouse_on {
                    self.left_mouse_on = false;
                    self.prev_mouse_point = None;
                    self.webgl
                        .as_ref()
                        .unwrap()
                        .write()
                        .unwrap()
                        .stop_brush(self.rotation_x, self.rotation_y)
                        .unwrap();
                } else if self.scroll_mouse_on {
                    self.scroll_mouse_on = false;
                    self.modify_rotation();
                }
                false
            }
            Msg::RenderCanvas => {
                self.webgl
                    .as_ref()
                    .unwrap()
                    .read()
                    .unwrap()
                    .show(self.rotation_x, self.rotation_y, self.fov, self.enable_grid)
                    .unwrap();
                request_animation_frame(self.render_canvas_f.read().unwrap().as_ref().unwrap());
                false
            }
            Msg::KeyDown { key_code } => {
                // crate::console_log!("key down {}", key_code);
                if key_code == 54 {
                    // '6' key
                    if self.dialog == Dialog::CubesToEquirectangular {
                        self.dialog = Dialog::None;
                    } else {
                        self.dialog = Dialog::CubesToEquirectangular;
                    }
                    true
                } else if key_code == 84 {
                    // 't' key
                    if self.dialog == Dialog::ImageTransfer {
                        self.dialog = Dialog::None;
                    } else {
                        self.dialog = Dialog::ImageTransfer;
                    }
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
            Msg::ChangeMainCanvasSize { height, width } => {
                self.app_height = height;
                self.app_width = width;
                if height > width {
                    self.main_canvas_size = width - 10.0;
                } else {
                    self.main_canvas_size = height - 10.0;
                }
                true
            }
            Msg::ChangeFOV { fov } => {
                self.fov = fov;
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
            <div
                id="yew_root"
                ref={self.yew_root_ref.clone()}
            >
                <canvas
                    id="main_canvas"
                    height=self.main_canvas_size.to_string()
                    width=self.main_canvas_size.to_string()
                    onmousedown=self.link.callback(|e: web_sys::MouseEvent| Msg::MouseDownCanvas{button: e.button(), offset_x: e.offset_x() as f32,
                        offset_y: e.offset_y() as f32})
                    onmouseup=self.link.callback(|_| Msg::MouseUpCanvas)
                    onmouseout=self.link.callback(|_| Msg::MouseUpCanvas)
                    onmousemove=self.link.callback(|e: web_sys::MouseEvent|
                        Msg::MouseMoveCanvas{
                            movement_x: e.movement_x() as f32,
                            movement_y: e.movement_y() as f32,
                            offset_x: e.offset_x() as f32,
                            offset_y: e.offset_y() as f32
                        }

                    )
                />
                <div id="tool">
                    <button onclick=self.link.callback(|_| Msg::SwitchEnableGrid)>{ "グリッド" }</button>
                    <input
                        type="range"
                        id="volume"
                        name="volume"
                        min="5"
                        max="120"
                        value=self.fov.to_string()
                        oninput=self.link.callback(|e: InputData| Msg::ChangeFOV{fov: e.value.parse::<f32>().unwrap()})
                        onchange=self.link.batch_callback(|e: ChangeData| {
                            if let ChangeData::Value(value) = e {
                                Some(Msg::ChangeFOV{fov: value.parse::<f32>().unwrap()})
                            } else {
                                None
                            }})
                    />
                    <label for="volume">{"FOV"}</label>
                </div>
                <div id="dialog">
                    {
                        if self.dialog.open() {
                            html! {
                            <div id="overlay"></div>
                            }
                        } else {
                            html! {
                            }
                        }
                    }
                    <CubesToEquirectangularDialog
                        open=self.dialog.cubes_to_equirectangular_dialog_open()
                    />
                    <ImageTransferDialog
                        open=self.dialog.image_transfer_dialog_open()
                    />
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

pub struct ModelWebGL {
    context: WebGl2RenderingContext,
    work_texture: Arc<Mutex<WebGlTexture>>,
    brush_texture: Arc<Mutex<Option<WebGlTexture>>>,
    brush_dist: f32,

    all_view_vert_shader: WebGlShader,
    drawing_canvas_vert_shader: WebGlShader,
    brush_vert_shader: WebGlShader,

    show_panorama_frag_shader: WebGlShader,
    draw_circle_frag_shader: WebGlShader,
    alpha_grid_frag_shader: WebGlShader,
    grid_frag_shader: WebGlShader,
    brush_frag_shader: WebGlShader,
    panorama_rotation_frag_shader: WebGlShader,
}

impl ModelWebGL {
    pub fn import_png_to_work_texture(&self, path: &Path) -> Result<(), JsValue> {
        let image = read_image(path);
        assert_eq!(image.height(), WORK_TEXTURE_HEIGHT as u32);
        assert_eq!(image.width(), WORK_TEXTURE_WIDTH as u32);

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
            &self.all_view_vert_shader,
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

    pub fn show(
        &self,
        rotation_x: f32,
        rotation_y: f32,
        fov: f32,
        enable_grid: bool,
    ) -> Result<(), JsValue> {
        self.show_alpha_grid(rotation_x, rotation_y, fov)?;
        self.show_texture(rotation_x, rotation_y, fov)?;
        self.show_brush_texture(fov)?;
        if enable_grid {
            self.show_grid(rotation_x, rotation_y, fov)?;
        }
        Ok(())
    }

    pub fn show_texture(&self, rotation_x: f32, rotation_y: f32, fov: f32) -> Result<(), JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("main_canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

        self.context
            .viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
        let program = link_program(
            &self.context,
            &self.drawing_canvas_vert_shader,
            &self.show_panorama_frag_shader,
        )?;
        let uniforms = get_uniform_locations(
            &self.context,
            &program,
            vec![
                "fov".to_string(),
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
        self.context.uniform1f(Some(&uniforms["fov"]), fov);
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

    pub fn show_alpha_grid(
        &self,
        rotation_x: f32,
        rotation_y: f32,
        fov: f32,
    ) -> Result<(), JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("main_canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

        self.context
            .viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
        let program = link_program(
            &self.context,
            &self.drawing_canvas_vert_shader,
            &self.alpha_grid_frag_shader,
        )?;
        let uniforms = get_uniform_locations(
            &self.context,
            &program,
            vec![
                "fov".to_string(),
                "rotation_x".to_string(),
                "rotation_y".to_string(),
            ],
        )?;
        self.context.use_program(Some(&program));

        self.context.enable(WebGl2RenderingContext::BLEND);
        self.context.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );

        self.context.uniform1f(Some(&uniforms["fov"]), fov);
        self.context
            .uniform1f(Some(&uniforms["rotation_x"]), rotation_x);
        self.context
            .uniform1f(Some(&uniforms["rotation_y"]), rotation_y);
        self.context
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);

        Ok(())
    }

    pub fn show_grid(&self, rotation_x: f32, rotation_y: f32, fov: f32) -> Result<(), JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("main_canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

        self.context
            .viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
        let program = link_program(
            &self.context,
            &self.drawing_canvas_vert_shader,
            &self.grid_frag_shader,
        )?;
        let uniforms = get_uniform_locations(
            &self.context,
            &program,
            vec![
                "fov".to_string(),
                "rotation_x".to_string(),
                "rotation_y".to_string(),
            ],
        )?;
        self.context.use_program(Some(&program));

        self.context.uniform1f(Some(&uniforms["fov"]), fov);
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

    pub fn start_brush(&mut self) -> Result<(), JsValue> {
        let brush_texture = self.context.create_texture().unwrap();

        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&brush_texture));
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
                None,
            )
            .unwrap();
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

        *self.brush_texture.lock().unwrap() = Some(brush_texture);

        self.brush_dist = 0.0;
        Ok(())
    }

    pub fn draw_brush(
        &mut self,
        start_position: (f32, f32, f32),
        end_position: (f32, f32, f32),
    ) -> Result<(), JsValue> {
        let x1 = (start_position.0 * start_position.0
            + start_position.1 * start_position.1
            + start_position.2 * start_position.2)
            .sqrt();
        let x1 = (
            start_position.0 / x1,
            start_position.1 / x1,
            start_position.2 / x1,
        );
        let x2 = (end_position.0 * end_position.0
            + end_position.1 * end_position.1
            + end_position.2 * end_position.2)
            .sqrt();
        let x2 = (
            end_position.0 / x2,
            end_position.1 / x2,
            end_position.2 / x2,
        );
        let dist = (x1.0 * x2.0 + x1.1 * x2.1 + x1.2 * x2.2).abs().acos() / 0.02;

        let frame_buffer = self.context.create_framebuffer().unwrap();
        self.context
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&frame_buffer));

        self.context.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(self.brush_texture.lock().unwrap().as_ref().unwrap()),
            0,
        );
        self.context
            .viewport(0, 0, WORK_TEXTURE_WIDTH as i32, WORK_TEXTURE_HEIGHT as i32);

        let program = link_program(
            &self.context,
            &self.brush_vert_shader,
            &self.brush_frag_shader,
        )?;
        let uniforms = get_uniform_locations(
            &self.context,
            &program,
            vec![
                "start_position".to_string(),
                "end_position".to_string(),
                "point_num".to_string(),
                "point_offset".to_string(),
            ],
        )?;
        self.context.use_program(Some(&program));
        self.context.uniform3f(
            Some(&uniforms["start_position"]),
            start_position.0,
            start_position.1,
            start_position.2,
        );
        self.context.uniform3f(
            Some(&uniforms["end_position"]),
            end_position.0,
            end_position.1,
            end_position.2,
        );
        self.context.uniform1i(
            Some(&uniforms["point_num"]),
            ((self.brush_dist + dist).floor() as i32) - self.brush_dist.floor() as i32,
        );
        self.context
            .uniform1f(Some(&uniforms["point_offset"]), self.brush_dist % 1.0);

        self.context.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );
        self.context
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 24);

        self.context
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);

        self.brush_dist += dist;
        Ok(())
    }
    pub fn stop_brush(&mut self, rotation_x: f32, rotation_y: f32) -> Result<(), JsValue> {
        if let Some(brush_texture) = self.brush_texture.lock().unwrap().as_ref() {
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
                &self.all_view_vert_shader,
                &self.panorama_rotation_frag_shader,
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

            self.context
                .active_texture(WebGl2RenderingContext::TEXTURE0);
            self.context
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(brush_texture));
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

            self.context
                .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
        }

        *self.brush_texture.lock().unwrap() = None;
        Ok(())
    }

    pub fn show_brush_texture(&self, fov: f32) -> Result<(), JsValue> {
        if let Some(brush_texture) = self.brush_texture.lock().unwrap().as_ref() {
            let document = web_sys::window().unwrap().document().unwrap();
            let canvas = document.get_element_by_id("main_canvas").unwrap();
            let canvas: web_sys::HtmlCanvasElement =
                canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

            self.context
                .viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
            let program = link_program(
                &self.context,
                &self.drawing_canvas_vert_shader,
                &self.show_panorama_frag_shader,
            )?;
            let uniforms = get_uniform_locations(
                &self.context,
                &program,
                vec![
                    "fov".to_string(),
                    "tex".to_string(),
                    "rotation_x".to_string(),
                    "rotation_y".to_string(),
                ],
            )?;
            self.context.use_program(Some(&program));

            self.context
                .active_texture(WebGl2RenderingContext::TEXTURE0);
            self.context
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(brush_texture));
            self.context.uniform1f(Some(&uniforms["fov"]), fov);
            self.context.uniform1i(Some(&uniforms["tex"]), 0);
            self.context.uniform1f(Some(&uniforms["rotation_x"]), 0.0);
            self.context.uniform1f(Some(&uniforms["rotation_y"]), 0.0);

            self.context.enable(WebGl2RenderingContext::BLEND);
            self.context.blend_func(
                WebGl2RenderingContext::SRC_ALPHA,
                WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
            );
            self.context
                .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
            self.context
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
        }
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
