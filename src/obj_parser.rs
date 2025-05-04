use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    sync::Arc,
};

use crate::{
    shapes::{group::Group, smooth_triangle::SmoothTriangle, triangle::Triangle, Shape},
    vec4::Vec4,
};

struct FaceIndex {
    vertex_index: usize,
    normal_index: Option<usize>
}

pub struct Parser {
    pub vertices: Vec<Vec4>,
    pub groups: HashMap<String, Group>,
    pub current_group: Option<String>,
    pub normals: Vec<Vec4>
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
            normals: Vec::new()
        }
    }

    pub fn parse_file(&mut self, file_path: &str) -> Group {
        self.vertices.push(Vec4::point(0.0, 0.0, 0.0));
        self.normals.push(Vec4::vector(0.0, 0.0, 0.0));

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
            } else if line.starts_with("vn ") {
                let mut parts = line.split_whitespace().into_iter();
                let _prefix = parts.next().unwrap();
                let x = (parts.next().unwrap()).parse::<f64>().unwrap();
                let y = (parts.next().unwrap()).parse::<f64>().unwrap();
                let z = (parts.next().unwrap()).parse::<f64>().unwrap();
                self.normals.push(Vec4::vector(x, y, z));
            } else if line.starts_with("g ") {
                let name = line[2..].trim().to_string();
                self.groups.entry(name.clone()).or_insert(Group::new());
                self.current_group = Some(name);
            } else if line.starts_with("f ") {
                let parts = line.split_whitespace().skip(1);
                let mut face_entry: Vec<FaceIndex> = Vec::new();
            
                for part in parts {
                    let tokens: Vec<&str> = part.split('/').collect();
                    let v = tokens[0].parse::<usize>().unwrap();
                    let vn = if tokens.len() == 3 && !tokens[2].is_empty() {
                        Some(tokens[2].parse::<usize>().unwrap())
                    } else if tokens.len() == 2 && !tokens[1].is_empty() {
                        // Handles `f v//n` form
                        Some(tokens[1].parse::<usize>().unwrap())
                    } else {
                        None
                    };
                    face_entry.push(FaceIndex {
                        vertex_index: v,
                        normal_index: vn,
                    });
                }

                for triangle in self.fan_triangulation(face_entry) {
                    let group = if let Some(ref name) = self.current_group {
                        self.groups.get_mut(name).unwrap()
                    } else {
                        self.groups.get_mut("default").unwrap()
                    };
                    group.add_child(triangle);
                }
            }
        }
        let mut top_group = Group::new();
        for (_name, group) in &mut self.groups.drain() {
            top_group.add_child(Arc::new(group));
        }
        top_group
    }

    fn fan_triangulation(&self, indices: Vec<FaceIndex>) -> Vec<Arc<dyn Shape>> {
        let mut triangles: Vec<Arc<dyn Shape>> = Vec::new();
        let p1 = self.vertices[indices[0].vertex_index];
        let n1 = indices[0].normal_index.map(|i| self.normals[i]);
    
        for i in 1..(indices.len() - 1) {
            let p2 = self.vertices[indices[i].vertex_index];
            let p3 = self.vertices[indices[i + 1].vertex_index];
    
            let n2 = indices[i].normal_index.map(|i| self.normals[i]);
            let n3 = indices[i + 1].normal_index.map(|i| self.normals[i]);
    
            let tri: Arc<dyn Shape> = match (n1, n2, n3) {
                (Some(n1), Some(n2), Some(n3)) => Arc::new(SmoothTriangle::new(Triangle::new(p1, p2, p3), n1, n2, n3)),
                //(Some(n1), Some(n2), Some(n3)) => Arc::new(Triangle::new(p1, p2, p3)),
                _ => Arc::new(Triangle::new(p1, p2, p3)),
            };
            triangles.push(tri);
        }
    
        triangles
    }
}

#[cfg(test)]
pub mod tests {

    use crate::{shapes::{group::Group, smooth_triangle::SmoothTriangle, triangle::Triangle}, vec4::Vec4};

    use super::Parser;


    #[test]
    fn triangulation() {
        let mut p = Parser::new();
        
        let g = p.parse_file("objects/test_files/obj1.obj");
        
        let g_internal = g.children[0].as_ref().as_any().downcast_ref::<Group>().unwrap();
        let t1 = g_internal.children[0].as_any().downcast_ref::<Triangle>().unwrap();
        let t2 = g_internal.children[1].as_any().downcast_ref::<Triangle>().unwrap();
        let t3 = g_internal.children[2].as_any().downcast_ref::<Triangle>().unwrap();

        assert_eq!(t1.p1, p.vertices[1]);
        assert_eq!(t1.p2, p.vertices[2]);
        assert_eq!(t1.p3, p.vertices[3]);

        assert_eq!(t2.p1, p.vertices[1]);
        assert_eq!(t2.p2, p.vertices[3]);
        assert_eq!(t2.p3, p.vertices[4]);

        assert_eq!(t3.p1, p.vertices[1]);
        assert_eq!(t3.p2, p.vertices[4]);
        assert_eq!(t3.p3, p.vertices[5]);
    }
    #[test]
    fn normals() {
        let mut p = Parser::new();
        let g = p.parse_file("objects/test_files/obj2.obj");
        assert_eq!(p.normals[1], Vec4::vector(0.0, 0.0, 1.0));
        assert_eq!(p.normals[2], Vec4::vector(0.707, 0.0, -0.707));
        assert_eq!(p.normals[3], Vec4::vector(1.0, 2.0, 3.0));
    }

    #[test]
    fn normals2() {
        let mut p = Parser::new();
        let g = p.parse_file("objects/test_files/obj3.obj");

        let g_inner = g.children[0].as_any().downcast_ref::<Group>().unwrap();
        let t1 = g_inner.children[0].as_any().downcast_ref::<SmoothTriangle>().unwrap();
        let t2 = g_inner.children[1].as_any().downcast_ref::<SmoothTriangle>().unwrap();

        println!("{:?}",t1.n1);
        println!("{:?}",t1.n2);
        println!("{:?}",t1.n3);

        assert_eq!(t1.triangle.p1, p.vertices[1]);
        assert_eq!(t1.triangle.p2, p.vertices[2]);
        assert_eq!(t1.triangle.p3, p.vertices[3]);
        assert_eq!(t1.n1, p.normals[3]);
        assert_eq!(t1.n2, p.normals[1]);
        assert_eq!(t1.n3, p.normals[2]);

        assert_eq!(t2.triangle.p1, p.vertices[1]);
        assert_eq!(t2.triangle.p2, p.vertices[2]);
        assert_eq!(t2.triangle.p3, p.vertices[3]);
        assert_eq!(t2.n1, p.normals[3]);
        assert_eq!(t2.n2, p.normals[1]);
        assert_eq!(t2.n3, p.normals[2]);
    }
}
