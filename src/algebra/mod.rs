use std::ops::Mul;

#[derive(Clone)]
pub struct Mat4{
    m: [f32; 16]
}

impl Mat4{

    pub fn new() -> Self{
        // an identity matrix
        let mut m = [0.0; 16];
        m[0] = 1.0;
        m[5] = 1.0;
        m[10] = 1.0;
        m[15] = 1.0;
        Mat4 { m }
    }

    pub fn translation(&mut self,t: &Vec3){
        *self = Mat4::new();
        self.m[12] = t.x;
        self.m[13] = t.y;
        self.m[14] = t.z;
    }

    pub fn scale(&mut self,t: &Vec3){

        *self = Mat4::new();
        self.m[0] = t.x;
        self.m[5] =  t.y;
        self.m[10] = t.z;
        self.m[15] = 1.0;
    }

    pub fn rotation(&mut self, q: &Vec4) {
        // lord bless chatgpt
        let xx = q.x * q.x;
        let yy = q.y * q.y;
        let zz = q.z * q.z;
        let xy = q.x * q.y;
        let xz = q.x * q.z;
        let yz = q.y * q.z;
        let wx = q.w * q.x;
        let wy = q.w * q.y;
        let wz = q.w * q.z;

        self.m = [0.0; 16];
        self.m[15] = 1.0;

        self.m[0]  = 1.0 - 2.0 * (yy + zz);
        self.m[1]  = 2.0 * (xy - wz);   // was m[4]'s value
        self.m[2]  = 2.0 * (xz + wy);   // was m[8]'s value

        self.m[4]  = 2.0 * (xy + wz);   // was m[1]'s value
        self.m[5]  = 1.0 - 2.0 * (xx + zz);
        self.m[6]  = 2.0 * (yz - wx);   // was m[9]'s value

        self.m[8]  = 2.0 * (xz - wy);   // was m[2]'s value
        self.m[9]  = 2.0 * (yz + wx);   // was m[6]'s value
        self.m[10] = 1.0 - 2.0 * (xx + yy);

    }
}

impl Mul for Mat4{
    type Output = Mat4;

    fn mul(self, rhs: Mat4) -> Mat4 {
        let mut result = Mat4 { m: [0.0; 16] };

        for col in 0..4 {
            for row in 0..4 {
                result.m[col * 4 + row] =
                    self.m[0 * 4 + row] * rhs.m[col * 4 + 0] +
                    self.m[1 * 4 + row] * rhs.m[col * 4 + 1] +
                    self.m[2 * 4 + row] * rhs.m[col * 4 + 2] +
                    self.m[3 * 4 + row] * rhs.m[col * 4 + 3];
            }
        }

        result
    }

} 
impl Mul<Vec4> for Mat4 {
    type Output = Vec4;

    fn mul(self, v: Vec4) -> Vec4 {
        Vec4 {
            x: self.m[0]*v.x + self.m[4]*v.y + self.m[8]*v.z + self.m[12]*v.w,
            y: self.m[1]*v.x + self.m[5]*v.y + self.m[9]*v.z + self.m[13]*v.w,
            z: self.m[2]*v.x + self.m[6]*v.y + self.m[10]*v.z + self.m[14]*v.w,
            w: self.m[3]*v.x + self.m[7]*v.y + self.m[11]*v.z + self.m[15]*v.w,
        }
    }
}

impl Mul<Vec3> for Mat4 {
    type Output = Vec3;

    fn mul(self, v: Vec3) -> Vec3 {
        Vec3 {
            x: self.m[0]*v.x + self.m[4]*v.y + self.m[8]*v.z + self.m[12],
            y: self.m[1]*v.x + self.m[5]*v.y + self.m[9]*v.z + self.m[13],
            z: self.m[2]*v.x + self.m[6]*v.y + self.m[10]*v.z + self.m[14],
        }
    }
}

impl Mul<Vec3> for &Mat4 {
    type Output = Vec3;

    fn mul(self, v: Vec3) -> Vec3 {
        Vec3 {
            x: self.m[0]  * v.x + self.m[4]  * v.y + self.m[8]  * v.z + self.m[12],
            y: self.m[1]  * v.x + self.m[5]  * v.y + self.m[9]  * v.z + self.m[13],
            z: self.m[2]  * v.x + self.m[6]  * v.y + self.m[10] * v.z + self.m[14],
        }
    }
}


#[derive(Copy,Clone,Debug)]
pub struct Vec4{
    x: f32,
    y: f32,
    z: f32,
    w: f32
}

impl Vec4{
    pub fn new(x: f32, y: f32, z: f32,w: f32) -> Self {
        Self { x, y, z, w }
    }
    pub fn normalize(&mut self){
        let len = (self.x*self.x + self.y*self.y + self.z*self.z + self.w*self.w).sqrt().recip();
        self.x = self.x * len;
        self.y = self.y * len;
        self.z = self.z * len;
        self.w = self.w * len;
    }
    pub fn return_items(&self) -> (f32,f32,f32,f32){
        (self.x,self.y,self.z,self.w)
    }
}


impl Mul for Vec4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Vec4{
        Vec4 {
            w: self.w*rhs.w - self.x*rhs.x - self.y*rhs.y - self.z*rhs.z,
            x: self.w*rhs.x + self.x*rhs.w + self.y*rhs.z - self.z*rhs.y,
            y: self.w*rhs.y - self.x*rhs.z + self.y*rhs.w + self.z*rhs.x,
            z: self.w*rhs.z + self.x*rhs.y - self.y*rhs.x + self.z*rhs.w,
        }
    }
}


#[derive(Copy, Clone,Debug)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    // tbr
    pub fn resolve(&self,camera: &Vec3) -> Option<(f32, f32)> {

        let rel_x = self.x - camera.x;
        let rel_y= self.y - camera.y;
        let rel_z = self.z - camera.z;


        const MIN: f32 = 0.1;
        if rel_z < MIN {
            return None;
        }
        const K1: f32 = 1.2;
        let new_x = ((rel_x * K1) / rel_z) + 0.5;
        let new_y = ((rel_y * K1) / rel_z) + 0.5;
        Some((new_x, new_y))
    }
    
    pub fn dot(&self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    pub fn normalize(&mut self) {
        let len = ((self.x * self.x) + (self.y * self.y) + (self.z * self.z)).sqrt();
        self.x = self.x / len;
        self.y = self.y / len;
        self.z = self.z / len;
    }
}

#[derive(Clone)]
pub struct Triangle {
    v0: usize,
    v1: usize,
    v2: usize,
}

impl Triangle {


    pub fn new(v0: usize , v1: usize , v2: usize) -> Self{
        Self {v0,v1,v2}
    }
    pub fn return_indices(&self) -> (usize,usize,usize){
        (self.v0,self.v1,self.v2)
    }


    pub fn normal(&self,points: &Vec<Vec3>) -> Vec3 {

        let v0 = points[self.v0];
        let v1 = points[self.v1];
        let v2 = points[self.v2];

        let edge1 = Vec3::new(
            v1.x - v0.x,
            v1.y - v0.y,
            v1.z - v0.z,
        );
        let edge2 = Vec3::new(
            v2.x - v0.x,
            v2.y - v0.y,
            v2.z - v0.z,
        );


        let nx = edge1.y * edge2.z - edge1.z * edge2.y;
        let ny = edge1.z * edge2.x - edge1.x * edge2.z;
        let nz = edge1.x * edge2.y - edge1.y * edge2.x;

        let len = (nx * nx + ny * ny + nz * nz).sqrt().recip();
        Vec3::new(nx * len, ny * len, nz * len)
    }
}
