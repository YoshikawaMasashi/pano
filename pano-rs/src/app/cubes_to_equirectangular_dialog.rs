use yew::prelude::*;

pub enum Msg {}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub open: bool,
}

pub struct CubesToEquirectangularDialog {
    #[allow(dead_code)]
    link: ComponentLink<Self>,
    open: bool,
}

impl Component for CubesToEquirectangularDialog {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            open: props.open,
        }
    }

    fn rendered(&mut self, _first_render: bool) {}

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
                </dialog>
            </div>
        }
    }
}
