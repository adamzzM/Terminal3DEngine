mod algebra;

mod terminal;
mod render;
mod entity;


use algebra::{Vec3,Vec4};

use render::Renderer;
use entity::Entity;



fn main() {

    let mut renderer = Renderer::new();

    let mut entities: Vec<Entity> = Vec::new();

    let mut cube = Entity::new();
    let _ = cube.load_obj("obj/skull.obj");

    // cube.transform.scale_by(Vec3 { x: 0.5, y: 0.5,z: 0.5 });
    cube.transform.translate(&Vec3 { x: 0.0, y: -10.0 ,z: 30.0 });
    cube.transform.rotate(&Vec4::new(1.0,0.0,0.0,0.0));
    cube.set_color(2, 100, 50);

    entities.push(cube);

    loop {
            renderer.render(&mut entities);
            for entity in &mut entities{
                entity.transform.rotate(&Vec4::new(0.01, 0.01, 0.0, 1.0));
        }
    }
   
}
