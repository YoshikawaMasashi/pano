use std::io::Cursor;
use std::path::Path;

use js_sys::Uint8Array;

use crate::wasm_bind::{read_file, write_file};

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

pub fn write_binary(path: &Path, data: Vec<u8>) {
    let array = Uint8Array::new_with_length(data.len() as u32);
    array.copy_from(data.as_slice());
    write_file(path.to_str().unwrap(), &array).unwrap();
}

pub fn read_image(path: &Path) -> image::RgbaImage {
    image::load(
        Cursor::new(read_binary(path).as_slice()),
        image::ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8()
}

pub fn write_image(path: &Path, data: image::RgbaImage) {
    let data = image::DynamicImage::ImageRgba8(data);
    let mut bytes: Vec<u8> = Vec::new();
    data.write_to(&mut bytes, image::ImageOutputFormat::Png)
        .unwrap();

    write_binary(path, bytes);
}
