use std::io::Cursor;
use std::path::Path;

use js_sys::Uint8Array;

use crate::wasm_bind::read_file;

pub fn read_binary(path: &Path) -> Vec<u8> {
    let buffer = read_file(path.to_str().unwrap()).unwrap();
    let buffer: Vec<u8> = Uint8Array::new_with_byte_offset_and_length(
        &buffer.buffer(),
        buffer.byte_offset(),
        buffer.length(),
    )
    .to_vec();
    buffer
}

pub fn read_image(path: &Path) -> image::RgbaImage {
    image::load(
        Cursor::new(read_binary(path).as_slice()),
        image::ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8()
}
