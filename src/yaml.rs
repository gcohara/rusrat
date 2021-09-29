use crate::canvas::Colour;
use crate::lighting::PointLight;
use crate::matrices::Matrix;
use crate::shapes::{Material, Pattern, Shape, ShapeType};
use crate::tuple::Tuple;
use crate::world::{self, Camera, World};
use yaml_rust::{yaml, Yaml};

enum EntityKind {
    Camera,
    Light,
    Plane,
    Sphere,
}

enum TupleKind {
    Vector,
    Point,
}

#[derive(Debug, PartialEq)]
enum TransformType {
    RotateX(f64),
    RotateY(f64),
    RotateZ(f64),
    Translate(f64, f64, f64),
    Scale(f64, f64, f64),
}

pub fn parse_config(config: &yaml::Yaml) -> (World, Camera) {
    let mut w = World::new();
    let mut c = Camera::default();
    // iterate over the structures
    if let Yaml::Array(entities) = config {
        for node in entities {
            if let Yaml::Hash(entity) = node {
                match entity_kind(entity) {
                    EntityKind::Camera => c = camera_from_config(node),
                    EntityKind::Light => w.lights.push(light_from_config(node)),
                    EntityKind::Plane | EntityKind::Sphere => {
                        w.objects.push(shape_from_config(node))
                    }
                };
            }
        }
    }
    (w, c)
}

// this function assumes that it's being given a Yaml::Hash whose "add" field is "camera"
// it will panic otherwise

fn camera_from_config(cam_yaml: &yaml::Yaml) -> world::Camera {
    if let Yaml::Hash(_cam_config) = cam_yaml {
        let from = destructure_yaml_array_into_tuple(&cam_yaml["from"], TupleKind::Point);
        let to = destructure_yaml_array_into_tuple(&cam_yaml["to"], TupleKind::Point);
        let up = destructure_yaml_array_into_tuple(&cam_yaml["up"], TupleKind::Vector);
        world::Camera::new(
            cam_yaml["width"].as_i64().unwrap() as usize,
            cam_yaml["height"].as_i64().unwrap() as usize,
            cam_yaml["field-of-view"].as_f64().unwrap(),
            world::view_transform(&from, &to, &up),
        )
    } else {
        unreachable!()
    }
}

fn shape_from_config(shape_yaml: &yaml::Yaml) -> Shape {
    if let Yaml::Hash(_) = shape_yaml {
        let mut out = Shape::default();
        if let Yaml::Array(_) = shape_yaml["transform"] {
            out.transform = parse_transforms(&shape_yaml["transform"]);
        };
        if let Yaml::Hash(_) = shape_yaml["material"] {
            out.material = parse_material(&shape_yaml["material"]);
        };
        out.shape = match &shape_yaml["add"] {
            Yaml::String(kind) if kind == "sphere" => ShapeType::Sphere,
            Yaml::String(kind) if kind == "plane" => ShapeType::Plane,
            _ => panic!(),
        };
        out
    } else {
        unreachable!()
    }
}

// assume that it's being given a Yaml::Hash whose "add" field is "light"

fn light_from_config(light_yaml: &yaml::Yaml) -> PointLight {
    if let Yaml::Hash(_) = light_yaml {
        let at = destructure_yaml_array_into_tuple(&light_yaml["at"], TupleKind::Point);
        let intensity = destructure_yaml_array_into_colour(&light_yaml["intensity"]);
        PointLight::new(intensity, at)
    } else {
        unreachable!()
    }
}

fn parse_transforms(transform_array: &yaml::Yaml) -> Matrix<f64, 4, 4> {
    if let Yaml::Array(ts) = transform_array {
        let mut out = Matrix::identity();
        for transform in ts.iter().rev() {
            out = out
                * match transform_type_and_data(transform) {
                    TransformType::RotateX(a) => Matrix::rotation_x(a),
                    TransformType::RotateY(a) => Matrix::rotation_y(a),
                    TransformType::RotateZ(a) => Matrix::rotation_z(a),
                    TransformType::Scale(x, y, z) => Matrix::scaling(x, y, z),
                    TransformType::Translate(x, y, z) => Matrix::translation(x, y, z),
                };
        }
        out
    } else {
        unreachable!()
    }
}

