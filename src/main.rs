#![allow(dead_code)]

mod canvas;
mod lighting;
mod matrices;
mod rays;
mod shapes;
mod tuple;
mod world;
mod yaml;


use yaml_rust::YamlLoader;
use canvas::Colour;
use lighting::PointLight;
use matrices::Matrix;
use shapes::{plane, sphere, CheckPattern3D, Material, Shape};
use std::f64::consts::PI;
use tuple::Tuple;
use world::{Camera, World};

pub const REFLECTION_RECURSION_DEPTH: usize = 7;

pub fn float_eq(a: f64, b: f64) -> bool {
    const EPSILON: f64 = 0.000001;
    (a - b).abs() < EPSILON
}

// fn parse_yaml() -> World {}

fn main() {
    let s = std::fs::read_to_string("scene1.yaml").unwrap();
    let yaml = YamlLoader::load_from_str(&s).unwrap();
    for thing in yaml {
        println!("{:#?}", thing);
        println!("Another thing...");
    }
    
    // let s = YamlLoader
    
    let floor = Shape {
        material: Material {
            colour: Colour::new(0.1, 0.1, 0.1),
            specular: 0.0,
            reflectivity: 0.3,
            pattern: Some(Box::new(CheckPattern3D {
                colour_a: Colour::new(0.1, 0.1, 0.1),
                colour_b: Colour::new(0.9, 0.9, 0.9),
                transform: Matrix::rotation_y(PI / 6.0),
            })),
            ..Default::default()
        },
        ..plane::default()
    };
    let left_wall = Shape {
        material: Material {
            specular: 1.0,
            reflectivity: 0.95,
            colour: Colour::new(0.0, 0.0, 0.0),
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
            reflectivity: 0.9,
            transparency: 0.2,
            colour: Colour::new(0.1, 0.1, 0.1),
            ..Default::default()
        },
        transform: Matrix::rotation_x(PI / 2.0)
            .rotate_y(PI / 4.0)
            .translate(0.0, 0.0, 5.0),
        ..plane::default()
    };

    let sphere_glass = Shape {
        material: Material {
            colour: Colour::new(0.9, 1.0, 1.0),
            diffuse: 0.0,
            specular: 0.9,
            transparency: 0.9,
            reflectivity: 0.9,
            refractive_index: 1.5,
            ..Default::default()
        },
        transform: Matrix::translation(0.0, 3.0, -8.0).scale(0.5, 0.5, 0.5),
        ..sphere::default()
    };
    let inner_ball = Shape {
        material: Material {
            colour: Colour::new(1.0, 1.0, 1.0),
            reflectivity: 0.9,
            ambient: 0.0,
            diffuse: 0.0,
            specular: 0.9,
            shininess: 300.0,
            transparency: 0.9,
            refractive_index: 1.00000034,
            ..Default::default()
        },
        transform: Matrix::translation(0.0, 3.0, -8.0).scale(0.3, 0.3, 0.3),
        ..Default::default()
    };
    let sphere = Shape {
        material: Material {
            colour: Colour::new(0.3, 0.1, 0.1),
            diffuse: 0.7,
            specular: 0.6,
            transparency: 0.0,
            reflectivity: 0.1,
            refractive_index: 1.5,
            ..Default::default()
        },
        transform: Matrix::translation(-0.5, 1.0, 0.5),
        ..sphere::default()
    };
    let mirror_ball = Shape {
        material: Material {
            colour: Colour::new(0.09, 0.09, 0.09),
            reflectivity: 0.9,
            ..Default::default()
        },
        transform: Matrix::translation(2.0, 2.0, 0.0),
        ..Default::default()
    };
    let light = PointLight::new(
        Colour::new(1.0, 1.0, 1.0),
        Tuple::point_new(-10.0, 10.0, -10.0),
    );
    let mut world = World::new();
    world.objects = vec![
        floor,
        sphere,
        left_wall,
        mirror_ball,
        sphere_glass,
        inner_ball,
    ];
    world.lights = vec![light];
    let mut cam = Camera::new(
        1000,
        1000,
        PI / 4.5,
        world::view_transform(
            &Tuple::point_new(0.0, 3.1, -10.3),
            &Tuple::point_new(0.0, 1.0, 0.0),
            &Tuple::vector_new(0.0, 1.0, 0.0),
        ),
    );
    // let canv = world::render(&mut cam, &world);
    // canv.write_out_as_ppm_file();

    ////// glass ball

    // let light = PointLight::new(
    //     Colour::new(0.9,0.9, 0.9),
    //     Tuple::point_new(2.0, 10.0, -5.0),
    // );
    // let mut cam = Camera::new(
    //     100,
    //     100,
    //     0.45,
    //     world::view_transform(
    //          &Tuple::point_new(0.0, 0.0, -5.),
    //         &Tuple::point_new(0.0, 0.0, 0.0),
    //         &Tuple::vector_new(0.0, 1.0, 0.0),
    //     ),
    // );

    // let floor = Shape {
    //     material: Material {
    //         ambient: 0.8,
    //         specular: 0.0,
    //         diffuse: 0.2,
    //         colour: Colour::new(0.1, 0.1, 0.1),
    //         pattern: Some(Box::new(CheckPattern3D {
    //             colour_a: Colour::new(0.15, 0.15, 0.15),
    //             colour_b: Colour::new(0.85,0.85,0.85),
    //             ..Default::default()
    //         })),
    //         ..Default::default()
    //     },
    //     transform: Matrix::rotation_x(PI / 2.0)
    //         .translate(0.0, 0.0, 10.0),
    //     ..plane::default()
    // };

    // let sphere = Shape {
    //     material: Material {
    //         colour: Colour::new(0.9, 1.0, 1.0),
    //         diffuse: 0.0,
    //         specular: 0.9,
    //         transparency:0.9,
    //         reflectivity: 0.9,
    //         refractive_index: 1.5,
    //         ..Default::default()
    //     },
    //     ..sphere::default()
    // };
    // let inner_ball = Shape {
    //     material: Material {
    //         colour: Colour::new(1.0, 1.0, 1.0),
    //         reflectivity: 0.9,
    //         ambient: 0.0,
    //         diffuse: 0.0,
    //         specular: 0.9,
    //         shininess:300.0,
    //         transparency:0.9,
    //         refractive_index: 1.00000034,
    //         ..Default::default()
    //     },
    //     transform: Matrix::scaling(0.5,0.5,0.5),
    //     ..Default::default()
    // };
    // let mut world = World::new();
    // world.objects = vec![floor, sphere, inner_ball];
    // world.lights = vec![light];
    // let canv = world::render(&mut cam, &world);
    // canv.write_out_as_ppm_file();
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn yaml_sphere() {
//         let yaml = "- add: plane
//   material:
//     colour: [1,1,1]
//     ambient: 1
//     diffuse: 0
//     specular: 0
//   transform:
//     - [rotate-x, 1.5707]
//     - [translate, 0, 0, 500]";
//     }
// }
