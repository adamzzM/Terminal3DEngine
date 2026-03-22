use crate::algebra::{Vec3, Vec4, Mat4, Triangle};
use std::{collections::HashMap,io};

use std::fs::File; 
use std::io::BufRead; 
use std::path::Path; 





#[derive(Clone)]
pub struct MatColor{
    pub r: u8,
    pub g: u8,
    pub b: u8
}
impl MatColor{
    pub fn new(r: u8,g: u8,b: u8) -> Self{
        MatColor {
            r,
            g,
            b
        }

    }
}

pub struct Entity {
    color: Option<MatColor>,
    pub transform: Transform,
    pub mesh: Mesh,
}

pub struct Transform{
    pos: Vec3,
    rotation: Vec4,
    scale: Vec3,
    world: Mat4,
    center: Vec3,
    dirty: bool

}

impl Transform{
    pub fn new() -> Self{
        let pos = Vec3::new(0.0,0.0,5.0);
        let rotation = Vec4::new(0.0,0.0,0.0,1.0);
        let scale = Vec3::new(1.0,1.0,1.0);
        let mut pos_mat = Mat4::new();
        let mut rot_mat = Mat4::new();
        let mut scale_mat = Mat4::new();

        scale_mat.scale(&scale);
        rot_mat.rotation(&rotation);
        pos_mat.translation(&pos);
        let world = scale_mat * rot_mat * pos_mat;


        Transform {
             pos: pos , 
             rotation: rotation, 
             scale: scale, 
             world: world,
             center: Vec3::new(0.0,0.0,0.0),
             dirty: false 
            }
    }

    pub fn translate(&mut self,t: &Vec3){
        self.dirty = true;

        self.pos.x += t.x;
        self.pos.y += t.y;
        self.pos.z += t.z;        
    }

    pub fn rotate(&mut self,r: &Vec4){
        self.dirty = true;
        self.rotation = self.rotation * *r;
        self.rotation.normalize();
    }

    pub fn scale_by(&mut self, factor: Vec3) {
        self.scale.x *= factor.x;
        self.scale.y *= factor.y;
        self.scale.z *= factor.z;
        self.dirty = true;
    }
    
    pub fn set_scale(&mut self,s: &Vec3){
        self.scale = *s;
    }
    pub fn set_center(&mut self,c: Vec3){
        self.dirty = true;
        self.center = c;
    } 


    pub fn recalculate_world(&mut self) {
        if !self.dirty { return; }

        let mut scale_mat = Mat4::new();
        scale_mat.scale(&self.scale);

        let mut rot_mat = Mat4::new();
        rot_mat.rotation(&self.rotation);

        let mut neg_center = Mat4::new();
        neg_center.translation(&Vec3::new(-self.center.x, -self.center.y, -self.center.z));

        let mut pos_center = Mat4::new();
        pos_center.translation(&self.center);

        let mut pos_mat = Mat4::new();
        pos_mat.translation(&self.pos);

        self.world = pos_mat * pos_center * rot_mat * scale_mat * neg_center;



        self.dirty = false;
    }



    
    pub fn mat4(&mut self) -> Mat4{
        self.recalculate_world();
        self.world.clone()
    }
}


type PosKey = (i32,i32,i32);

#[derive(Clone)]
pub struct Mesh {
    pub points: Vec<Vec3>,
    pub triangles: Vec<Triangle>,
    pub material_color: Vec<MatColor>
}

fn parse_err(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg)
}

impl Mesh {
    fn new() -> Self {
        Self {
            points: Vec::new(),
            triangles: Vec::new(),
            material_color: Vec::new(),
        }
    }

    fn add_triangle(&mut self,p1: Vec3, p2: Vec3, p3: Vec3,map: &mut HashMap<PosKey, usize>,material_idx: usize,) {
        fn quantize(v: f32) -> i32 { (v * 1000.0).round() as i32 }
        fn make_key(v: &Vec3) -> (i32, i32, i32) {
            (quantize(v.x), quantize(v.y), quantize(v.z))
        }

        let mut values = [0usize; 3];
        for (i, (key, point)) in [(make_key(&p1), p1),(make_key(&p2), p2),(make_key(&p3), p3),].iter().enumerate(){
            let index = map.entry(*key).or_insert_with(|| {
                self.points.push(*point);
                self.points.len() - 1
            });
            values[i] = *index;
        }

        self.triangles.push(Triangle::new(values[0], values[1], values[2], material_idx));
    }

    fn load_mtl(&mut self, path: &str) -> io::Result<HashMap<String, usize>> {
        let file = File::open(Path::new(path))?;
        let reader = io::BufReader::new(file);
        let mut name_to_idx: HashMap<String, usize> = HashMap::new();

        // Accumulate material properties, then push once per `newmtl` block.
        let mut current_name: Option<String> = None;
        let mut current_kd: Option<(f32, f32, f32)> = None;

        // Helper: flush the pending material entry into self.materials.
        let flush = |name: &Option<String>, kd: &Option<(f32, f32, f32)>,
                         materials: &mut Vec<MatColor>,
                         name_to_idx: &mut HashMap<String, usize>| {
            if let Some(n) = name {
                let (r, g, b) = kd.unwrap_or((1.0, 1.0, 1.0));
                let idx = materials.len();
                materials.push(MatColor::new(
                    (r * 255.0) as u8,
                    (g * 255.0) as u8,
                    (b * 255.0) as u8,
                ));
                name_to_idx.insert(n.clone(), idx);
            }
        };

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            let mut parts = line.split_whitespace();
            match parts.next() {
                Some("newmtl") => {
                    flush(&current_name, &current_kd, &mut self.material_color, &mut name_to_idx);
                    current_name = Some(parts.next().unwrap_or("").to_string());
                    current_kd = None;
                }
                Some("Kd") => {
                    let r = parts.next().and_then(|v| v.parse::<f32>().ok()).unwrap_or(1.0);
                    let g = parts.next().and_then(|v| v.parse::<f32>().ok()).unwrap_or(1.0);
                    let b = parts.next().and_then(|v| v.parse::<f32>().ok()).unwrap_or(1.0);
                    // Store; don't push yet — other properties (Ks, Ka, etc.) may follow.
                    current_kd = Some((r, g, b));
                }
                // Future properties like Ks, Ka, Ns can be parsed here and stored
                // in local variables without creating premature material entries.
                _ => {}
            }
        }

