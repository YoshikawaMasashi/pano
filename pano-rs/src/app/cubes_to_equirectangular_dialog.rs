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

pub enum Msg {}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub open: bool,
}

pub struct CubesToEquirectangularDialog {
    #[allow(dead_code)]
    link: ComponentLink<Self>,
    webgl: Option<Arc<RwLock<ModelWebGL>>>,

    open: bool,
}

pub struct ModelWebGL {
    context: WebGl2RenderingContext,
    cubes_to_equirectangular_vert_shader: WebGlShader,
    cubes_to_equirectangular_frag_shader: WebGlShader,
}

impl Component for CubesToEquirectangularDialog {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            webgl: None,
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

            let cubes_to_equirectangular_vert_shader = compile_shader(
                &context,
                WebGl2RenderingContext::VERTEX_SHADER,
                include_str!("../shaders/6cubes_to_equirectangular.vert"),
            )
            .unwrap();
            let cubes_to_equirectangular_frag_shader = compile_shader(
                &context,
                WebGl2RenderingContext::FRAGMENT_SHADER,
                include_str!("../shaders/6cubes_to_equirectangular.frag"),
            )
            .unwrap();

            self.webgl = Some(Arc::new(RwLock::new(ModelWebGL {
                context,
                cubes_to_equirectangular_vert_shader,
                cubes_to_equirectangular_frag_shader,
            })));

            self.webgl
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .convert(Path::new(
                    "C:/Users/earne/Desktop/pano_project/pano/pano-rs/examples/6cubes_image",
                ))
                .unwrap();
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
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
                    id="6cubes to equirectangular dialog"
                    open=self.open
                >
                    {"6 cubes to equirectangular"}
                    <br />
                    {"6cubes images: front.png, back.png, left.png, right.png, top.png, botton.pngが入ったディレクトリを指定してください"}
                    <br />
                    <input/>
                    <button>{ "ファイルを選択" }</button>
                    <canvas
                        id="6cubes_canvas"
                        height="1"
                        width="1"
                    ></canvas>
                </dialog>
            </div>
        }
    }
}

impl ModelWebGL {
    pub fn convert(&self, path: &Path) -> Result<(), JsValue> {
        let front_image = read_image(Path::new(path.join("front.png").as_path()));
        let back_image = read_image(Path::new(path.join("back.png").as_path()));
        let left_image = read_image(Path::new(path.join("left.png").as_path()));
        let right_image = read_image(Path::new(path.join("right.png").as_path()));
        let top_image = read_image(Path::new(path.join("top.png").as_path()));
        let bottom_image = read_image(Path::new(path.join("bottom.png").as_path()));

        let front_texture = self.context.create_texture().unwrap();
        let back_texture = self.context.create_texture().unwrap();
        let left_texture = self.context.create_texture().unwrap();
        let right_texture = self.context.create_texture().unwrap();
        let top_texture = self.context.create_texture().unwrap();
        let bottom_texture = self.context.create_texture().unwrap();

        for (texture, image) in [
            (&front_texture, &front_image),
            (&back_texture, &back_image),
            (&left_texture, &left_image),
            (&right_texture, &right_image),
            (&top_texture, &top_image),
            (&bottom_texture, &bottom_image),
        ] {
            self.context
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
            self.context
                .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                    WebGl2RenderingContext::TEXTURE_2D,
                    0,
                    WebGl2RenderingContext::RGBA as i32,
                    WORK_TEXTURE_WIDTH as i32,
                    WORK_TEXTURE_WIDTH as i32,
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
        }

        let output_texture = self.context.create_texture().unwrap();
        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&output_texture));
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
            &self.cubes_to_equirectangular_vert_shader,
            &self.cubes_to_equirectangular_frag_shader,
        )?;
        let uniforms = get_uniform_locations(
            &self.context,
            &program,
            vec![
                "front".to_string(),
                "back".to_string(),
                "left".to_string(),
                "right".to_string(),
                "top".to_string(),
                "bottom".to_string(),
            ],
        )?;
        self.context.use_program(Some(&program));

        for (texturei, texture, name, idx) in [
            (WebGl2RenderingContext::TEXTURE0, &front_texture, "front", 0),
            (WebGl2RenderingContext::TEXTURE1, &back_texture, "back", 1),
            (WebGl2RenderingContext::TEXTURE2, &left_texture, "left", 2),
            (WebGl2RenderingContext::TEXTURE3, &right_texture, "right", 3),
            (WebGl2RenderingContext::TEXTURE4, &top_texture, "top", 4),
            (
                WebGl2RenderingContext::TEXTURE5,
                &bottom_texture,
                "bottom",
                5,
            ),
        ] {
            self.context.active_texture(texturei);
            self.context
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));
            self.context.uniform1i(Some(&uniforms[name]), idx);
        }
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
        write_image(path.join("equirectangular.png").as_path(), data);
        Ok(())
    }
}
