#![allow(dead_code)]

mod canvas;
mod lighting;
mod matrices;
mod rays;
mod shapes;
mod tuple;
mod world;

use canvas::Colour;
use lighting::PointLight;
use matrices::Matrix;
use shapes::{plane, sphere, Material, Shape};
use std::f64::consts::PI;
use tuple::Tuple;
use world::{Camera, World};

pub fn float_eq(a: f64, b: f64) -> bool {
    const EPSILON: f64 = 0.000001;
    (a - b).abs() < EPSILON
}

fn main() {
    let floor = Shape {
        material: Material {
            colour: Colour::new(1.0, 0.9, 0.9),
            specular: 0.0,
            ..Default::default()
        },
        ..plane::default()
    };
    let sphere = Shape {
        material: Material {
            colour: Colour::new(0.1, 1.0, 0.5),
            diffuse: 0.7,
            specular: 0.3,
            ..Default::default()
        },
        transform: Matrix::translation(-0.5, 1.0, 0.5),
        ..sphere::default()
    };
    let light = PointLight::new(
        Colour::new(1.0, 1.0, 1.0),
        Tuple::point_new(-10.0, 10.0, -10.0),
    );
    let mut world = World::new();
    world.objects = vec![floor, sphere];
    world.lights = vec![light];
    let mut cam = Camera::new(
        80,
        40,
        PI / 3.0,
        world::view_transform(
            &Tuple::point_new(0.0, 1.5, -5.0),
            &Tuple::point_new(0.0, 1.0, 0.0),
            &Tuple::vector_new(0.0, 1.0, 0.0),
        ),
    );
    let canv = world::render(&mut cam, &world);
    canv.write_out_as_ppm_file();
}
