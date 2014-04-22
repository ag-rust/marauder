// See LICENSE file for copyright and license details.

use std::str::Words;
use std::str::CharSplits;
use std::from_str::FromStr;
use std::io::{BufferedReader, File};
use cgmath::vector::{Vector3, Vector2};
use core::types::{MBool, MInt};
use visualizer::types::{VertexCoord, TextureCoord, Normal};

struct Face {
    vertex: [MInt, ..3],
    texture: [MInt, ..3],
    normal: [MInt, ..3],
}

pub struct Model {
    coords: Vec<VertexCoord>,
    normals: Vec<Normal>,
    texture_coords: Vec<TextureCoord>,
    faces: Vec<Face>,
}

fn parse_word<T: FromStr>(words: &mut Words) -> T {
    let str = words.next().expect("Can not read next word");
    from_str(str).expect("Can not convert from string")
}

fn parse_charsplit<T: FromStr>(words: &mut CharSplits<char>) -> T {
    let str = words.next().expect("Can not read next word");
    from_str(str).expect("Can not convert from string")
}

impl Model {
    pub fn new(path: &Path) -> Model {
        let mut obj = Model {
            coords: Vec::new(),
            normals: Vec::new(),
            texture_coords: Vec::new(),
            faces: Vec::new(),
        };
        obj.read(path);
        obj
    }

    fn read_v_or_vn(words: &mut Words) -> VertexCoord {
        Vector3 {
            x: parse_word(words),
            y: parse_word(words),
            z: parse_word(words),
        }
    }

    fn read_vt(words: &mut Words) -> TextureCoord {
        Vector2 {
            x: parse_word(words),
            y: 1.0 - parse_word(words), // flip
        }
    }

    fn read_f(words: &mut Words) -> Face {
        let mut face = Face {
            vertex: [0, 0, 0],
            texture: [0, 0, 0],
            normal: [0, 0, 0],
        };
        let mut i = 0;
        for group in *words {
            let mut w = group.split('/');
            face.vertex[i] = parse_charsplit(&mut w);
            face.texture[i] = parse_charsplit(&mut w);
            face.normal[i] = parse_charsplit(&mut w);
            i += 1;
        }
        face
    }

    fn read_line(&mut self, line: &str) {
        let mut words = line.words();
        fn is_correct_tag(tag: &str) -> MBool {
            tag.len() != 0 && tag[0] != ('#' as u8)
        }
        match words.next() {
            Some(tag) if is_correct_tag(tag) => {
                let w = &mut words;
                match tag {
                    &"v" => self.coords.push(Model::read_v_or_vn(w)),
                    &"vn" => self.normals.push(Model::read_v_or_vn(w)),
                    &"vt" => self.texture_coords.push(Model::read_vt(w)),
                    &"f" => self.faces.push(Model::read_f(w)),
                    _ => {},
                }
            }
            _ => {},
        };
    }

    fn read(&mut self, path: &Path) {
        let mut file = BufferedReader::new(File::open(path));
        for line in file.lines() {
            match line {
                Ok(line) => self.read_line(line),
                Err(msg) => fail!("Obj: read error: {}", msg),
            }
        }
    }

    pub fn build(&self) -> Vec<VertexCoord> {
        let mut mesh = Vec::new();
        for face in self.faces.iter() {
            for i in range(0, 3) {
                let vertex_id = face.vertex[i as uint] - 1;
                mesh.push(*self.coords.get(vertex_id as uint));
            }
        }
        mesh
    }

    pub fn build_tex_coord(&self) -> Vec<TextureCoord> {
        let mut tex_coords = Vec::new();
        for face in self.faces.iter() {
            for i in range(0, 3) {
                let texture_coord_id = face.texture[i as uint] as uint - 1;
                tex_coords.push(*self.texture_coords.get(texture_coord_id));
            }
        }
        tex_coords
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
