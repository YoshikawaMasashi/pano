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
}

#[macro_export]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (crate::wasm_bind::log(&format_args!($($t)*).to_string()))
}