// should be given a &Yaml::Array, which looks like ["rotate-x", 1]

fn transform_type_and_data(transform: &yaml::Yaml) -> TransformType {
    match &transform[0] {
        Yaml::String(s) if s == "rotate-x" => TransformType::RotateX(parse_number(&transform[1])),
        Yaml::String(s) if s == "rotate-y" => TransformType::RotateY(parse_number(&transform[1])),
        Yaml::String(s) if s == "rotate-z" => TransformType::RotateZ(parse_number(&transform[1])),
        Yaml::String(s) if s == "translate" => TransformType::Translate(
            parse_number(&transform[1]),
            parse_number(&transform[2]),
            parse_number(&transform[3]),
        ),
        Yaml::String(s) if s == "scale" => TransformType::Scale(
            parse_number(&transform[1]),
            parse_number(&transform[2]),
            parse_number(&transform[3]),
        ),
        Yaml::String(s) => panic!("String {} is not a valid transform", s),
        _ => {
            println!(
                "Value {:?} is not a valid transform. Please check the yaml file for errors.",
                &transform[0]
            );
            unreachable!()
        }
    }
}

// must only be passed a Yaml::Integer or Yaml::Real.
// returns the number within as an f64

fn parse_number(num: &yaml::Yaml) -> f64 {
    match num {
        Yaml::Integer(x) => *x as f64,
        Yaml::Real(x) => x.parse().unwrap(),
        _ => unreachable!(),
    }
}

// expects to be given a Yaml::Hash, which maps the properties of the material
// e.g "colour" onto their appropriate yaml::Yaml variants.

fn parse_material(material: &yaml::Yaml) -> Material {
    let mut out = Material::default();
    if material["colour"] != Yaml::BadValue {
        out.colour = destructure_yaml_array_into_colour(&material["colour"]);
    } else if material["color"] != Yaml::BadValue {
        out.colour = destructure_yaml_array_into_colour(&material["color"]);
    }
    if material["ambient"] != Yaml::BadValue {
        out.ambient = parse_number(&material["ambient"]);
    }
    if material["diffuse"] != Yaml::BadValue {
        out.diffuse = parse_number(&material["diffuse"]);
    }
    if material["specular"] != Yaml::BadValue {
        out.specular = parse_number(&material["specular"]);
    }
    if material["shininess"] != Yaml::BadValue {
        out.shininess = parse_number(&material["shininess"]);
    }
    if material["reflectivity"] != Yaml::BadValue {
        out.reflectivity = parse_number(&material["reflectivity"]);
    }
    if material["transparency"] != Yaml::BadValue {
        out.transparency = parse_number(&material["transparency"]);
    }
    if material["refractive_index"] != Yaml::BadValue {
        out.refractive_index = parse_number(&material["refractive_index"]);
    }
    if material["pattern"] != Yaml::BadValue {
        out.pattern = Some(parse_pattern(&material["pattern"]));
    }
    out
}

// expects to be given a Yaml::Hash, which contains the type of pattern and
// the relevant colours and transform etc

fn parse_pattern(pattern_map: &yaml::Yaml) -> Pattern {
    match &pattern_map["type"] {
        Yaml::String(s) if s == "3d-check" => parse_check_pattern(pattern_map),
        Yaml::String(s) if s == "stripe" => parse_stripe_pattern(pattern_map),
        _ => unreachable!(),
    }
}

