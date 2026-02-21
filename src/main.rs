
use std::{collections::HashMap, io::{self, Write}, time::Duration}; 
use std::thread::sleep; 
use std::fs::File; 
use std::io::BufRead; 
use std::path::Path; 
use std::time::Instant; 

use std::thread;
use std::sync::{Arc, Mutex};


mod algebra;
mod input; 
mod terminal;

use algebra::{Vec3,Vec4,Mat4,Triangle};
use winapi::um::winnt::INCREF;



struct Entity {
    transform: Transform,
    mesh: Mesh,
}

struct Transform{
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


struct ZBuffer {
    depths : Vec<f32>,
    width: usize
}

impl ZBuffer {
    fn new(term_size: &(u16, u16)) -> Self {
        Self {
            depths: vec![f32::INFINITY; term_size.0 as usize * term_size.1 as usize],
            width: term_size.0 as usize,
        }
    }

    #[inline(always)]
    fn test_and_set_idx(&mut self, idx: usize, z: f32) -> bool {
        if idx >= self.depths.len() {
            return false;
        }
        if z < self.depths[idx] {
            self.depths[idx] = z;
            true
        } else {
            false
        }
    }

    pub fn resize(&mut self, term_size: (u16, u16)) {
        let new_width  = term_size.0 as usize;
        let new_height = term_size.1 as usize;

        self.width = new_width;
        self.depths.clear();
        self.depths.resize(new_width * new_height, f32::INFINITY);
    }

}


type PosKey = (i32,i32,i32);

#[derive(Clone)]
struct Mesh {
    points: Vec<Vec3>,
    triangles: Vec<Triangle>,
    map: HashMap<PosKey,usize>,
}

impl Mesh {
    fn new() -> Self {
        Self {
            points: Vec::new(),
            triangles: Vec::new(),
            map: HashMap::new()
        }
    }



    fn add_triangle(&mut self, p1: Vec3, p2: Vec3, p3: Vec3) {

        fn quantize(v: f32) -> i32 {
            (v * 1000.0).round() as i32
        }

        fn make_key(v: &Vec3) -> (i32, i32, i32) {
            (quantize(v.x), quantize(v.y), quantize(v.z))
        }

        let mut values = [0usize; 3];

        for (i, (key, point)) in [
            (make_key(&p1), p1),
            (make_key(&p2), p2),
            (make_key(&p3), p3),
        ]
        .iter()
        .enumerate()
        {
            let index = *self.map.entry(*key).or_insert_with(|| {
                self.points.push(*point);
                self.points.len() - 1
            });

            values[i] = index;
        }

        self.triangles.push(algebra::Triangle::new(
            values[0],
            values[1],
            values[2],
        ));
    }

    


    fn from_obj(&mut self, path: &str) -> io::Result<()> {
        let file = File::open(Path::new(path))?;
        let reader = io::BufReader::new(file);

        let mut vertices: Vec<Vec3> = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut parts = line.split_whitespace();
            match parts.next() {
                Some("v") => {
                    let x: f32 = parts.next().unwrap().parse().unwrap();
                    let y: f32 = parts.next().unwrap().parse().unwrap();
                    let z: f32 = parts.next().unwrap().parse().unwrap();
                    vertices.push(Vec3::new(x, y, z));
                }
                Some("f") => {
                    let indices: Vec<usize> = parts
                        .map(|p| {
                            p.split('/')
                                .next()
                                .unwrap()
                                .parse::<usize>()
                                .unwrap()
                                - 1
                        })
                        .collect();
                    if indices.len() == 3 {
                        self.add_triangle(vertices[indices[0]],vertices[indices[1]],vertices[indices[2]]);
                    } else if indices.len() == 4 {
                        self.add_triangle(vertices[indices[0]],vertices[indices[1]],vertices[indices[2]]);
                        self.add_triangle(vertices[indices[0]],vertices[indices[2]],vertices[indices[3]]);
                    }
                }
                _ => {}
            }
        }
        self.map.clear();
        Ok(())
    }
    pub fn compute_center(&self) -> Vec3 {

        let mut c = Vec3::new(0.0, 0.0, 0.0);

        for point in &self.points{
            c.x += point.x;
            c.y += point.y;
            c.z += point.z;
        } 
        let n = (self.points.len()) as f32;
        Vec3::new(c.x / n, c.y / n, c.z / n)

    }


}


impl Entity{
    fn new() -> Self{
        Self{
            mesh: Mesh::new(),
            transform: Transform::new(),
        }
    }
    fn load_obj(&mut self,path: &str) -> io::Result<()>{
        self.mesh.from_obj(path)?;
        Ok(())
    }
    fn set_center(&mut self){
        let center = self.mesh.compute_center();
        self.transform.center = center;
    }
}

struct Renderer{
    zbuffer: ZBuffer,
    screen_size: (u16,u16),
    buffer: Vec<u8>,
    camera: Vec3,
    render_buffer: Vec<Vec3>,
    light_dir: Vec3,
}

impl Renderer{

