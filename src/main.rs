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
use shapes::{Material, Plane, Shape, Sphere};
use std::f64::consts::PI;
use tuple::Tuple;
use world::{Camera, World};

fn main() {
    let floor = Plane::new(
        Material {
            colour: Colour::new(1.0, 0.9, 0.9),
            specular: 0.0,
            ..Default::default()
        },
        Matrix::identity(),
    );
    let left_wall = Sphere::new(
        Material {
            colour: Colour::new(1.0, 0.9, 0.9),
            specular: 0.0,
            ..Default::default()
        },
        Matrix::scaling(10.0, 0.01, 10.0)
            .rotate_x(PI / 2.0)
            .rotate_y(-PI / 4.0)
            .translate(0.0, 0.0, 5.0),
    );
    let right_wall = Sphere::new(
        Material {
            colour: Colour::new(1.0, 0.9, 0.9),
            specular: 0.0,
            ..Default::default()
        },
        Matrix::scaling(10.0, 0.01, 10.0)
            .rotate_x(PI / 2.0)
            .rotate_y(PI / 4.0)
            .translate(0.0, 0.0, 5.0),
    );
    let sphere = Sphere::new(
        Material {
            colour: Colour::new(0.1, 1.0, 0.5),
            diffuse: 0.7,
            specular: 0.3,
            ..Default::default()
        },
        Matrix::translation(-0.5, 1.0, 0.5),
    );
    let light = PointLight::new(
        Colour::new(1.0, 1.0, 1.0),
        Tuple::point_new(-10.0, 10.0, -10.0),
    );
    let mut world = World::new();
    world.objects = vec![
        shapes::ShapeType::Sphere(right_wall),
        shapes::ShapeType::Sphere(left_wall),
        shapes::ShapeType::Plane(floor),
        shapes::ShapeType::Sphere(sphere),
    ];
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
