// See LICENSE file for copyright and license details.

use std;
use glfw;
use gl;
use gl::types::{
    GLint,
    GLuint,
    GLchar,
    GLenum,
    GLsizeiptr,
    GLsizei,
};
use cgmath::matrix::{
    Matrix,
    Mat4,
    Mat3,
    ToMat4,
};
use cgmath::vector::{
    Vec3,
};
use cgmath::angle;
use stb_image::image;
use misc::{
    deg_to_rad,
};
use gl_types::{
    Float,
    Color3,
    TextureCoord,
    VertexCoord,
    ShaderId,
    AttrId,
    VboId,
    TextureId,
    MatId,
};
use core_types::{
    Size2,
    Int,
};

fn c_str(s: &str) -> *GLchar {
    unsafe {
        s.to_c_str().unwrap()
    }
}

pub fn compile_shader(src: &str, shader_type: GLenum) -> GLuint {
    let shader = gl::CreateShader(shader_type);
    unsafe {
        gl::ShaderSource(shader, 1, &c_str(src), std::ptr::null());
        gl::CompileShader(shader);
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            // subtract 1 to skip the trailing null character
            let mut buf = std::vec::from_elem(len as uint - 1, 0u8);
            gl::GetShaderInfoLog(shader, len, std::ptr::mut_null(),
                buf.as_mut_ptr() as *mut GLchar
            );
            fail!("compile_shader(): " + std::str::raw::from_utf8(buf));
        }
    }
    shader
}

pub fn link_program(vertex_shader: GLuint, fragment_shader: GLuint) -> GLuint {
    let program = gl::CreateProgram();
    gl::AttachShader(program, vertex_shader);
    gl::AttachShader(program, fragment_shader);
    gl::LinkProgram(program);
    unsafe {
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            // subtract 1 to skip the trailing null character
            let mut buf = std::vec::from_elem(len as uint - 1, 0u8);
            gl::GetProgramInfoLog(program, len, std::ptr::mut_null(),
                buf.as_mut_ptr() as *mut GLchar
            );
            fail!("link_program(): " + std::str::raw::from_utf8(buf));
        }
    }
    program
}

pub fn compile_program(
    vertex_shader_src: &str,
    frag_shader_src: &str,
) -> ShaderId {
    let vertex_shader = compile_shader(
        vertex_shader_src, gl::VERTEX_SHADER);
    let fragment_shader = compile_shader(
        frag_shader_src, gl::FRAGMENT_SHADER);
    let program = link_program(vertex_shader, fragment_shader);
    // mark shaders for deletion after program deletion
    gl::DeleteShader(fragment_shader);
    gl::DeleteShader(vertex_shader);
    ShaderId(program)
}

pub fn get_attr(shader: &ShaderId, name: &str) -> AttrId {
    unsafe {
        let ShaderId(id) = *shader;
        let attr_id = gl::GetAttribLocation(id, c_str(name));
        AttrId(attr_id as GLuint)
    }
}

pub fn get_uniform(shader: &ShaderId, name: &str) -> GLuint {
    let ShaderId(id) = *shader;
    unsafe {
        gl::GetUniformLocation(id, c_str(name)) as GLuint
    }
}

pub fn draw_mesh(faces_count: Int) {
    let starting_index = 0;
    let vertices_count = faces_count * 3;
    gl::DrawArrays(gl::TRIANGLES, starting_index, vertices_count);
}

pub fn uniform_mat4f(mat_id: MatId, mat: &Mat4<Float>) {
    unsafe {
        let MatId(id) = mat_id;
        gl::UniformMatrix4fv(id as Int, 1, gl::FALSE, mat.cr(0, 0));
    }
}

pub fn tr(m: Mat4<Float>, v: Vec3<Float>) -> Mat4<Float> {
    let mut t = Mat4::<Float>::identity();
    *t.mut_cr(3, 0) = v.x;
    *t.mut_cr(3, 1) = v.y;
    *t.mut_cr(3, 2) = v.z;
    m.mul_m(&t)
}

pub fn rot_x(m: Mat4<Float>, angle: Float) -> Mat4<Float> {
    let rad = angle::rad(deg_to_rad(angle));
    let r = Mat3::from_angle_x(rad).to_mat4();
    m.mul_m(&r)
}

pub fn rot_z(m: Mat4<Float>, angle: Float) -> Mat4<Float> {
    let rad = angle::rad(deg_to_rad(angle));
    let r = Mat3::from_angle_z(rad).to_mat4();
    m.mul_m(&r)
}

