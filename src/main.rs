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

pub const REFLECTION_RECURSION_DEPTH: usize = 7;

pub fn float_eq(a: f64, b: f64) -> bool {
    const EPSILON: f64 = 0.000001;
    (a - b).abs() < EPSILON
}

fn main() {
    let floor = Shape {
        material: Material {
            colour: Colour::new(0.9, 0.9, 0.9),
            specular: 0.0,
            reflectivity: 0.0,
            ..Default::default()
        },
        ..plane::default()
    };
    let left_wall = Shape {
        material: Material {
            specular: 1.0,
            reflectivity: 0.95,
            colour: Colour::new(0.0,0.0,0.0),
            diffuse: 0.1,
            ambient: 0.1,
            ..Default::default()
        },
        transform: Matrix::rotation_x(PI / 2.0)
            .rotate_y(-PI / 4.0)
            .translate(0.0, 0.0, 5.0),
        ..plane::default()
    };
    let right_wall = Shape {
        material: Material {
            specular: 0.0,
            reflectivity: 0.01,
            colour: Colour::new(0.55, 0.1, 0.1),
            ..Default::default()
        },
        transform: Matrix::rotation_x(PI / 2.0)
            .rotate_y(PI / 4.0)
            .translate(0.0, 0.0, 5.0),
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
    world.objects = vec![floor, sphere, left_wall, right_wall];
    world.lights = vec![light];
    let mut cam = Camera::new(
        500,
        500,
        PI / 4.5,
        world::view_transform(
            &Tuple::point_new(0.0, 3.1, -10.3),
            &Tuple::point_new(0.0, 1.0, 0.0),
            &Tuple::vector_new(0.0, 1.0, 0.0),
        ),
    );
    let canv = world::render(&mut cam, &world);
    canv.write_out_as_ppm_file();
}