    // TODO REPLACE ALL .resolve WITH THIS
    fn resolve_point(camera: &Vec3,point: &Vec3) -> Option<(f32,f32)>{
        
        let rel_x = point.x - camera.x;
        let rel_y= point.y - camera.y;
        let rel_z = point.z - camera.z;


        const MIN: f32 = 0.1;
        if rel_z < MIN {
            return None;
        }
        const K1: f32 = 1.2;
        let new_x = ((rel_x * K1) / rel_z) + 0.5;
        let new_y = ((rel_y * K1) / rel_z) + 0.5;
        Some((new_x, new_y))
    }

    pub fn new() -> Self{
        let term = terminal::get_terminal_size();
        let mut light_dir = Vec3::new(0.5,-0.5,-1.0);

        let mut buffer: Vec<u8> = Vec::new();

        buffer = vec![b' '; term.0 as usize * term.1 as usize];

        light_dir.normalize();
        Renderer { 
            zbuffer: ZBuffer::new(&term),
            screen_size: term,
            buffer: buffer,
            camera: (Vec3::new(0.0,0.0,0.0)),
            render_buffer: Vec::new(),
            light_dir,
        }
    }

    fn render(&mut self,entities: &mut Vec<Entity>){

        const TARGET_FPS: u64 = 60;
        const FRAME_TIME: Duration = Duration::from_millis(1000 / TARGET_FPS);


        let frame_start = Instant::now();




        let start = Instant::now();

        let term = terminal::get_terminal_size();
        if term != self.screen_size{
            self.zbuffer.resize(term);
            self.buffer = vec![b' '; term.0 as usize * term.1 as usize];
            self.screen_size = term;
        }

        // log_to_file(&format!("time taken to get terminal info: {:?}",start.elapsed()));
        // log_to_file(&format!("terminal size: {:?}",term));
        // log_to_file(&format!("buffer size: {:?}",self.buffer.len()));
        

        

        let start = Instant::now();

        for entity in entities{
            self.draw_entity(entity);
        } 

        log_to_file(&format!("time taken to do mathemtaical thingy on entity: {:?}",start.elapsed()));


         let start = Instant::now();

        self.render_buffer();

        log_to_file(&format!("time taken to render buffer: {:?}",start.elapsed()));

        self.buffer.fill(b' ');
        self.zbuffer.depths.fill(f32::INFINITY);

     
       let frame_elapsed = frame_start.elapsed();
            if frame_elapsed < FRAME_TIME {
                sleep(FRAME_TIME - frame_elapsed);
       }

    }

    fn apply_transform_to_buffer(&mut self,entity: &mut Entity){
        let mat4 = entity.transform.mat4();
        
        self.render_buffer.resize(entity.mesh.points.len(),Vec3::new(0.0,0.0,0.0));

        for (itera,item) in entity.mesh.points.iter().enumerate(){
            self.render_buffer[itera] = &mat4 * *item; 
        }
    }


