use std::path::Path;
use std::sync::{Arc, RwLock};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, WebGl2RenderingContext, WebGlShader};
use yew::prelude::*;

use crate::file_io::{read_image, write_image};
use crate::webgl_utils::{compile_shader, get_uniform_locations, link_program};

const WORK_TEXTURE_WIDTH: usize = 3840;
const WORK_TEXTURE_HEIGHT: usize = 1920;

pub enum Msg {
    OpenInputImageDialog,
    OpenOutputImageDialog,
    ExecuteTransfer,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub open: bool,
}

pub struct ImageTransferDialog {
    #[allow(dead_code)]
    link: ComponentLink<Self>,
    webgl: Option<Arc<RwLock<ModelWebGL>>>,
    input_of_input_image_ref: NodeRef,
    input_of_output_image_ref: NodeRef,

    open: bool,
}

pub struct ModelWebGL {
    context: WebGl2RenderingContext,
    image_transfer_vert_shader: WebGlShader,
    image_transfer_frag_shader: WebGlShader,
}

impl Component for ImageTransferDialog {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let input_of_input_image_ref = NodeRef::default();
        let input_of_output_image_ref = NodeRef::default();
        Self {
            link,
            webgl: None,
            input_of_input_image_ref,
            input_of_output_image_ref,

            open: props.open,
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            let document = web_sys::window().unwrap().document().unwrap();
            let canvas = document.get_element_by_id("6cubes_canvas").unwrap();
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

            let image_transfer_vert_shader = compile_shader(
                &context,
                WebGl2RenderingContext::VERTEX_SHADER,
                include_str!("../shaders/image_transfer.vert"),
            )
            .unwrap();
            let image_transfer_frag_shader = compile_shader(
                &context,
                WebGl2RenderingContext::FRAGMENT_SHADER,
                include_str!("../shaders/image_transfer.frag"),
            )
            .unwrap();

            self.webgl = Some(Arc::new(RwLock::new(ModelWebGL {
                context,
                image_transfer_vert_shader,
                image_transfer_frag_shader,
            })));
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::OpenInputImageDialog => {
                let dialog_promise: js_sys::Promise =
                    crate::wasm_bind::show_open_png_dialog().unwrap().into();
                let input_of_input_image_ref = self.input_of_input_image_ref.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let path_or_undefined = wasm_bindgen_futures::JsFuture::from(dialog_promise)
                        .await
                        .unwrap();
                    if let Some(path) = path_or_undefined.as_string() {
                        if let Some(input) = input_of_input_image_ref.cast::<HtmlInputElement>() {
                            input.set_value(path.as_str());
                        }
                    }
                });
                false
            }
            Msg::OpenOutputImageDialog => {
                let dialog_promise: js_sys::Promise =
                    crate::wasm_bind::show_save_png_dialog().unwrap().into();
                let input_of_output_image_ref = self.input_of_output_image_ref.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let path_or_undefined = wasm_bindgen_futures::JsFuture::from(dialog_promise)
                        .await
                        .unwrap();
                    if let Some(path) = path_or_undefined.as_string() {
                        if let Some(input) = input_of_output_image_ref.cast::<HtmlInputElement>() {
                            input.set_value(path.as_str());
                        }
                    }
                });
                false
            }
            Msg::ExecuteTransfer => {
                if let Some(input_of_input_image) =
                    self.input_of_input_image_ref.cast::<HtmlInputElement>()
                {
                    if let Some(input_of_output_image) =
                        self.input_of_output_image_ref.cast::<HtmlInputElement>()
                    {
                        let input_image_path = input_of_input_image.value();
                        let input_image_path = Path::new(&input_image_path);
                        let output_image_path = input_of_output_image.value();
                        let output_image_path = Path::new(&output_image_path);
                        self.webgl
                            .as_ref()
                            .unwrap()
                            .read()
                            .unwrap()
                            .transfer(input_image_path, output_image_path)
                            .unwrap();
                    }
                }
                false
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.open != props.open {
            self.open = props.open;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        html! {
            <div id="centerpoint">
                <dialog
                    id="image transfer dialog"
                    open=self.open
                >
                    {"Image Transfer"}
                    <br />
                    {"Input Image"}
                    <br />
                    <input
                        ref={self.input_of_input_image_ref.clone()}
                    />
                    <button onclick=self.link.callback(|_| Msg::OpenInputImageDialog)>{ "ファイルを選択" }</button>
                    <br />
                    <br />
                    {"Output Image"}
                    <br />
                    <input
                        ref={self.input_of_output_image_ref.clone()}
                    />
                    <button onclick=self.link.callback(|_| Msg::OpenOutputImageDialog)>{ "ファイルを選択" }</button>
                    <br />
                    <button onclick=self.link.callback(|_| Msg::ExecuteTransfer)>{ "実行" }</button>
                </dialog>
            </div>
        }
    }
}

impl ModelWebGL {
    pub fn transfer(
        &self,
        input_image_path: &Path,
        output_image_path: &Path,
    ) -> Result<(), JsValue> {
        let input_image = read_image(input_image_path);

        let input_texture = self.context.create_texture().unwrap();
        let output_texture = self.context.create_texture().unwrap();

        for (texture, pixels) in [
            (&input_texture, Some(input_image.as_raw().as_slice())),
            (&output_texture, None),
        ] {
            self.context
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
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
                    pixels,
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
        }

        let frame_buffer = self.context.create_framebuffer().unwrap();
        self.context
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&frame_buffer));

        self.context.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&output_texture),
            0,
        );
        self.context
            .viewport(0, 0, WORK_TEXTURE_WIDTH as i32, WORK_TEXTURE_HEIGHT as i32);

        let program = link_program(
            &self.context,
            &self.image_transfer_vert_shader,
            &self.image_transfer_frag_shader,
        )?;
        let uniforms = get_uniform_locations(&self.context, &program, vec!["tex".to_string()])?;
        self.context.use_program(Some(&program));

        self.context
            .active_texture(WebGl2RenderingContext::TEXTURE0);
        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&input_texture));
        self.context.uniform1i(Some(&uniforms["tex"]), 0);

        self.context.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );
        self.context
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);

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
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
        self.context
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);

        let data =
            image::RgbaImage::from_vec(WORK_TEXTURE_WIDTH as u32, WORK_TEXTURE_HEIGHT as u32, data)
                .unwrap();
        write_image(output_image_path, data);
        Ok(())
    }
}
