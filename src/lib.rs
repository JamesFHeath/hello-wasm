use cgmath::{Matrix4, PerspectiveFov, Rad, Vector3};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlBuffer, WebGlProgram, WebGlRenderingContext, WebGlShader};

const CANVAS_HEIGHT: u32 = 600;
const CANVAS_WIDTH: u32 = 600;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    // let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    canvas.set_height(CANVAS_HEIGHT);
    canvas.set_width(CANVAS_WIDTH);

    let context = canvas
        .get_context("webgl")
        .unwrap()
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()
        .unwrap();

    // Set clear color to black, fully opaque
    context.clear_color(0.0, 0.0, 0.0, 1.0);
    // Clear the color buffer with specified clear color
    context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    let vs_source = r#"
        attribute vec4 aVertexPosition;

        uniform mat4 uModelViewMatrix;
        uniform mat4 uProjectionMatrix;

        void main() {
        gl_Position = uProjectionMatrix * uModelViewMatrix * aVertexPosition;
        }
    "#;
    let fs_source = r#"
        void main() {
            gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
        }
    "#;

    let shader_program = init_shader_program(&context, vs_source, fs_source)?;
    context.use_program(Some(&shader_program));

    let buffers = init_buffers(&context)?;

    if draw_scene(&context, buffers, &shader_program).is_ok() {
        Ok(())
    } else {
        Err(JsValue::from_str("Error"))
    }
}

pub fn load_shader(
    context: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
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

pub fn init_shader_program(
    context: &WebGlRenderingContext,
    vert_shader: &str,
    frag_shader: &str,
) -> Result<WebGlProgram, String> {
    let vertex_shader = load_shader(&context, WebGlRenderingContext::VERTEX_SHADER, &vert_shader)?;
    let fragment_shader = load_shader(
        &context,
        WebGlRenderingContext::FRAGMENT_SHADER,
        &frag_shader,
    )?;

    let shader_program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&shader_program, &vertex_shader);
    context.attach_shader(&shader_program, &fragment_shader);
    context.link_program(&shader_program);

    if context
        .get_program_parameter(&shader_program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader_program)
    } else {
        Err(context
            .get_program_info_log(&shader_program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

pub fn init_buffers(context: &WebGlRenderingContext) -> Result<(WebGlBuffer, WebGlBuffer), String> {
    let position_buffer = context.create_buffer();
    if position_buffer.is_none() {
        return Err(String::from("Error creating color buffer"));
    }
    context.bind_buffer(
        WebGlRenderingContext::ARRAY_BUFFER,
        position_buffer.as_ref(),
    );
    let positions: [f32; 8] = [-1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0];
    unsafe {
        let vert_array = js_sys::Float32Array::view(&positions);

        context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &vert_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
    }

    let colors: [f32; 16] = [
        1.0, 1.0, 1.0, 1.0, // white
        1.0, 0.0, 0.0, 1.0, // red
        0.0, 1.0, 0.0, 1.0, // green
        0.0, 0.0, 1.0, 1.0, // blue
    ];

    let color_buffer = context.create_buffer();
    if color_buffer.is_none() {
        return Err(String::from("Error creating position buffer"));
    }
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, color_buffer.as_ref());
    unsafe {
        let vert_array = js_sys::Float32Array::view(&colors);

        context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &vert_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
    }

    return Ok((position_buffer.unwrap(), color_buffer.unwrap()));
}

pub fn draw_scene(
    context: &WebGlRenderingContext,
    buffers: (WebGlBuffer, WebGlBuffer),
    shader_program: &WebGlProgram,
) -> Result<(), String> {
    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear_depth(1.0);
    context.enable(WebGlRenderingContext::DEPTH_TEST);
    context.depth_func(WebGlRenderingContext::LEQUAL);

    // context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
    context.clear(WebGlRenderingContext::DEPTH_BUFFER_BIT);

    const FIELD_OF_VIEW: Rad<f32> = Rad {
        0: 45.0 * std::f32::consts::PI / 180.0,
    };
    const ASPECT: f32 = (CANVAS_HEIGHT / CANVAS_WIDTH) as f32;
    const Z_NEAR: f32 = 0.1;
    const Z_FAR: f32 = 100.0;

    let perspective_fov: PerspectiveFov<f32> = PerspectiveFov {
        fovy: FIELD_OF_VIEW,
        aspect: ASPECT,
        near: Z_NEAR,
        far: Z_FAR,
    };

    let mut model_view_matrix: Matrix4<f32> = Matrix4::from_translation(Vector3 {
        x: -0.0,
        y: 0.0,
        z: -6.0,
    });

    const NUM_COMPONENTS: i32 = 2;
    const VAP_TYPE: u32 = WebGlRenderingContext::FLOAT;
    const NORMALIZE: bool = false;
    const STRIDE: i32 = 0;
    const OFFSET: i32 = 0;

    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(buffers.0).as_ref());
    let vertex_position = context.get_attrib_location(shader_program, "aVertexPosition") as u32;
    context.vertex_attrib_pointer_with_i32(
        vertex_position,
        NUM_COMPONENTS,
        VAP_TYPE,
        NORMALIZE,
        STRIDE,
        OFFSET,
    );
    context.enable_vertex_attrib_array(vertex_position);

    context.use_program(Some(shader_program));

    let program_projection_matrix =
        context.get_uniform_location(shader_program, "uProjectionMatrix");
    let mut projection_matrix = Matrix4::from(perspective_fov);
    let projection_matrix_slice: &[f32; 16] = &projection_matrix.as_mut();
    context.uniform_matrix4fv_with_f32_array(
        program_projection_matrix.as_ref(),
        false,
        projection_matrix_slice,
    );

    let program_model_view_matrix =
        context.get_uniform_location(shader_program, "uModelViewMatrix");
    let model_view_matrix_slice: &[f32; 16] = &model_view_matrix.as_mut();
    context.uniform_matrix4fv_with_f32_array(
        program_model_view_matrix.as_ref(),
        false,
        model_view_matrix_slice,
    );

    const VERTEX_COUNT: i32 = 4;
    context.draw_arrays(WebGlRenderingContext::TRIANGLE_STRIP, OFFSET, VERTEX_COUNT);

    Ok(())
}
