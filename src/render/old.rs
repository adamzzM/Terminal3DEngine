
use crate::{
    algebra::{
        Triangle, Vec3
    },
    entity::Entity,
    terminal::get_terminal_size,
};

use std::{
    io::{self,Write}, num, thread::sleep, time::{Duration, Instant}
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
    prev_buffer: Vec<u32>

}

impl Renderer{

    // TODO REPLACE ALL .resolve WITH THIS
    fn resolve_point(&self,point: &Vec3) -> Option<(f32,f32)>{
        
        let rel_x = point.x - self.camera.x;
        let rel_y= point.y - self.camera.y;
        let rel_z = point.z - self.camera.z;


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

        print!("\x1B[2J\x1B[H"); // clear screen


        let term = get_terminal_size();
        let mut light_dir = Vec3::new(0.5,-0.5,-1.0);

        let mut buffer: Vec<u32> = Vec::new();

        buffer = vec![b' ' as u32; term.0 as usize * term.1 as usize];

        let mut prev_buffer: Vec<u32> = Vec::new();

        prev_buffer = vec![b' ' as u32; term.0 as usize * term.1 as usize];

        light_dir.normalize();
        Renderer { 
            zbuffer: ZBuffer::new(&term),
            screen_size: term,
            buffer: buffer,
            camera: (Vec3::new(0.0,0.0,0.0)),
            render_buffer: Vec::new(),
            light_dir,
            prev_buffer: prev_buffer
        }
    }

