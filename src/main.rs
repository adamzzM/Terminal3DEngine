use std::{io::{self, Write}, time::Duration};
use std::thread::sleep;
use std::fs::File;
use std::io::{BufRead, BufReader};




#[derive(Copy,Clone)]
struct Pos{
    x: f32,
    y: f32,
    z: f32
}
impl Pos{
    fn new(x: f32,y: f32,z:f32) -> Self{
        Self {x,y,z}
    }

    fn resolve(&self) -> Option<(f32, f32)> {

        const MIN: f32 = 0.5;        
        if self.z < MIN{
            return None
        }

        const K1: f32 = 0.5;
        let new_x = ((self.x * K1) / self.z) + 0.5;
        let new_y = ((self.y * K1) / self.z) + 0.5;

        Some((new_x,new_y))
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
}
#[derive(Clone)]
struct Mesh {
    triangles: Vec<Triangle>,
}

impl Mesh {
    fn new() -> Self {
        Self { triangles: Vec::new() }
    }
    fn from_obj() -> Self{
        // TODO 
        Self {triangles: Vec::new()}
    }
    }

    fn shift(&mut self, offset: Pos) {
        for tri in &mut self.triangles {
            tri.v0.x += offset.x; tri.v0.y += offset.y; tri.v0.z += offset.z;
            tri.v1.x += offset.x; tri.v1.y += offset.y; tri.v1.z += offset.z;
            tri.v2.x += offset.x; tri.v2.y += offset.y; tri.v2.z += offset.z;
        }
    }

    fn rotate_x_about_axis(&mut self, angle: f32,axis: Option<Pos>) {
        let center = if axis.is_none(){
            self.get_center()
        }else{
            axis.unwrap()
        };

        for tri in &mut self.triangles {
            for v in [&mut tri.v0, &mut tri.v1, &mut tri.v2] {
                let shifted = Pos::new(v.x - center.x, v.y - center.y, v.z - center.z).rotate_x(angle);
                v.x = shifted.x + center.x;
                v.y = shifted.y + center.y;
                v.z = shifted.z + center.z;
            }
        }
    }
    fn rotate_y_about_axis(&mut self, angle: f32,axis: Option<Pos>) {
        let center = if axis.is_none(){
            self.get_center()
        }else{
            axis.unwrap()
    };

        for tri in &mut self.triangles {
            for v in [&mut tri.v0, &mut tri.v1, &mut tri.v2] {
                let shifted = Pos::new(v.x - center.x, v.y - center.y, v.z - center.z).rotate_y(angle);
                v.x = shifted.x + center.x;
                v.y = shifted.y + center.y;
                v.z = shifted.z + center.z;
            }
        }
    }
    fn rotate_z_about_axis(&mut self, angle: f32,axis: Option<Pos>) {
        let center = if axis.is_none(){
            self.get_center()
        }else{
            axis.unwrap()
    };

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
        let mut min = self.triangles[0].v0.clone();
        let mut max = self.triangles[0].v0.clone();
        for tri in &self.triangles {
            for v in [&tri.v0, &tri.v1, &tri.v2] {
                min.x = min.x.min(v.x); min.y = min.y.min(v.y); min.z = min.z.min(v.z);
                max.x = max.x.max(v.x); max.y = max.y.max(v.y); max.z = max.z.max(v.z);
            }
        }
        Pos {
            x: (min.x + max.x) / 2.0,
            y: (min.y + max.y) / 2.0,
            z: (min.z + max.z) / 2.0,
        }
    }
}
fn cube_mesh(size: f32, depth: f32) -> Mesh {
    let mut mesh = Mesh::new();

    let p = |x,y,z| Pos::new(x,y,z);

    // Cube vertices
    let v = [
        p(-size, -size, depth),      // 0
        p( size, -size, depth),      // 1
        p(-size,  size, depth),      // 2
        p( size,  size, depth),      // 3
        p(-size, -size, depth+size*3.0), // 4
        p( size, -size, depth+size*3.0), // 5
        p(-size,  size, depth+size*3.0), // 6
        p( size,  size, depth+size*3.0), // 7
    ];

    // Front face
    mesh.triangles.push(Triangle::new(v[0], v[1], v[2]));
    mesh.triangles.push(Triangle::new(v[1], v[3], v[2]));

    // Back face
    mesh.triangles.push(Triangle::new(v[4], v[6], v[5]));
    mesh.triangles.push(Triangle::new(v[5], v[6], v[7]));

    // Top face
    mesh.triangles.push(Triangle::new(v[2], v[3], v[6]));
    mesh.triangles.push(Triangle::new(v[3], v[7], v[6]));

    // Bottom face
    mesh.triangles.push(Triangle::new(v[0], v[4], v[1]));
    mesh.triangles.push(Triangle::new(v[1], v[4], v[5]));

    // Left face
    mesh.triangles.push(Triangle::new(v[0], v[2], v[4]));
    mesh.triangles.push(Triangle::new(v[2], v[6], v[4]));

    // Right face
    mesh.triangles.push(Triangle::new(v[1], v[5], v[3]));
    mesh.triangles.push(Triangle::new(v[3], v[5], v[7]));

    mesh
}


