
use std::{io::{self, Write}, time::Duration};
use std::thread::sleep;
use std::fs::File;
use std::io::BufRead;
use std::path::Path;
use std::time::Instant;


#[derive(Clone)]
struct ZBuffer {
    depths: Vec<f32>,
    width: usize,
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

}

#[derive(Copy, Clone)]
struct Pos {
    x: f32,
    y: f32,
    z: f32,
}

impl Pos {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn resolve(&self) -> Option<(f32, f32)> {
        const MIN: f32 = 0.5;
        if self.z < MIN {
            return None;
        }
        const K1: f32 = 0.5;
        let new_x = ((self.x * K1) / self.z) + 0.5;
        let new_y = ((self.y * K1) / self.z) + 0.5;
        Some((new_x, new_y))
    }

    fn rotate_y(&self, angle: f32) -> Pos {
        let cos = angle.cos();
        let sin = angle.sin();
        Pos {
            x: self.x * cos + self.z * sin,
            y: self.y,
            z: -self.x * sin + self.z * cos,
        }
    }

    fn rotate_z(&self, angle: f32) -> Pos {
        let cos = angle.cos();
        let sin = angle.sin();
        Pos {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
            z: self.z,
        }
    }

    fn rotate_x(&self, angle: f32) -> Pos {
        let cos = angle.cos();
        let sin = angle.sin();
        Pos {
            x: self.x,
            y: self.y * cos - self.z * sin,
            z: self.y * sin + self.z * cos,
        }
    }
}

#[derive(Clone)]
struct Triangle {
    v0: Pos,
    v1: Pos,
    v2: Pos,
}

impl Triangle {
    fn new(v0: Pos, v1: Pos, v2: Pos) -> Self {
        Self { v0, v1, v2 }
    }

    fn normal(&self) -> Pos {
        let edge1 = Pos::new(
            self.v1.x - self.v0.x,
            self.v1.y - self.v0.y,
            self.v1.z - self.v0.z,
        );
        let edge2 = Pos::new(
            self.v2.x - self.v0.x,
            self.v2.y - self.v0.y,
            self.v2.z - self.v0.z,
        );
        let nx = edge1.y * edge2.z - edge1.z * edge2.y;
        let ny = edge1.z * edge2.x - edge1.x * edge2.z;
        let nz = edge1.x * edge2.y - edge1.y * edge2.x;
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        Pos::new(nx / len, ny / len, nz / len)
    }
}

#[derive(Clone)]
struct Mesh {
    triangles: Vec<Triangle>,
}

impl Mesh {
    fn new() -> Self {
        Self {
            triangles: Vec::new(),
        }
    }

    fn from_obj(path: &str) -> io::Result<Self> {
        let file = File::open(Path::new(path))?;
        let reader = io::BufReader::new(file);

        let mut vertices: Vec<Pos> = Vec::new();
        let mut triangles: Vec<Triangle> = Vec::new();

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
                    vertices.push(Pos::new(x, y, z));
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
                        triangles.push(Triangle::new(
                            vertices[indices[0]],
                            vertices[indices[1]],
                            vertices[indices[2]],
                        ));
                    } else if indices.len() == 4 {
                        triangles.push(Triangle::new(
                            vertices[indices[0]],
                            vertices[indices[1]],
                            vertices[indices[2]],
                        ));
                        triangles.push(Triangle::new(
                            vertices[indices[0]],
                            vertices[indices[2]],
                            vertices[indices[3]],
                        ));
                    }
                }
                _ => {}
            }
        }
        Ok(Self { triangles })
    }

    fn shift(&mut self, offset: Pos) {
        for tri in &mut self.triangles {
            tri.v0.x += offset.x;
            tri.v0.y += offset.y;
            tri.v0.z += offset.z;
            tri.v1.x += offset.x;
            tri.v1.y += offset.y;
            tri.v1.z += offset.z;
            tri.v2.x += offset.x;
            tri.v2.y += offset.y;
            tri.v2.z += offset.z;
        }
    }

    fn rotate_x_about_axis(&mut self, angle: f32, axis: Option<Pos>) {
        let center = axis.unwrap_or_else(|| self.get_center());
        for tri in &mut self.triangles {
            for v in [&mut tri.v0, &mut tri.v1, &mut tri.v2] {
                let shifted = Pos::new(v.x - center.x, v.y - center.y, v.z - center.z).rotate_x(angle);
                v.x = shifted.x + center.x;
                v.y = shifted.y + center.y;
                v.z = shifted.z + center.z;
            }
        }
    }

    fn rotate_y_about_axis(&mut self, angle: f32, axis: Option<Pos>) {
        let center = axis.unwrap_or_else(|| self.get_center());
        for tri in &mut self.triangles {
            for v in [&mut tri.v0, &mut tri.v1, &mut tri.v2] {
                let shifted = Pos::new(v.x - center.x, v.y - center.y, v.z - center.z).rotate_y(angle);
                v.x = shifted.x + center.x;
                v.y = shifted.y + center.y;
                v.z = shifted.z + center.z;
            }
        }
    }

    fn rotate_z_about_axis(&mut self, angle: f32, axis: Option<Pos>) {
        let center = axis.unwrap_or_else(|| self.get_center());
        for tri in &mut self.triangles {
            for v in [&mut tri.v0, &mut tri.v1, &mut tri.v2] {
                let shifted = Pos::new(v.x - center.x, v.y - center.y, v.z - center.z).rotate_z(angle);
                v.x = shifted.x + center.x;
                v.y = shifted.y + center.y;
                v.z = shifted.z + center.z;
            }
        }
    }

    fn get_center(&self) -> Pos {
        let mut min = self.triangles[0].v0;
        let mut max = self.triangles[0].v0;
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
        Pos {
            x: (min.x + max.x) / 2.0,
            y: (min.y + max.y) / 2.0,
            z: (min.z + max.z) / 2.0,
        }
    }
}


