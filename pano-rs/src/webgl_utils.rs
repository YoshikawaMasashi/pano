extern crate console_error_panic_hook;

use std::collections::HashMap;
use std::path::Path;

use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

use crate::file_io::read_binary;

pub fn read_shader(
    path: &Path,
    context: &WebGl2RenderingContext,
    shader_type: u32,
) -> Result<WebGlShader, String> {
    compile_shader(
        context,
        shader_type,
        std::str::from_utf8(read_binary(path).as_slice()).unwrap(),
    )
}

pub fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    context: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

pub fn get_uniform_locations(
    context: &WebGl2RenderingContext,
    program: &WebGlProgram,
    keys: Vec<String>,
) -> Result<HashMap<String, WebGlUniformLocation>, String> {
    let mut locations: HashMap<String, WebGlUniformLocation> = HashMap::new();
    for key in keys {
        locations.insert(
            key.clone(),
            context.get_uniform_location(program, &key).unwrap(),
        );
    }
    Ok(locations)
}