    fn fill_triangle(&mut self, tri: &Triangle, ch: char) {
        let (t0, t1, t2) = tri.return_indices();

        let (r0, r1, r2) = match (
            self.render_buffer[t0].resolve(&self.camera),
            self.render_buffer[t1].resolve(&self.camera),
            self.render_buffer[t2].resolve(&self.camera),
        ) {
            (Some(a), Some(b), Some(c)) => (a, b, c),
            _ => return,
        };

        let (nx0, ny0) = r0;
        let (nx1, ny1) = r1;
        let (nx2, ny2) = r2;

        if nx0 < 0.0 && nx1 < 0.0 && nx2 < 0.0 { return; }
        if nx0 > 1.0 && nx1 > 1.0 && nx2 > 1.0 { return; }
        if ny0 < 0.0 && ny1 < 0.0 && ny2 < 0.0 { return; }
        if ny0 > 1.0 && ny1 > 1.0 && ny2 > 1.0 { return; }

        let sw = self.screen_size.0 as f32;
        let sh = self.screen_size.1 as f32;

        let x0 = (sw * nx0).round() as i32;
        let y0 = (sh * ny0).round() as i32;
        let x1 = (sw * nx1).round() as i32;
        let y1 = (sh * ny1).round() as i32;
        let x2 = (sw * nx2).round() as i32;
        let y2 = (sh * ny2).round() as i32;

        let denom = ((y1 - y2) * (x0 - x2) + (x2 - x1) * (y0 - y2)) as f32;
        if denom.abs() < 0.001 { return; }
        let inv_denom = 1.0 / denom;

        let min_x = x0.min(x1).min(x2).max(0);
        let max_x = x0.max(x1).max(x2).min(self.screen_size.0 as i32 - 1);
        let min_y = y0.min(y1).min(y2).max(0);
        let max_y = y0.max(y1).max(y2).min(self.screen_size.1 as i32 - 1);

        if min_x > max_x || min_y > max_y { return; }

        let z0 = self.render_buffer[t0].z;
        let z1 = self.render_buffer[t1].z;
        let z2 = self.render_buffer[t2].z;

        let w0_x_step = (y1 - y2) as f32 * inv_denom;
        let w1_x_step = (y2 - y0) as f32 * inv_denom;

        let w0_y_step = (x2 - x1) as f32 * inv_denom;
        let w1_y_step = (x0 - x2) as f32 * inv_denom;

        let w0_seed = ((y1 - y2) * (min_x - x2) + (x2 - x1) * (min_y - y2)) as f32 * inv_denom;
        let w1_seed = ((y2 - y0) * (min_x - x2) + (x0 - x2) * (min_y - y2)) as f32 * inv_denom;

        let mut w0_row = w0_seed;
        let mut w1_row = w1_seed;

        let width = self.screen_size.0 as usize;

        for py in min_y..=max_y {
            let mut w0 = w0_row;
            let mut w1 = w1_row;
            let mut inside = false;
            let row_base = py as usize * width;

            for px in min_x..=max_x {
                let w2 = 1.0 - w0 - w1;
                if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                    inside = true;
                    let z = w0 * z0 + w1 * z1 + w2 * z2;
                    if self.zbuffer.test_and_set_idx(row_base + px as usize, z) {
                        draw_pixel_to_buffer(px as u16, py as u16, ch, &mut self.buffer, self.screen_size);
                    }
                } else if inside {
                    break;
                }
                w0 += w0_x_step;
                w1 += w1_x_step;
            }

            w0_row += w0_y_step;
            w1_row += w1_y_step;
        }
    }


    fn intensity_to_char(&self,intensity: f32) -> char {
        let chars = [
        ' ', '`', '.', ',', '\'', ':', ';', '-' , '^', 
        '~', '=', '+', '*', '!' ,'o', 'O', '#', '%', '@', '█'
        ];

        let gamma = intensity.powf(0.6);
        let idx = (gamma * (chars.len() - 1) as f32) as usize;
        chars[idx.min(chars.len() - 1)]
    }

    fn calculate_lighting(&self,normal: Vec3, light_dir: Vec3) -> f32 {
        let light_fall_off = 0.3;
        let dot = normal.dot(light_dir);
        let distance_fade = (1.0 / (1.0 + normal.z * light_fall_off)).clamp(0.1, 1.0);
        (dot * distance_fade).max(0.0)
    }



    fn render_buffer(&self) {
        let w = self.screen_size.0 as usize;
        let h = self.screen_size.1 as usize;
        let mut out: Vec<u8> = Vec::with_capacity(w * h + h + 8);
        
        out.extend_from_slice(b"\x1B[H");
        
        for y in 0..h {
            let row_start = y * w;
            out.extend_from_slice(&self.buffer[row_start..row_start + w]);
            out.push(b'\n');
        }
        
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(&out).unwrap();
        handle.flush().unwrap();
    }

    fn draw_entity(&mut self,entity: &mut Entity){
        let start= Instant::now();
        self.apply_transform_to_buffer(entity); // local buffer of points
        let now = start.elapsed();
        log_to_file(&format!("time taken to apply transform to buffer is : {:?}",now));


        let mut normal_total = Duration::ZERO;
        let mut cull_total = Duration::ZERO;
        let mut lighting_total = Duration::ZERO;
        let mut raster_total = Duration::ZERO;


        for tri in &entity.mesh.triangles {

            let start = Instant::now();
            let normal = tri.normal(&self.render_buffer);
            normal_total += start.elapsed();

            let start = Instant::now();
            let view_dir = Vec3::new(0.0, 0.0, -1.0);
            let dot = normal.dot(view_dir);
            if dot <= 0.0 { continue; }
            cull_total += start.elapsed();

            let start = Instant::now();
            let intensity = self.calculate_lighting(normal, self.light_dir);
            let ch = self.intensity_to_char(intensity);
            lighting_total += start.elapsed();

            let start = Instant::now();
            self.fill_triangle(tri, ch);
            raster_total += start.elapsed();
    }

    log_to_file(&format!(
        "Normals: {:?}, Cull: {:?}, Lighting: {:?}, Raster: {:?}",
        normal_total, cull_total, lighting_total, raster_total
    ));
    }
}





fn main() {
    

    let mut renderer = Renderer::new();

    let mut entities: Vec<Entity> = Vec::new();

    let mut cube = Entity::new();
    let _ = cube.load_obj("obj/skull.obj");

    cube.set_center();

    // cube.transform.scale_by(Vec3::new( 0.5,0.5,0.5));
    cube.transform.translate(&Vec3 { x: -0.0, y: 10.0 ,z:45.0 });
    

    entities.push(cube);


    loop {
            renderer.render(&mut entities);
            for entity in &mut entities{
                entity.transform.rotate(&Vec4::new(0.03, 0.0, 0.03, 1.0));
        }
    }

   

        
}


fn draw_pixel_to_buffer(x: u16, y: u16, ch: char, buffer: &mut Vec<u8>, screen_size: (u16, u16)) {
    let idx = (y as usize) * (screen_size.0 as usize) + (x as usize);
    if idx < buffer.len() {
        buffer[idx] = ch as u8;
    }
}

use std::fs::OpenOptions;

fn log_to_file(msg: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("timing.log")
        .unwrap();

    writeln!(file, "{msg}").unwrap();
}
