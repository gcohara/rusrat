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
}