        // Flush the last material in the file.
        flush(&current_name, &current_kd, &mut self.material_color, &mut name_to_idx);

        Ok(name_to_idx)
    }

    fn from_obj(&mut self, path: &str) -> io::Result<()> {
        let mut map: HashMap<PosKey, usize> = HashMap::new();
        let file = File::open(Path::new(path))?;
        let reader = io::BufReader::new(file);
        let mut vertices: Vec<Vec3> = Vec::new();
        let mut mtl_map: HashMap<String, usize> = HashMap::new();
        let mut current_material_idx: usize = 0;

        // Index 0 is always the default white material.
        // load_mtl is aware of self.materials.len() when it starts, so appended
        // material indices are naturally offset past this entry — but we document
        // the dependency explicitly here so future readers understand the contract.
        self.material_color.push(MatColor::new(255,255,255)); // index 0 = fallback white

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut parts = line.split_whitespace();
            match parts.next() {
                Some("mtllib") => {
                    let mtl_filename = parts.next().unwrap_or("");
                    let obj_dir = Path::new(path).parent().unwrap_or(Path::new(""));
                    let mtl_path = obj_dir.join(mtl_filename);
                    match self.load_mtl(mtl_path.to_str().unwrap_or("")) {
                        Ok(loaded) => mtl_map = loaded,
                        Err(e) => eprintln!("Warning: could not load mtllib '{}': {}", mtl_filename, e),
                    }
                }
                Some("usemtl") => {
                    let name = parts.next().unwrap_or("");
                    match mtl_map.get(name) {
                        Some(&idx) => current_material_idx = idx,
                        None => {
                            eprintln!("Warning: material '{}' not found, falling back to default", name);
                            current_material_idx = 0;
                        }
                    }
                }
                Some("v") => {
                    let x: f32 = parts.next()
                        .ok_or_else(|| parse_err("missing x in vertex"))?
                        .parse()
                        .map_err(|_| parse_err("invalid x in vertex"))?;
                    let y: f32 = parts.next()
                        .ok_or_else(|| parse_err("missing y in vertex"))?
                        .parse()
                        .map_err(|_| parse_err("invalid y in vertex"))?;
                    let z: f32 = parts.next()
                        .ok_or_else(|| parse_err("missing z in vertex"))?
                        .parse()
                        .map_err(|_| parse_err("invalid z in vertex"))?;
                    vertices.push(Vec3::new(x, y, z));
                }
                Some("f") => {
                    let indices: Result<Vec<usize>, io::Error> = parts
                        .map(|p| {
                            let raw = p.split('/').next()
                                .ok_or_else(|| parse_err("malformed face token"))?;
                            let i: usize = raw.parse::<usize>()
                                .map_err(|_| parse_err("invalid face index"))?;
                            // OBJ indices are 1-based; check bounds before subtracting.
                            if i == 0 || i > vertices.len() {
                                return Err(parse_err("face index out of range"));
                            }
                            Ok(i - 1)
                        })
                        .collect();

                    let indices = indices?;
                    match indices.len() {
                        3 => {
                            self.add_triangle(
                                vertices[indices[0]], vertices[indices[1]], vertices[indices[2]],
                                &mut map, current_material_idx,
                            );
                        }
                        4 => {
                            self.add_triangle(
                                vertices[indices[0]], vertices[indices[1]], vertices[indices[2]],
                                &mut map, current_material_idx,
                            );
                            self.add_triangle(
                                vertices[indices[0]], vertices[indices[2]], vertices[indices[3]],
                                &mut map, current_material_idx,
                            );
                        }
                        n => eprintln!("Warning: skipping face with {} vertices (only 3 and 4 supported)", n),
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Returns the geometric center of the mesh's axis-aligned bounding box.
    /// Unlike a vertex centroid, this is not biased by vertex density.
    pub fn compute_center(&self) -> Vec3 {
        if self.points.is_empty() {
            return Vec3::new(0.0, 0.0, 0.0);
        }

        let mut min = self.points[0];
        let mut max = self.points[0];

        for p in &self.points {
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            min.z = min.z.min(p.z);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
            max.z = max.z.max(p.z);
        }

        Vec3::new(
            (min.x + max.x) * 0.5,
            (min.y + max.y) * 0.5,
            (min.z + max.z) * 0.5,
        )
    }
}

impl Entity{
    pub fn new() -> Self{
        Self{
            color: None,
            mesh: Mesh::new(),
            transform: Transform::new(),
        }
    }
    pub fn set_color(&mut self,r: u8,g: u8,b: u8){
        self.color = Some(MatColor::new(r,g,b));
    }
    pub fn get_color(&self) -> Option<MatColor>{
        self.color.clone()
    }
    pub fn load_obj(&mut self,path: &str) -> io::Result<()>{
        self.mesh.from_obj(path)?;
        self.set_center();
        Ok(())
    }
    fn set_center(&mut self){
        let center = self.mesh.compute_center();
        self.transform.center = center;
    }
}