fn parse_check_pattern(pattern_map: &yaml::Yaml) -> Pattern {
    let colour_a = if pattern_map["colour-a"] != Yaml::BadValue {
        destructure_yaml_array_into_colour(&pattern_map["colour-a"])
    } else if pattern_map["color-a"] != Yaml::BadValue {
        destructure_yaml_array_into_colour(&pattern_map["color-a"])
    } else {
        unreachable!();
    };

    let colour_b = if pattern_map["colour-b"] != Yaml::BadValue {
        destructure_yaml_array_into_colour(&pattern_map["colour-b"])
    } else if pattern_map["color-a"] != Yaml::BadValue {
        destructure_yaml_array_into_colour(&pattern_map["color-b"])
    } else {
        unreachable!();
    };

    let transform = if pattern_map["transform"] != Yaml::BadValue {
        parse_transforms(&pattern_map["transform"])
    } else {
        unreachable!();
    };
    Pattern::Check3D {
        colour_a,
        colour_b,
        transform,
    }
}

fn parse_stripe_pattern(pattern_map: &yaml::Yaml) -> Pattern {
    let colour_a = if pattern_map["colour-a"] != Yaml::BadValue {
        destructure_yaml_array_into_colour(&pattern_map["colour-a"])
    } else if pattern_map["color-a"] != Yaml::BadValue {
        destructure_yaml_array_into_colour(&pattern_map["color-a"])
    } else {
        unreachable!();
    };

    let colour_b = if pattern_map["colour-b"] != Yaml::BadValue {
        destructure_yaml_array_into_colour(&pattern_map["colour-b"])
    } else if pattern_map["color-a"] != Yaml::BadValue {
        destructure_yaml_array_into_colour(&pattern_map["color-b"])
    } else {
        unreachable!();
    };

    let transform = if pattern_map["transform"] != Yaml::BadValue {
        parse_transforms(&pattern_map["transform"])
    } else {
        unreachable!();
    };
    Pattern::Stripe {
        colour_a,
        colour_b,
        transform,
    }
}

fn destructure_yaml_array_into_tuple(array: &yaml::Yaml, kind: TupleKind) -> Tuple {
    if let Yaml::Array(a) = array {
        let mut tuple_as_array: [f64; 3] = [0.0; 3];
        for i in 0..3 {
            tuple_as_array[i] = match &a[i] {
                Yaml::Integer(val) => *val as f64,
                Yaml::Real(val) => val.parse().unwrap(),
                _ => {
                    println!("Value {:?} is not a valid number!", &a[i]);
                    panic!()
                }
            }
        }
        let [x, y, z] = tuple_as_array;
        match kind {
            TupleKind::Vector => Tuple::vector_new(x, y, z),
            TupleKind::Point => Tuple::point_new(x, y, z),
        }
    } else {
        unreachable!()
    }
}

fn destructure_yaml_array_into_colour(array: &yaml::Yaml) -> Colour {
    if let Yaml::Array(a) = array {
        let mut colour_as_array: [f64; 3] = [0.0; 3];
        for i in 0..3 {
            colour_as_array[i] = match &a[i] {
                Yaml::Integer(val) => *val as f64,
                Yaml::Real(val) => val.parse().unwrap(),
                _ => panic!(),
            }
        }
        let [r, g, b] = colour_as_array;
        Colour::new(r, g, b)
    } else {
        unreachable!()
    }
}

