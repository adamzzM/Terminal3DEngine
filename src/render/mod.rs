
use crate::{
    algebra::{
        Triangle, Vec3
    },
    entity::{
        self, Entity, MatColor
    },
    terminal::get_terminal_size,

};

use std::{
    time::{Duration, Instant},
    thread::sleep,
    io::{self,Write}
};

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


// helper functions

#[inline(always)]
fn make_cell(ch: u8, r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | ch as u32
}

#[inline(always)]
fn unpack_cell(cell: u32) -> (u8, u8, u8, u8) {
    (
        (cell >> 24) as u8,         // r
        (cell >> 16) as u8,         // g
        (cell >> 8)  as u8,         // b
        cell as u8,                 // ch
    )
}

#[inline(always)]
fn push_num(out: &mut Vec<u8>, n: u8) {
    if n >= 100 {
        out.push(b'0' + n / 100);
        out.push(b'0' + (n % 100) / 10);
    } else if n >= 10 {
        out.push(b'0' + n / 10);
    }
    out.push(b'0' + n % 10);
}

#[inline(always)]
fn push_color(out: &mut Vec<u8>, r: u8, g: u8, b: u8) {
    out.extend_from_slice(b"\x1B[38;2;");
    push_num(out, r); out.push(b';');
    push_num(out, g); out.push(b';');
    push_num(out, b); out.push(b'm');
}



pub struct Renderer{
    zbuffer: ZBuffer,
    screen_size: (u16,u16),
    buffer: Vec<u32>,
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
        let term = get_terminal_size();
        let mut light_dir = Vec3::new(0.5,-0.5,-1.0);

        let mut buffer: Vec<u32> = Vec::new();

        buffer = vec![b' ' as u32; term.0 as usize * term.1 as usize];

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

    pub fn render(&mut self,entities: &mut Vec<Entity>){

        const TARGET_FPS: u64 = 60;
        const FRAME_TIME: Duration = Duration::from_millis(1000 / TARGET_FPS);


        let frame_start = Instant::now();


        let term = get_terminal_size();
        if term != self.screen_size{
            self.zbuffer.resize(term);
            self.buffer = vec![b' ' as u32; term.0 as usize * term.1 as usize];
            self.screen_size = term;
        }

        for entity in entities{
            self.draw_entity(entity);
        } 


        self.render_buffer();


        self.buffer.fill(b' ' as u32);
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


    fn fill_triangle(&mut self, tri: &Triangle, ch: u8,r: u8,g: u8,b: u8) {
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
                        self.buffer[(py as usize) * (self.screen_size.0 as usize) + (px as usize)] = make_cell(ch, r, g, b);
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

    fn intensity_to_char(&self,intensity: f32) -> u8{

        // '█'
        let chars = [
        b' ', b'`', b'.', b',', b'\'', b':', b';', b'-' , b'^', 
        b'~', b'=', b'+', b'*', b'!' ,b'o', b'O', b'#', b'%', b'@', 
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
        let mut out: Vec<u8> = Vec::with_capacity(w * h * 21);

        out.extend_from_slice(b"\x1B[H");

        let mut last_rgb: u32 = u32::MAX; // impossible value forces first emit

        for y in 0..h {
            for x in 0..w {
                let cell = self.buffer[y * w + x];
                let rgb = cell >> 8;  // top 3 bytes are r,g,b

                if rgb != last_rgb {
                    let r = (cell >> 24) as u8;
                    let g = (cell >> 16) as u8;
                    let b = (cell >> 8)  as u8;
                    push_color(&mut out, r, g, b);
                    last_rgb = rgb;
                }

                out.push(cell as u8); // ch is lowest byte
            }
            out.push(b'\n');
        }

        out.extend_from_slice(b"\x1B[0m");

        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(&out).unwrap();
        handle.flush().unwrap();
    }

    fn draw_entity(&mut self,entity: &mut Entity){

        self.apply_transform_to_buffer(entity); // local buffer of points

        let ent_color = entity.get_color();

        let is_color = ent_color.is_none();

        let mut r: u8 = 0;
        let mut g: u8 = 0;
        let mut b: u8 = 0;



        if !is_color{
            let c = ent_color.unwrap();
            r = c.r;
            g = c.g;
            b = c.b;
        }

        for tri in &entity.mesh.triangles {

            let normal = tri.normal(&self.render_buffer);

            let view_dir = Vec3::new(0.0, 0.0, -1.0);
            let dot = normal.dot(view_dir);
            if dot <= 0.0 { continue; }

            let intensity = self.calculate_lighting(normal, self.light_dir);
            let ch = self.intensity_to_char(intensity);

            if is_color{
                let mat = &entity.mesh.material_color[tri.return_mat_idx()];
                r = (mat.r as f32 * intensity) as u8;
                g = (mat.g as f32 * intensity) as u8;
                b = (mat.b as f32 * intensity) as u8;
            }

            self.fill_triangle(tri, ch,r,g,b);
        }
    }

}