pub fn gen_buffer() -> VboId {
    let mut n = 0;
    unsafe {
        gl::GenBuffers(1, &mut n);
    }
    VboId(n)
}

pub fn delete_buffer(buffer: &VboId) {
    unsafe {
        let VboId(id) = *buffer;
        gl::DeleteBuffers(1, &id);
    }
}

fn fill_buffer<T>(buffer_size: i64, data: &[T]) {
    unsafe {
        let data_ptr = std::cast::transmute(&data[0]);
        gl::BufferData(
            gl::ARRAY_BUFFER, buffer_size, data_ptr, gl::STATIC_DRAW);
    }
}

pub fn fill_current_coord_vbo(data: &[VertexCoord]) {
    let size = std::mem::size_of::<VertexCoord>();
    let buffer_size = (data.len() * size) as GLsizeiptr;
    fill_buffer(buffer_size, data);
}

pub fn fill_current_color_vbo(data: &[Color3]) {
    let size = std::mem::size_of::<Color3>();
    let buffer_size = (data.len() * size) as GLsizeiptr;
    fill_buffer(buffer_size, data);
}

pub fn fill_current_texture_coords_vbo(data: &[TextureCoord]) {
    let size = std::mem::size_of::<TextureCoord>();
    let buffer_size = (data.len() * size) as GLsizeiptr;
    fill_buffer(buffer_size, data);
}

pub fn vertex_attrib_pointer(attr: AttrId, components_count: Int) {
    let normalized = gl::FALSE;
    let stride = 0;
    let AttrId(id) = attr;
    unsafe {
        gl::VertexAttribPointer(
            id,
            components_count,
            gl::FLOAT,
            normalized,
            stride,
            std::ptr::null(),
        );
    }
}

// TODO: shader method
pub fn use_program(shader: &ShaderId) {
    let ShaderId(shader_id) = *shader;
    gl::UseProgram(shader_id);
}

pub fn enable_vertex_attrib_array(attr: &AttrId) {
    let AttrId(id) = *attr;
    gl::EnableVertexAttribArray(id);
}

pub fn init_opengl() {
    // TODO: Remove 'use glfw, glfw::...'?
    gl::load_with(glfw::get_proc_address);
    gl::Enable(gl::DEPTH_TEST);
}

// TODO: Drop
pub fn delete_program(shader: &ShaderId) {
    let ShaderId(id) = *shader;
    gl::DeleteProgram(id);
}

pub fn set_clear_color(r: Float, g: Float, b: Float) {
    gl::ClearColor(r, g, b, 1.0);
}

pub fn clear() {
    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
}

pub fn viewport(size: Size2<Int>) {
    gl::Viewport(0, 0, size.w, size.h);
}

pub fn bind_buffer(buffer: &VboId) {
    let VboId(id) = *buffer;
    gl::BindBuffer(gl::ARRAY_BUFFER, id);
}

pub fn enable_texture(shader: &ShaderId, texture: &TextureId) {
    let TextureId(id) = *texture;
    let basic_texture_loc = get_uniform(shader, "basic_texture") as GLint;
    gl::Uniform1ui(basic_texture_loc, 0);
    gl::ActiveTexture(gl::TEXTURE0);
    gl::BindTexture(gl::TEXTURE_2D, id);
}

fn load_image(path: ~str) -> image::Image<u8> {
    let load_result = image::load(path);
    match load_result {
        image::ImageU8(image) => {
            image
        },
        image::Error(message) => {
            fail!("{}", message);
        },
        _ => {
            fail!("Unknown image format");
        }
    }
}

pub fn load_texture(path: ~str) -> TextureId {
    let image = load_image(path);
    let mut id = 0;
    unsafe {
        gl::GenTextures(1, &mut id)
    };
    gl::ActiveTexture(gl::TEXTURE0);
    gl::BindTexture(gl::TEXTURE_2D, id);
    let format = match image.depth {
        4 => gl::RGBA,
        3 => gl::RGB,
        _ => fail!("wrong depth"),
    };
    unsafe {
        let level = 0;
        let border = 0;
        gl::TexImage2D(
            gl::TEXTURE_2D,
            level,
            format as GLint,
            image.width as GLsizei,
            image.height as GLsizei,
            border,
            format,
            gl::UNSIGNED_BYTE,
            std::cast::transmute(&image.data[0]),
        );
    }
    gl::TexParameteri(gl::TEXTURE_2D,
        gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
    gl::TexParameteri(gl::TEXTURE_2D,
        gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
    gl::TexParameteri(gl::TEXTURE_2D,
        gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
    gl::TexParameteri(gl::TEXTURE_2D,
        gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
    TextureId(id)
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
