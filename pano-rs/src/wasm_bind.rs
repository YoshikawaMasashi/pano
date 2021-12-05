use std::sync::{Arc, RwLock};

use js_sys::ArrayBuffer;
use wasm_bindgen::prelude::*;

use js_sys::Uint8Array;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    pub type Buffer;
}

#[wasm_bindgen(raw_module = "../fs.js")]
extern "C" {
    #[wasm_bindgen(js_name = readFileSync, catch)]
    pub fn read_file(path: &str) -> Result<Buffer, JsValue>;

    #[wasm_bindgen(method, getter)]
    pub fn buffer(this: &Buffer) -> ArrayBuffer;

    #[wasm_bindgen(method, getter, js_name = byteOffset)]
    pub fn byte_offset(this: &Buffer) -> u32;

    #[wasm_bindgen(method, getter)]
    pub fn length(this: &Buffer) -> u32;

    #[wasm_bindgen(js_name = writeFileSync, catch)]
    pub fn write_file(path: &str, data: &Uint8Array) -> Result<(), JsValue>;

    #[wasm_bindgen(js_name = is_directory, catch)]
    pub fn is_directory_(path: &str) -> Result<JsValue, JsValue>;
}

pub fn is_directory(path: &str) -> bool {
    let promise: js_sys::Promise = is_directory_(path).unwrap().into();
    let ret: Arc<RwLock<bool>> = Arc::new(RwLock::new(false));
    let ret_cloned = ret.clone();
    wasm_bindgen_futures::spawn_local(async move {
        let ret_or_undefined = wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
        if let Some(ret_) = ret_or_undefined.as_bool() {
            *ret_cloned.write().unwrap() = ret_;
        } else {
            *ret_cloned.write().unwrap() = false;
        }
        crate::console_log!("ret_cloned {:?}", ret_cloned);
    });
    crate::console_log!("ret {:?}", ret);
    return *ret.clone().read().unwrap();
}

#[wasm_bindgen(raw_module = "../dialog.js")]
extern "C" {
    #[wasm_bindgen(js_name = showOpenDirectoryDialog, catch)]
    pub fn show_open_directory_dialog() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = showOpenPngDialog, catch)]
    pub fn show_open_png_dialog() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = showSavePngDialog, catch)]
    pub fn show_save_png_dialog() -> Result<JsValue, JsValue>;
}

#[wasm_bindgen(raw_module = "../electron_on.js")]
extern "C" {
    #[wasm_bindgen(js_name = set_on_click_export_png)]
    pub fn set_on_click_export_png(func: &js_sys::Function);
    #[wasm_bindgen(js_name = set_on_click_import_png)]
    pub fn set_on_click_import_png(func: &js_sys::Function);
}

#[macro_export]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (crate::wasm_bind::log(&format_args!($($t)*).to_string()))
}