fn main() {

    const SLEEP_TIME: u64 = 700/60;

    // let's define a cube 

    // let mut cube: Object = Object::new();
    // let size = 1.0;
    // let depth = 5.0;


    // cube.vertices.push(Pos::new(-size, -size, depth));           // 0
    // cube.vertices.push(Pos::new( size, -size, depth));           // 1
    // cube.vertices.push(Pos::new(-size,  size, depth));           // 2
    // cube.vertices.push(Pos::new( size,  size, depth));           // 3
    // cube.vertices.push(Pos::new(-size, -size, depth + size * 5.0));    // 4
    // cube.vertices.push(Pos::new( size, -size, depth + size * 5.0));    // 5
    // cube.vertices.push(Pos::new(-size,  size, depth + size * 5.0));    // 6
    // cube.vertices.push(Pos::new( size,  size, depth + size * 5.0));    // 7

    // // All 12 edges...
    // cube.lines.extend_from_slice(&[
    //     (0,1), (1,3), (3,2), (2,0),
    //     (4,5), (5,7), (7,6), (6,4),
    //     (0,4), (1,5), (2,6), (3,7),
    // ]);

    let mut cube_mesh = cube_mesh(1.0, 1.0);


    let mut angle = 0.0;


    loop {
        sleep(Duration::from_millis(SLEEP_TIME));
        clear_screen();
        let term_size = get_terminal_size();
        
        // let mut cube = cube.clone();
        // cube.rotate_x_about_axis(angle, None);
        // cube.rotate_y_about_axis(angle, None);
        // cube.rotate_z_about_axis(angle, None);
        // draw dat shii
        // for item in &cube.lines {
        //     let _ = draw_line(cube.vertices[item.0], cube.vertices[item.1], '*', term_size);
        // }
        let mut mesh = cube_mesh.clone();
        mesh.rotate_x_about_axis(angle,None);
        mesh.rotate_y_about_axis(angle,None);
        draw_mesh(&mesh, term_size);
        
        angle += 0.06;  // Spin!
    }
}



fn clear_screen(){
    print!("\x1B[2J");
    print!("\x1B[?25l"); // hide cursor
}

fn draw_mesh(mesh: &Mesh, screen_size: (u16, u16)) {
    for tri in &mesh.triangles {
        draw_line(tri.v0, tri.v1, '*', screen_size);
        draw_line(tri.v1, tri.v2, '*', screen_size);
        draw_line(tri.v2, tri.v0, '*', screen_size);
    }
}





fn draw(x: f32, y: f32,ch: char,screen_size: (u16,u16)){

    fn draw_at(x: u16, y: u16, ch: char) {
        let mut stdout = io::stdout();
        write!(stdout, "\x1B[{};{}H{}", y, x, ch).unwrap();
        stdout.flush().unwrap();
    }

    if x > 1.0 || x < 0.0 || y > 1.0 || y < 0.0{
        return 
    }

    draw_at(((screen_size.0 as f32) * x) as u16,((screen_size.1 as f32) * y) as u16,ch);
}


fn draw_line(pos1: Pos, pos2: Pos, ch: char, screen_size: (u16,u16)){

    let resolved_0 = pos1.resolve();
    let resolved_1 = pos2.resolve();

    if resolved_0.is_none() || resolved_1.is_none(){
        return
    }


    let (nx0, ny0) = resolved_0.unwrap();
    let (nx1, ny1) = resolved_1.unwrap();
    
    let mut x0 = (screen_size.0 as f32 * nx0).round() as i32;
    let mut y0 = (screen_size.1 as f32 * ny0).round() as i32;
    let x1 = (screen_size.0 as f32 * nx1).round() as i32;
    let y1 = (screen_size.1 as f32 * ny1).round() as i32;
    
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    
    loop {
        let nx = x0 as f32 / screen_size.0 as f32;
        let ny = y0 as f32 / screen_size.1 as f32;
        
        draw(nx, ny, ch, screen_size);
        
        if x0 == x1 && y0 == y1 { break; }
        
        let e2 = 2 * err; 
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
    
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

        let width  = (csbi.srWindow.Right - csbi.srWindow.Left + 1) as u16;
        let height = (csbi.srWindow.Bottom - csbi.srWindow.Top + 1) as u16;

        (width, height)
    }
}