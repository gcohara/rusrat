#![allow(dead_code)]

mod canvas;
mod lighting;
mod matrices;
mod rays;
mod shapes;
mod tuple;
mod world;

use crate::canvas::{Canvas, Colour};
use crate::lighting::{PointLight};
use crate::matrices::Matrix;
use crate::rays::{Intersection, Ray};
use crate::shapes::{Material, Sphere};
use crate::tuple::Tuple;

fn main() {
    let ray_origin = Tuple::point_new(0.0, 0.0, -5.0);
    let wall_z = 10.0;
    let wall_size = 8.0;
    let canvas_size = 100;
    let pixel_size = wall_size / canvas_size as f64;
    let half = wall_size / 2.0;
    let mut canvas = Canvas::new(canvas_size, canvas_size);
    
    // sphere one
    let t = Matrix::translation(1.4, -1.0, 0.0).scale(0.3, 0.3, 0.3);
    let mut m = Material::default();
    m.colour = Colour::new(1.0, 0.2, 1.0);
    let s = Sphere::new(m, t);

    // sphere two
    let t = Matrix::translation(1.4, 1.0, 0.0).scale(0.3, 0.3, 0.3);
    let mut m = Material::default();
    m.colour = Colour::new(1.0, 0.2, 1.0);
    let  s2 = Sphere::new(m, t);


    // sphere three
    let t = Matrix::scaling(0.9, 0.3, 0.7).translate(-1.28, 0.0, 0.0);
    let mut m = Material::default();
    m.colour = Colour::new(1.0, 0.2, 1.0);
    let  s3 = Sphere::new(m, t);


    let light = PointLight::new(
        Colour::new(1.0, 1.0, 1.0),
        Tuple::point_new(0.0, 10.0, -15.0),
    );

    let spheres = vec![s3, s2, s];
    for x in 0..canvas_size {
        let world_x = -half + pixel_size * x as f64;
        for y in 0..canvas_size {
            let world_y = half - pixel_size * y as f64;
            let position = Tuple::point_new(world_x, world_y, wall_z);
            let r = Ray::new(ray_origin, (position - ray_origin).normalise());
            for s in spheres.iter() {
                let intersects = r.intersects(&s);
                if let Some(hit) = Intersection::hit(intersects) {
                    let hit_point = r.position(hit.t);
                    let hit_normal = s.normal_at(&hit_point);
                    let eye = r.direction.negate();
                    let clr = lighting::calculate_lighting(
                        &hit.object.material,
                        &light,
                        &hit_point,
                        &eye,
                        &hit_normal,
                    );
                    canvas.write_pixel((x, y), clr);
                }
            }
        }
    }
    canvas.write_out_as_ppm_file();
}
