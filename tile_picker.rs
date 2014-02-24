// See LICENSE file for copyright and license details.

use std;
use cgmath::vector::{
    Vec3,
    Vec2,
};
use glh = gl_helpers;
use map::MapPosIter;
use camera::Camera;
use geom::Geom;
use mesh::Mesh;
use misc::read_file;
use core_types::{
    Int,
    Size2,
    MapPos,
};
use gl_types::{
    VertexCoord,
    Color3,
    Float,
    ShaderId,
    MatId,
};

fn build_hex_map_mesh(
    geom: &Geom,
    map_size: Size2<Int>
) -> (~[VertexCoord], ~[Color3]) {
    let mut c_data = ~[];
    let mut v_data = ~[];
    for tile_pos in MapPosIter::new(map_size) {
        let pos3d = geom.map_pos_to_world_pos(tile_pos);
        for num in range(0 as Int, 6) {
            let vertex = geom.index_to_hex_vertex(num);
            let next_vertex = geom.index_to_hex_vertex(num + 1);
            let col_x = tile_pos.x as Float / 255.0;
            let col_y = tile_pos.y as Float / 255.0;
            let color = Color3{r: col_x, g: col_y, b: 1.0};
            v_data.push(pos3d + vertex);
            c_data.push(color);
            v_data.push(pos3d + next_vertex);
            c_data.push(color);
            v_data.push(pos3d + Vec3::zero());
            c_data.push(color);
        }
    }
    (v_data, c_data)
}

pub struct TilePicker {
    program: ShaderId,
    map_mesh: Mesh,
    mat_id: MatId,
    win_size: Size2<Int>,
}

impl TilePicker {
    pub fn new(win_size: Size2<Int>) -> TilePicker {
        let picker = TilePicker {
            program: ShaderId(0),
            map_mesh: Mesh::new(),
            mat_id: MatId(0),
            win_size: win_size,
        };
        picker
    }

    pub fn set_win_size(&mut self, win_size: Size2<Int>) {
        self.win_size = win_size;
    }

    pub fn cleanup_opengl(&self) {
        glh::delete_program(&self.program);
    }

    pub fn init(&mut self, geom: &Geom, map_size: Size2<Int>) {
        self.program = glh::compile_program(
            read_file(&Path::new("pick.vs.glsl")),
            read_file(&Path::new("pick.fs.glsl")),
        );
        glh::use_program(&self.program);
        let position_attr = glh::get_attr(
            &self.program, "in_vertex_coordinates");
        let color_attr = glh::get_attr(&self.program, "color");
        glh::enable_vertex_attrib_array(&position_attr);
        glh::enable_vertex_attrib_array(&color_attr);
        glh::vertex_attrib_pointer(position_attr, 3);
        glh::vertex_attrib_pointer(color_attr, 3);
        let (vertex_data, color_data) = build_hex_map_mesh(geom, map_size);
        self.map_mesh.set_vertex_coords(vertex_data);
        self.map_mesh.set_color(color_data);
        self.mat_id = MatId(glh::get_uniform(&self.program, "mvp_mat"));
    }

    fn read_coords_from_image_buffer(
        &self,
        mouse_pos: Vec2<Int>
    ) -> Option<MapPos> {
        use gl; // TODO: remove
        let height = self.win_size.h;
        let reverted_y = height - mouse_pos.y;
        let data: [u8, ..4] = [0, 0, 0, 0]; // mut
        unsafe {
            let data_ptr = std::cast::transmute(&data[0]);
            gl::ReadPixels(
                mouse_pos.x, reverted_y, 1, 1,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data_ptr
            );
        }
        if data[2] != 0 {
            Some(Vec2{x: data[0] as Int, y: data[1] as Int})
        } else {
            None
        }
    }

    pub fn pick_tile(
        &mut self,
        camera: &Camera,
        mouse_pos: Vec2<Int>
    ) -> Option<MapPos> {
        glh::use_program(&self.program);
        glh::uniform_mat4f(self.mat_id, &camera.mat());
        glh::set_clear_color(0.0, 0.0, 0.0);
        glh::clear();
        self.map_mesh.draw(&self.program);
        self.read_coords_from_image_buffer(mouse_pos)
    }
}

impl Drop for TilePicker {
    fn drop(&mut self) {
        self.cleanup_opengl();
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
