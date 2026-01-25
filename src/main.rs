use std::{io::{self, Write}, time::Duration}; 
use std::thread::sleep; 
use std::fs::File; 
use std::io::BufRead; 
use std::path::Path; 
use std::time::Instant; 

mod algebra;
mod input; 
mod terminal;


use algebra::{Vec3,Vec4,Mat4,Triangle};


const EPS: f32 = 1e-5;

struct Entity {
    transform: Transform,
    mesh: Mesh,
}

struct Transform{
    pos: Vec3,
    rotation: Vec4,
    scale: Vec3,
    world: Mat4,
    dirty: bool

}
impl Transform{
    pub fn new() -> Self{
        let pos = Vec3::new(0.0,0.0,0.0);
        let rotation = Vec4::new(0.0,0.0,0.0,1.0);
        let scale = Vec3::new(1.0,1.0,1.0);
        let mut world = Mat4::new();

        world.scale(&scale);
        world.rotation(&rotation);
        world.translation(&pos);


        Transform {
             pos: pos , 
             rotation: rotation, 
             scale: scale, 
             world: world,
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

    pub fn recalucate_world(&mut self){
        if !(self.dirty){return}
        let mut new_world = Mat4::new();
        new_world.scale(&self.scale);
        new_world.rotation(&self.rotation);
        new_world.translation(&self.pos);
        self.world = new_world;
        self.dirty = false;
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
    fn test_and_set(&mut self, x: usize, y: usize, z: f32) -> bool {
        if x >= self.width || y >= (self.depths.len() / self.width) {
            return false;
        }
        let idx = y * self.width + x;
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



#[derive(Clone)]
struct Mesh {
    points: Vec<Vec3>,
    triangles: Vec<Triangle>,
}

impl Mesh {
    fn new() -> Self {
        Self {
            points: Vec::new(),
            triangles: Vec::new(),
        }
    }

    // area for optimizatioonnnn
    fn add_triangle(&mut self,p1: Vec3,p2: Vec3,p3: Vec3){
        let mut values: [isize; 3] = [-1 ; 3];

        for (i,j) in self.points.iter().enumerate(){
            if j.approx_eq(&p1, EPS){
                values[0] = i as isize; 
            }
            if j.approx_eq(&p2, EPS){
                values[1] = i as isize; 
            }
            if j.approx_eq(&p3, EPS){
                values[2] = i as isize; 
            }
        }
        if values[0] == -1{
           self.points.push(p1);
           values[0] = (self.points.len() - 1 ) as isize; // for consistency
        }
        if values[1] == -1{
            self.points.push(p2);   
           values[1] = (self.points.len() - 1) as isize;
        }
        if values[2] == -1{
           self.points.push(p3);
           values[2] = (self.points.len() - 1) as isize;
        }
        self.triangles.push(algebra::Triangle::new(values[0] as usize,values[1] as usize,values[2] as usize));

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
                        // triangles.push(Triangle::new(
                        //     vertices[indices[0]],
                        //     vertices[indices[1]],
                        //     vertices[indices[2]],
                        // ));
                    } else if indices.len() == 4 {
                        self.add_triangle(vertices[indices[0]],vertices[indices[1]],vertices[indices[2]]);

                        self.add_triangle(vertices[indices[0]],vertices[indices[2]],vertices[indices[3]]);
                        // triangles.push(Triangle::new(
                        //     vertices[indices[0]],
                        //     vertices[indices[1]],
                        //     vertices[indices[2]],
                        // ));
                        // triangles.push(Triangle::new(
                        //     vertices[indices[0]],
                        //     vertices[indices[2]],
                        //     vertices[indices[3]],
                        // ));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    // fn shift(&mut self, offset: Vec3) {
    //     for tri in &mut self.triangles {
    //         tri.v0.x += offset.x;
    //         tri.v0.y += offset.y;
    //         tri.v0.z += offset.z;
    //         tri.v1.x += offset.x;
    //         tri.v1.y += offset.y;
    //         tri.v1.z += offset.z;
    //         tri.v2.x += offset.x;
    //         tri.v2.y += offset.y;
    //         tri.v2.z += offset.z;
    //     }
    // }

    // fn rotate_x_about_axis(&mut self, angle: f32, axis: Option<Vec3>) {
    //     let center = axis.unwrap_or_else(|| self.get_center());
    //     for tri in &mut self.triangles {
    //         for v in [&mut tri.v0, &mut tri.v1, &mut tri.v2] {
    //             let shifted = Vec3::new(v.x - center.x, v.y - center.y, v.z - center.z).rotate_x(angle);
    //             v.x = shifted.x + center.x;
    //             v.y = shifted.y + center.y;
    //             v.z = shifted.z + center.z;
    //         }
    //     }
    // }

    // fn rotate_y_about_axis(&mut self, angle: f32, axis: Option<Vec3>) {
    //     let center = axis.unwrap_or_else(|| self.get_center());
    //     for tri in &mut self.triangles {
    //         for v in [&mut tri.v0, &mut tri.v1, &mut tri.v2] {
    //             let shifted = Vec3::new(v.x - center.x, v.y - center.y, v.z - center.z).rotate_y(angle);
    //             v.x = shifted.x + center.x;
    //             v.y = shifted.y + center.y;
    //             v.z = shifted.z + center.z;
    //         }
    //     }
    // }

    // fn rotate_z_about_axis(&mut self, angle: f32, axis: Option<Vec3>) {
    //     let center = axis.unwrap_or_else(|| self.get_center());
    //     for tri in &mut self.triangles {
    //         for v in [&mut tri.v0, &mut tri.v1, &mut tri.v2] {
    //             let shifted = Vec3::new(v.x - center.x, v.y - center.y, v.z - center.z).rotate_z(angle);
    //             v.x = shifted.x + center.x;
    //             v.y = shifted.y + center.y;
    //             v.z = shifted.z + center.z;
    //         }
    //     }
    // }

    fn get_center(&self) -> Vec3 {
        let indice = self.triangles[0].return_indices(); 
        let mut min = indice;
        let mut max = indice;
        for tri in &self.triangles {
            for v in [&tri.v0, &tri.v1, &tri.v2] {
                min.x = min.x.min(v.x);
                min.y = min.y.min(v.y);
                min.z = min.z.min(v.z);
                max.x = max.x.max(v.x);
                max.y = max.y.max(v.y);
                max.z = max.z.max(v.z);
            }
        }
        Vec3 {
            x: (min.x + max.x) / 2.0,
            y: (min.y + max.y) / 2.0,
            z: (min.z + max.z) / 2.0,
        }
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
        self.mesh = Mesh::from_obj(path)?;
        Ok(())
    }
}

struct renderer{
    zbuffer: ZBuffer,
    screen_size: (u16,u16),
    buffer: Vec<char>,
    camera: Vec3,
    render_buffer: Vec<Vec3>
}
impl renderer{
    pub fn new() -> Self{
        let term = terminal::get_terminal_size();
        renderer { 
            zbuffer: ZBuffer::new(&term),
            screen_size: term,
            buffer: Vec::new(),
            camera: (Vec3::new(0.0,0.0,0.0)),
            render_buffer: Vec::new(),
        }
    }
    fn start_of_cycle(&mut self){
        let term = terminal::get_terminal_size();
        if term != self.screen_size{
            self.zbuffer.resize(term);
            self.buffer = vec![' '; term.0 as usize * term.1 as usize];
            self.screen_size = term;
        }
    }

    fn draw_entity(&mut self,entity: Entity){
    }
}



fn intensity_to_char(intensity: f32) -> char {
    let chars = [
    ' ', '`', '.', '\'', ',', ':', ';', '"', '^', 
    '-', '~', '=', '+', '*', 'o', 'O', '#', '%', '@', '█'
    ];
    let gamma = intensity.powf(0.6);
    let idx = (gamma * (chars.len() - 1) as f32) as usize;
    chars[idx.min(chars.len() - 1)]
}

fn main() {
    const TARGET_FPS: u64 = 60;
    const FRAME_TIME: Duration = Duration::from_millis(1000 / TARGET_FPS);

    let mut term_size = terminal::get_terminal_size();

    let mut buffer: Vec<char> = vec![' '; term_size.0 as usize * term_size.1 as usize];

    let mut new_cube = Entity::new();
    let _ = new_cube.load_obj("obj/skull.obj").expect("failed to load obj");




    let mut cube = Mesh::from_obj("obj/cube.obj").unwrap();
    cube.shift(Vec3::new(0.0,0.0,5.0));



    //cube.rotate_z_about_axis(90.0, None);
    //cube.rotate_x_about_axis(70.0, None);

    let mut angle = 0.0;
    let mut offset = Vec3::new(0.0,0.0,0.0);

    let mut camera = Vec3::new(0.0,0.0,0.0);

    loop {

        if term_size != terminal::get_terminal_size() {
            term_size = terminal::get_terminal_size();
            buffer = vec![' '; term_size.0 as usize * term_size.1 as usize];
        }

        let frame_start = Instant::now();
        let mut z_buffer = ZBuffer::new(&term_size);


        render_buffer(&buffer, term_size);
        buffer.fill(' '); 

        let mut mesh = cube.clone();
        mesh.rotate_y_about_axis(angle, None);
        // mesh.rotate_z_about_axis(angle, None);

        mesh.shift(offset);
   //     draw_mesh(&secon_mesh, term_size, &mut z_buffer, &mut buffer, &camera);
        draw_mesh(&mesh, term_size, &mut z_buffer,&mut buffer,&camera);

        angle += 0.06;
        //camera.x += 0.2;
        //offset.z -= 0.06;

        if let Some(key) = input::poll_key() {
            match key {
                b'w' => camera.z += 1.0,
                b's' => camera.z -= 1.0,
                b'a' => camera.x -= 1.0,
                b'd' => camera.x += 1.0,
                b'q' => break,
                _ => {}
            }
        }

        let frame_elapsed = frame_start.elapsed();
        if frame_elapsed < FRAME_TIME {
            sleep(FRAME_TIME - frame_elapsed);
        }
    }
}


fn draw_mesh(mesh: &Mesh, screen_size: (u16, u16), z_buffer: &mut ZBuffer,buffer: &mut Vec<char>,camera: &Vec3) {
    let light_dir = Vec3::new(0.5, -0.5, -1.0);
    let len = (light_dir.x * light_dir.x + light_dir.y * light_dir.y + light_dir.z * light_dir.z).sqrt();
    let light_dir = Vec3::new(light_dir.x / len, light_dir.y / len, light_dir.z / len);

    for tri in &mesh.triangles {
        let normal = tri.normal();
        
        // only draw if facing camera
        let view_dir = Vec3::new(0.0, 0.0, -1.0);
        let dot = normal.dot(view_dir);
        if dot <= 0.0 {
            continue;
        }
        // only draw if visible to camera TODO
        
        let intensity = calculate_lighting(normal, light_dir);
        let ch = intensity_to_char(intensity);
        
        fill_triangle(tri, ch, screen_size, z_buffer,buffer,camera);
    }
}


fn calculate_lighting(normal: Vec3, light_dir: Vec3) -> f32 {
    let light_fall_off = 0.3;
    let dot = normal.dot(light_dir);
    let distance_fade = (1.0 / (1.0 + normal.z * light_fall_off)).clamp(0.1, 1.0);
    (dot * distance_fade).max(0.0)
}

fn fill_triangle(tri: &Triangle, ch: char, screen_size: (u16, u16), z_buffer: &mut ZBuffer,buffer: &mut Vec<char>,camera: &Vec3) {
    let r0 = tri.v0.resolve(camera);
    let r1 = tri.v1.resolve(camera);
    let r2 = tri.v2.resolve(camera);

    if r0.is_none() || r1.is_none() || r2.is_none() {
        return;
    }

    let (nx0, ny0) = r0.unwrap();
    let (nx1, ny1) = r1.unwrap();
    let (nx2, ny2) = r2.unwrap();

    let x0 = (screen_size.0 as f32 * nx0).round() as i32;
    let y0 = (screen_size.1 as f32 * ny0).round() as i32;
    let x1 = (screen_size.0 as f32 * nx1).round() as i32;
    let y1 = (screen_size.1 as f32 * ny1).round() as i32;
    let x2 = (screen_size.0 as f32 * nx2).round() as i32;
    let y2 = (screen_size.1 as f32 * ny2).round() as i32;

    let min_x = x0.min(x1).min(x2).max(0);
    let max_x = x0.max(x1).max(x2).min(screen_size.0 as i32 - 1);
    let min_y = y0.min(y1).min(y2).max(0);
    let max_y = y0.max(y1).max(y2).min(screen_size.1 as i32 - 1);

    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let (w0, w1, w2) = barycentric(px, py, x0, y0, x1, y1, x2, y2);
            
            if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                let z = w0 * tri.v0.z + w1 * tri.v1.z + w2 * tri.v2.z;  // interpolated depth
                
                if z_buffer.test_and_set(px as usize, py as usize, z) {
                    draw_pixel_to_buffer(px as u16, py as u16, ch,buffer,screen_size);
                }
            }
        }
    }
}

fn barycentric(px: i32, py: i32, x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32) -> (f32, f32, f32) {
    let denom = ((y1 - y2) * (x0 - x2) + (x2 - x1) * (y0 - y2)) as f32;
    if denom.abs() < 0.001 {
        return (-1.0, -1.0, -1.0);
    }
    let w0 = ((y1 - y2) * (px - x2) + (x2 - x1) * (py - y2)) as f32 / denom;
    let w1 = ((y2 - y0) * (px - x2) + (x0 - x2) * (py - y2)) as f32 / denom;
    let w2 = 1.0 - w0 - w1;
    (w0, w1, w2)
}

fn draw_pixel_to_buffer(x: u16, y: u16, ch: char, buffer: &mut Vec<char>, screen_size: (u16, u16)) {
    let idx = (y as usize) * (screen_size.0 as usize) + (x as usize);
    if idx < buffer.len() {
        buffer[idx] = ch;
    }
}

fn render_buffer(buffer: &Vec<char>, screen_size: (u16, u16)) {
    let mut stdout = io::stdout();
    for y in 0..screen_size.1 {
        write!(stdout, "\x1B[{};1H", y + 1).unwrap(); // move cursor to beginning of line 
        for x in 0..screen_size.0 {
            let idx = (y as usize) * (screen_size.0 as usize) + (x as usize);
            let ch = buffer[idx];
            write!(stdout, "{}", ch).unwrap();
        }
    }
    stdout.flush().unwrap();
}