    pub fn render(&mut self,entities: &mut Vec<Entity>){

        const TARGET_FPS: u64 = 60;
        const FRAME_TIME: Duration = Duration::from_millis(1000 / TARGET_FPS);


        let frame_start = Instant::now();


        let term = get_terminal_size();
        if term != self.screen_size{
            print!("\x1B[2J\x1B[H"); // clear scree
            self.zbuffer.resize(term);
            self.buffer = vec![b' ' as u32; term.0 as usize * term.1 as usize];
            self.prev_buffer = vec![b' ' as u32; term.0 as usize * term.1 as usize];
            self.screen_size = term;
        }

        for entity in entities{
            self.draw_entity(entity);
        } 


        self.render_buffer();
        self.swap_buffers(); // adjust buffer


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

            fn render_buffer(&self) {
                let w = self.screen_size.0 as usize;
                let h = self.screen_size.1 as usize;
                let mut out: Vec<u8> = Vec::with_capacity(w * h * 8); 

                let mut last_rgb: u32 = u32::MAX;
                let mut cursor_row: usize = usize::MAX;
                let mut cursor_col: usize = usize::MAX;

                for y in 0..h {
                    for x in 0..w {
                        let idx = y * w + x;
                        let cell = self.buffer[idx];

                        if cell == self.prev_buffer[idx] {
                            cursor_col = usize::MAX; // must reposition
                            continue;
                        }

                        // Reposition cursor only when needed
                        if cursor_row != y || cursor_col != x {
                            // \x1B[row;colH — 1-indexed
                            write!(out, "\x1B[{};{}H", y + 1, x + 1).unwrap();
                            cursor_row = y;
                            cursor_col = x;
                            last_rgb = u32::MAX; // color state is lost after move
                        }

                        let rgb = cell >> 8;
                        if rgb != last_rgb {
                            let r = (cell >> 24) as u8;
                            let g = (cell >> 16) as u8;
                            let b = (cell >> 8)  as u8;
                            push_color(&mut out, r, g, b);
                            last_rgb = rgb;
                        }

                        out.push(cell as u8);
                        cursor_col += 1;
                    }
                    cursor_row = y; // we may have written on this row
                }

                if !out.is_empty() {
                    out.extend_from_slice(b"\x1B[0m");
                    let stdout = io::stdout();
                    let mut handle = stdout.lock();
                    handle.write_all(&out).unwrap();
                    handle.flush().unwrap();
                }
    }


    fn swap_buffers(&mut self) {
        self.prev_buffer.copy_from_slice(&self.buffer);
    }

    

    fn draw_entity(&mut self,entity: &mut Entity){

        fn fill_triangle(tri: &Triangle, ch: u8,r: u8,g: u8,b: u8,screen_size:(u16, u16),render_buffer: Vec<Vec3>,camera: Vec3 ) -> Option<Vec<(usize,usize,u32)>> {
                let mut zbuffer = ZBuffer::new(&screen_size);

                let mut buffer: Vec<(usize , usize , u32)> = Vec::new();

                let (t0, t1, t2) = tri.return_indices();

                let (r0, r1, r2) = match (
                    render_buffer[t0].resolve(&camera),
                    render_buffer[t1].resolve(&camera),
                    render_buffer[t2].resolve(&camera),
                ) {
                    (Some(a), Some(b), Some(c)) => (a, b, c),
                    _ => return None,
                };

                let (nx0, ny0) = r0;
                let (nx1, ny1) = r1;
                let (nx2, ny2) = r2;

                if nx0 < 0.0 && nx1 < 0.0 && nx2 < 0.0 { return None }
                if nx0 > 1.0 && nx1 > 1.0 && nx2 > 1.0 { return None }
                if ny0 < 0.0 && ny1 < 0.0 && ny2 < 0.0 { return None ; }
                if ny0 > 1.0 && ny1 > 1.0 && ny2 > 1.0 { return None; }

                let sw = screen_size.0 as f32;
                let sh = screen_size.1 as f32;

                let x0 = (sw * nx0).round() as i32;
                let y0 = (sh * ny0).round() as i32;
                let x1 = (sw * nx1).round() as i32;
                let y1 = (sh * ny1).round() as i32;
                let x2 = (sw * nx2).round() as i32;
                let y2 = (sh * ny2).round() as i32;

                let denom = ((y1 - y2) * (x0 - x2) + (x2 - x1) * (y0 - y2)) as f32;
                if denom.abs() < 0.001 { return None; }
                let inv_denom = 1.0 / denom;

                let min_x = x0.min(x1).min(x2).max(0);
                let max_x = x0.max(x1).max(x2).min(screen_size.0 as i32 - 1);
                let min_y = y0.min(y1).min(y2).max(0);
                let max_y = y0.max(y1).max(y2).min(screen_size.1 as i32 - 1);

                if min_x > max_x || min_y > max_y { return None; }

                let z0 = render_buffer[t0].z;
                let z1 = render_buffer[t1].z;
                let z2 = render_buffer[t2].z;

                let w0_x_step = (y1 - y2) as f32 * inv_denom;
                let w1_x_step = (y2 - y0) as f32 * inv_denom;

                let w0_y_step = (x2 - x1) as f32 * inv_denom;
                let w1_y_step = (x0 - x2) as f32 * inv_denom;

                let w0_seed = ((y1 - y2) * (min_x - x2) + (x2 - x1) * (min_y - y2)) as f32 * inv_denom;
                let w1_seed = ((y2 - y0) * (min_x - x2) + (x0 - x2) * (min_y - y2)) as f32 * inv_denom;

                let mut w0_row = w0_seed;
                let mut w1_row = w1_seed;

                let width = screen_size.0 as usize;

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
                            if zbuffer.test_and_set_idx(row_base + px as usize, z) {
                                buffer.push(((py as usize),(px as usize),make_cell(ch, r, g, b))); 
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
                Some(buffer)
            }

        
        fn intensity_to_char(intensity: f32) -> u8{
            // '█'
            let chars = [
            b' ', b'`', b'.', b',', b'\'', b':', b';', b'-' , b'^', 
            b'~', b'=', b'+', b'*', b'!' ,b'o', b'O', b'#', b'%', b'@', 
            ];

            let gamma = intensity.powf(0.6);
            let idx = (gamma * (chars.len() - 1) as f32) as usize;
            chars[idx.min(chars.len() - 1)]
        }

        fn calculate_lighting(normal: Vec3, light_dir: Vec3) -> f32 {
            let light_fall_off = 0.3;
            let dot = normal.dot(light_dir);
            let distance_fade = (1.0 / (1.0 + normal.z * light_fall_off)).clamp(0.1, 1.0);
            (dot * distance_fade).max(0.0)
        }

        fn approax_eq(a: usize ,b: usize,tolerance: usize ) -> bool{
            let dif = a.abs_diff(b);
            let largest = a.max(b);
            dif <= largest * tolerance 
        }

        fn get_min_x_y(pos1: (f32,f32), pos2: (f32,f32), pos3: (f32,f32)) -> (f32,f32) {
            let min_x = pos1.0.min(pos2.0).min(pos3.0);
            let min_y = pos1.1.min(pos2.1).min(pos3.1);
            (min_x, min_y)
        }

        fn get_max_x_y(pos1: (f32,f32), pos2: (f32,f32), pos3: (f32,f32)) -> (f32,f32) {
            let max_x = pos1.0.max(pos2.0).max(pos3.0);
            let max_y = pos1.1.max(pos2.1).max(pos3.1);
            (max_x, max_y)
        }


        let number_in_a_row = 5;
        let number_of_tiles = number_in_a_row*number_in_a_row;

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

        let mut which_tile: Vec<Vec<Triangle>> = vec![vec![]; number_of_tiles];

        for tri in &entity.mesh.triangles {
            // multi threading calc 
            let (t0, t1, t2) = tri.return_indices();
            let (r0, r1, r2) = match (
                self.resolve_point(&self.render_buffer[t0]),
                self.resolve_point(&self.render_buffer[t1]),
                self.resolve_point(&self.render_buffer[t2]),
            ) {
                (Some(a), Some(b), Some(c)) => (a, b, c),
                _ => continue,
            };
            // see for i / l .. (i + 1) / l
            // which I it lies in for both x and y 

            let x_min = r0.0.min(r1.0).min(r2.0);
            let x_max = r0.0.max(r1.0).max(r2.0);
            let y_min = r0.1.min(r1.1).min(r2.1);
            let y_max = r0.1.max(r1.1).max(r2.1);

            let col_min = ((x_min * number_in_a_row as f32) as usize).min(number_in_a_row - 1);
            let col_max = ((x_max * number_in_a_row as f32) as usize).min(number_in_a_row - 1);
            let row_min = ((y_min * number_in_a_row as f32) as usize).min(number_in_a_row - 1);
            let row_max = ((y_max * number_in_a_row as f32) as usize).min(number_in_a_row - 1);

            for row in row_min..=row_max {
                for col in col_min..=col_max {
                    which_tile[row * number_in_a_row + col].push(tri.clone());
                }
            }
            // norm calculations 
            // 


        }

        // calc avg per square 

        let def: num::NonZero<usize> = num::NonZero::new(1_usize).unwrap();
        let num_of_threads = std::thread::available_parallelism().unwrap_or(def).get();

        // Pre-partition tiles BEFORE spawning threads
        let thread_optimum = entity.mesh.triangles.len() / num_of_threads;
        let mut tile_assignments: Vec<Vec<&Vec<Triangle>>> = vec![Vec::new(); num_of_threads];
        let mut current_thread = 0;
        let mut count = 0;

        for tile in &which_tile {
            if current_thread + 1 < num_of_threads
                && count >= thread_optimum
            {
                current_thread += 1;
                count = 0;
            }
            tile_assignments[current_thread].push(tile);
            count += tile.len();
        }

        let screen_size = self.screen_size;
        let points = &self.render_buffer;

        std::thread::scope(|s| {
            let handles: Vec<_> = tile_assignments
                .into_iter()
                .map(|given_tiles| {
                    let camera = self.camera.clone();
                    let light_dir = self.light_dir;
                    let buf_size = screen_size.0 as usize * screen_size.1 as usize;

                    s.spawn(move || {
                        let mut final_buffer: Vec<u32> = vec![0; buf_size];

                        for tile in given_tiles {
                            for tri in tile {
                                let normal = tri.normal(points);
                                let view_dir = Vec3::new(0.0, 0.0, -1.0);
                                if normal.dot(view_dir) <= 0.0 {
                                    continue;
                                }
                                let intensity = calculate_lighting(normal, light_dir);
                                let ch = intensity_to_char(intensity);

                                let (r, g, b) = if is_color {
                                    let mat = &entity.mesh.material_color[tri.return_mat_idx()];
                                    (
                                        (mat.r as f32 * intensity) as u8,
                                        (mat.g as f32 * intensity) as u8,
                                        (mat.b as f32 * intensity) as u8,
                                    )
                                } else {
                                    (255, 255, 255) // or whatever your default is
                                };

                                let pixels = fill_triangle(tri, ch, r, g, b, screen_size, *points, camera);
                                // merge pixels into final_buffer here
                                for (idx, val) in unpack_cell(pixels){
                                    final_buffer[idx] = val;
                                }
                            }
                        }
                        final_buffer
                    })
                })
                .collect();

            // Merge all thread buffers into self.render_buffer
            for handle in handles {
                let partial = handle.join().unwrap();
                // merge partial into main buffer...
            }
        });

    }

}