fn intensity_to_char(intensity: f32) -> char {
    let chars = ['.', ':', '-', '=', '+', '*', '#', '%', '@'];
    let idx = (intensity * (chars.len() - 1) as f32) as usize;
    chars[idx.min(chars.len() - 1)]
}

fn main() {
    const TARGET_FPS: u64 = 60;
    const FRAME_TIME: Duration = Duration::from_millis(1000 / TARGET_FPS);

    let mut term_size = get_terminal_size();

    let mut buffer: Vec<char> = vec![' '; term_size.0 as usize * term_size.1 as usize];


    let mut cube = Mesh::from_obj("obj/horse.obj").unwrap();
    cube.shift(Pos::new(0.0,10.0,10.0));
    //cube.rotate_z_about_axis(90.0, None);
    //cube.rotate_x_about_axis(70.0, None);
    let mut angle = 0.0;

    loop {

        if term_size != get_terminal_size() {
            term_size = get_terminal_size();
            buffer = vec![' '; term_size.0 as usize * term_size.1 as usize];
        }

        let frame_start = Instant::now();
        let mut z_buffer = ZBuffer::new(&term_size);


        render_buffer(&buffer, term_size);
        buffer.fill(' '); 


        let mut mesh = cube.clone();
        mesh.rotate_x_about_axis(angle, None);
        mesh.rotate_y_about_axis(angle, None);
        draw_mesh(&mesh, term_size, &mut z_buffer,&mut buffer);

        angle += 0.06;

        let frame_elapsed = frame_start.elapsed();
        if frame_elapsed < FRAME_TIME {
            sleep(FRAME_TIME - frame_elapsed);
        }
    }
}


fn draw_mesh(mesh: &Mesh, screen_size: (u16, u16), z_buffer: &mut ZBuffer,buffer: &mut Vec<char>) {
    let light_dir = Pos::new(0.5, -0.5, -1.0);
    let len = (light_dir.x * light_dir.x + light_dir.y * light_dir.y + light_dir.z * light_dir.z).sqrt();
    let light_dir = Pos::new(light_dir.x / len, light_dir.y / len, light_dir.z / len);

    for tri in &mesh.triangles {
        let normal = tri.normal();
        
        // Backface culling - only draw if facing camera
        let view_dir = Pos::new(0.0, 0.0, -1.0);
        let dot = normal.x * view_dir.x + normal.y * view_dir.y + normal.z * view_dir.z;
        if dot <= 0.0 {
            continue;
        }

        let intensity = calculate_lighting(normal, light_dir);
        let ch = intensity_to_char(intensity);
        
        fill_triangle(tri, ch, screen_size, z_buffer,buffer);
    }
}

fn calculate_lighting(normal: Pos, light_dir: Pos) -> f32 {
    let dot = normal.x * light_dir.x + normal.y * light_dir.y + normal.z * light_dir.z;
    dot.max(0.0)
}

fn fill_triangle(tri: &Triangle, ch: char, screen_size: (u16, u16), z_buffer: &mut ZBuffer,buffer: &mut Vec<char>) {
    let r0 = tri.v0.resolve();
    let r1 = tri.v1.resolve();
    let r2 = tri.v2.resolve();

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


#[cfg(unix)]
fn get_terminal_size() -> (u16, u16) {
    use libc::{ioctl, winsize, STDOUT_FILENO, TIOCGWINSZ};
    unsafe {
        let mut ws: winsize = std::mem::zeroed();
        ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut ws);
        (ws.ws_col, ws.ws_row)
    }
}

#[cfg(windows)]
fn get_terminal_size() -> (u16, u16) {
    use winapi::um::wincon::{GetConsoleScreenBufferInfo, CONSOLE_SCREEN_BUFFER_INFO};
    use winapi::um::processenv::GetStdHandle;
    use winapi::um::winbase::STD_OUTPUT_HANDLE;
    use std::mem::zeroed;
    unsafe {
        let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = zeroed();
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        GetConsoleScreenBufferInfo(handle, &mut csbi);
        let width = (csbi.srWindow.Right - csbi.srWindow.Left + 1) as u16;
        let height = (csbi.srWindow.Bottom - csbi.srWindow.Top + 1) as u16;
        (width, height)
    }
}
