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

pub enum Msg {}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub open: bool,
}

pub struct ImageTransferDialog {
    #[allow(dead_code)]
    link: ComponentLink<Self>,
    webgl: Option<Arc<RwLock<ModelWebGL>>>,

    open: bool,
}

pub struct ModelWebGL {}

impl Component for ImageTransferDialog {
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
        if first_render {}
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {}
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
                    id="image transfer dialog"
                    open=self.open
                >
                    {"Image Transfer"}
                </dialog>
            </div>
        }
    }
}
