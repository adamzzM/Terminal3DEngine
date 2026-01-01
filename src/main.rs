use std::{io::{self, Write}, time::Duration};
use std::thread::sleep;




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
struct Object{
    vertices: Vec<Pos>,
    lines: Vec<(usize,usize)>
}
impl Object{
    fn new() -> Self{
        let vertices : Vec<Pos> = Vec::new();
        let lines: Vec<(usize,usize)> = Vec::new();
        Self {
            vertices,
            lines
        }
    }
    fn shift(&mut self,vector: Pos){
        for item in &mut self.vertices{
            item.x += vector.x;
            item.y += vector.y;
            item.z += vector.z;
        }
    }
}



fn main() {

    const SLEEP_TIME: u64 = 1000/60;

    //let mut points: Vec<Pos> = Vec::new();

//    points.push(Pos::new(0.1, 0.1, 0.2));
  //  points.push(Pos::new(0.9, 0.9, 0.2));

    // let's define a cube 

    let mut cube: Object = Object::new();
    let size = 1.0;
    let depth = 5.0;


    cube.vertices.push(Pos::new(-size, -size, depth));           // 0
    cube.vertices.push(Pos::new( size, -size, depth));           // 1
    cube.vertices.push(Pos::new(-size,  size, depth));           // 2
    cube.vertices.push(Pos::new( size,  size, depth));           // 3
    cube.vertices.push(Pos::new(-size, -size, depth + size * 3.0));    // 4
    cube.vertices.push(Pos::new( size, -size, depth + size * 3.0));    // 5
    cube.vertices.push(Pos::new(-size,  size, depth + size * 3.0));    // 6
    cube.vertices.push(Pos::new( size,  size, depth + size * 3.0));    // 7

    // All 12 edges...
    cube.lines.extend_from_slice(&[
        (0,1), (1,3), (3,2), (2,0),
        (4,5), (5,7), (7,6), (6,4),
        (0,4), (1,5), (2,6), (3,7),
    ]);


    let mut angle = 0.0;


    loop {
        sleep(Duration::from_millis(SLEEP_TIME));
        clear_screen();
        let term_size = get_terminal_size();
        
        let mut cube = cube.clone();
        for vertex in &mut cube.vertices {
            *vertex = vertex.rotate_y(angle);
        }
        
        // draw dat shii
        for item in &cube.lines {
            let _ = draw_line(cube.vertices[item.0], cube.vertices[item.1], '.', term_size);
        }
        
        angle += 0.05;  // Spin!
    }
}



fn clear_screen(){
    print!("\x1B[2J");
    print!("\x1B[?25l"); // hide cursor
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