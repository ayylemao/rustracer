use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    sync::Arc,
};

use crate::{
    shapes::{group::Group, triangle::Triangle},
    vec4::Vec4,
};

pub struct Parser {
    pub vertices: Vec<Vec4>,
    pub groups: HashMap<String, Group>,
    pub current_group: Option<String>,
}

impl Parser {
    pub fn new() -> Self {
        let vertices = Vec::new();
        let mut groups: HashMap<String, Group> = HashMap::new();
        groups.entry("default".to_string()).or_insert(Group::new());
        Parser {
            vertices: vertices,
            groups: groups,
            current_group: None,
        }
    }

    pub fn parse_file(&mut self, file_path: &str) -> Group {
        self.vertices.push(Vec4::point(0.0, 0.0, 0.0));

        let file = File::open(file_path).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents).unwrap();

        for line in contents.split('\n') {
            if line.starts_with("v ") {
                let mut parts = line.split_whitespace().into_iter();
                let _prefix = parts.next().unwrap();
                let x = (parts.next().unwrap()).parse::<f64>().unwrap();
                let y = (parts.next().unwrap()).parse::<f64>().unwrap();
                let z = (parts.next().unwrap()).parse::<f64>().unwrap();
                self.vertices.push(Vec4::point(x, y, z));
            } else if line.starts_with("g ") {
                let name = line[2..].trim().to_string();
                self.groups.entry(name.clone()).or_insert(Group::new());
                self.current_group = Some(name);
            } else if line.starts_with("f ") {
                let parts = line.split(" ").into_iter();
                let mut face_entry: Vec<usize> = Vec::new();
                for part in parts.skip(1) {
                    face_entry.push(part.parse::<usize>().unwrap());
                }
                for triangle in self.fan_triangulation(face_entry) {
                    let group = if let Some(ref name) = self.current_group {
                        self.groups.get_mut(name).unwrap()
                    } else {
                        self.groups.get_mut("default").unwrap()
                    };
                    group.add_child(Arc::new(triangle));
                }
            }
        }
        let mut top_group = Group::new();
        for (_name, group) in &mut self.groups.drain() {
            top_group.add_child(Arc::new(group));
        }
        top_group
    }

    fn fan_triangulation(&self, indices: Vec<usize>) -> Vec<Triangle> {
        let mut triangles: Vec<Triangle> = Vec::new();
        let p1 = self.vertices[indices[0]];
        for i in 1..(indices.len() - 1) {
            let p2 = self.vertices[indices[i]];
            let p3 = self.vertices[indices[i + 1]];
            let tri = Triangle::new(p1, p2, p3);
            triangles.push(tri);
        }
        triangles
    }
}