fn entity_kind(entity: &yaml::Hash) -> EntityKind {
    let s = entity.get(&Yaml::String("add".to_string())).unwrap();
    match s {
        Yaml::String(kind) if kind == "sphere" => EntityKind::Sphere,
        Yaml::String(kind) if kind == "plane" => EntityKind::Plane,
        Yaml::String(kind) if kind == "camera" => EntityKind::Camera,
        Yaml::String(kind) if kind == "light" => EntityKind::Light,
        _ => panic!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes;

    #[test]
    fn reads_in_camera() {
        let yaml_file = "
- add: camera
  width: 100
  height: 100
  field-of-view: 0.785
  from: [ 1, 3, 2 ]
  to: [4, -2, 8]
  up: [1, 1, 0]
";
        let config = &yaml::YamlLoader::load_from_str(yaml_file).unwrap()[0][0];
        let cam = camera_from_config(config);
        let expected = world::Camera::new(
            100,
            100,
            0.785,
            Matrix::from_array(&[
                [-0.50709, 0.50709, 0.67612, -2.36643],
                [0.76772, 0.60609, 0.12122, -2.82843],
                [-0.35857, 0.59761, -0.71714, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]),
        );
        assert_eq!(cam, expected);
    }

    #[test]
    fn reads_in_light() {
        let yaml_file = "
- add: light
  at: [50, 100, -50]
  intensity: [1, 1, 0.2]
";
        let config = &yaml::YamlLoader::load_from_str(yaml_file).unwrap()[0][0];
        let light = light_from_config(config);
        let expected = PointLight::new(
            Colour::new(1.0, 1.0, 0.2),
            Tuple::point_new(50.0, 100.0, -50.0),
        );
        assert_eq!(light, expected);
    }

    #[test]
    fn reads_in_a_rotation() {
        let yaml_transform = "
[rotate-x, 0.345]
    ";
        let config = &yaml::YamlLoader::load_from_str(yaml_transform).unwrap()[0];
        let transform = transform_type_and_data(config);
        assert_eq!(transform, TransformType::RotateX(0.345));
    }

    #[test]
    fn reads_in_a_translation() {
        let yaml_transform = "
[translate, 0.345, 5, 7.5]
    ";
        let config = &yaml::YamlLoader::load_from_str(yaml_transform).unwrap()[0];
        let transform = transform_type_and_data(config);
        assert_eq!(transform, TransformType::Translate(0.345, 5.0, 7.5));
    }

    #[test]
    fn reads_in_several_transforms() {
        let yaml_transforms = "
transform:
  - [rotate-x, 1.57079632679]
  - [scale, 5, 5, 5]
  - [translate, 10, 5, 7]
";
        let config = &yaml::YamlLoader::load_from_str(yaml_transforms).unwrap()[0];
        let transform = parse_transforms(&config["transform"]);
        let expected = Matrix::from_array(&[
            [5.0, 0.0, 0.0, 10.0],
            [0.0, 0.0, -5.0, 5.0],
            [0.0, 5.0, 0.0, 7.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        assert_eq!(transform, expected);
    }

    #[test]
    fn reads_in_a_sphere() {
        let yaml_sphere = "
- add: sphere
  material:
    colour: [1,1,1]
    ambient: 1
    diffuse: 0
    specular: 0
  transform:
    - [ rotate-x, 1.57079632679]
    - [translate, 0, 0, 500]
";
        let config = &yaml::YamlLoader::load_from_str(yaml_sphere).unwrap()[0][0];
        dbg!(config);
        let sphere = shape_from_config(config);
        let expected = shapes::Shape {
            material: Material {
                colour: Colour::new(1.0, 1.0, 1.0),
                ambient: 1.0,
                diffuse: 0.0,
                specular: 0.0,
                ..Default::default()
            },
            transform: Matrix::rotation_x(1.57079632679).translate(0.0, 0.0, 500.0),
            ..Default::default()
        };
        assert_eq!(sphere, expected);
    }

    #[test]
    fn reads_in_a_world() {}

    #[test]
    fn reads_in_a_sphere_with_no_transform() {
        let yaml_sphere = "
- add: sphere
  material:
    colour: [1,1,1]
    ambient: 1
    diffuse: 0
    specular: 0
";
        let config = &yaml::YamlLoader::load_from_str(yaml_sphere).unwrap()[0][0];
        dbg!(config);
        let sphere = shape_from_config(config);
        let expected = shapes::Shape {
            material: Material {
                colour: Colour::new(1.0, 1.0, 1.0),
                ambient: 1.0,
                diffuse: 0.0,
                specular: 0.0,
                ..Default::default()
            },
            transform: Matrix::identity(),
            ..Default::default()
        };
        assert_eq!(sphere, expected);
    }
}
