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
use yaml::parse_config;

pub const REFLECTION_RECURSION_DEPTH: usize = 7;

pub fn float_eq(a: f64, b: f64) -> bool {
    const EPSILON: f64 = 0.000001;
    (a - b).abs() < EPSILON
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let yaml_file = &args[1];
    let s = std::fs::read_to_string(yaml_file).unwrap();
    let yaml = YamlLoader::load_from_str(&s).unwrap();
    let config = &yaml[0];
    let (w, mut c) = parse_config(config);
    let canv = world::render(&mut c, &w);
    canv.write_out_as_ppm_file();
    

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
