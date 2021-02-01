use crate::matrices::Matrix;
use crate::tuple::Tuple;
use crate::canvas::Colour;

#[derive(Debug, PartialEq)]
pub struct Sphere {
    pub material: Material,
    pub transform: Matrix<f64, 4, 4>,
}

#[derive(Debug, PartialEq)]
pub struct Material {
    pub colour: Colour,
    pub ambient: f64,
    pub diffuse: f64,
    pub specular: f64,
    pub shininess: f64,
}

pub trait Shape {}
impl Shape for Sphere {}

impl Sphere {
    pub fn new(m: Material, t: Matrix<f64, 4, 4>) -> Sphere {
        Sphere {
            material: m,
            transform: t,
        }
    }

    pub fn set_transform(&mut self, t: Matrix<f64, 4, 4>) {
        self.transform = t;
    }

    pub fn set_material(&mut self, m: Material) {
        self.material = m;
    }

    pub fn normal_at(&self, point: &Tuple) -> Tuple {
        let transform_inverse = self.transform.inverse();
        let object_space_point = &transform_inverse * point;
        let object_normal = object_space_point - Tuple::point_new(0.0, 0.0, 0.0);
        let world_normal = transform_inverse.transpose() * &object_normal;
        world_normal.normalise()
    }
}

impl Default for Sphere {
    fn default() -> Sphere {
        Sphere {
            material: Material::default(),
            transform: Matrix::identity(),
        }
    }
}

impl Material {
    pub fn new(colour: Colour, ambient: f64, diffuse: f64, specular: f64, shininess: f64) -> Material {
        Material {
            colour,
            ambient,
            diffuse,
            specular,
            shininess,
        }
    }
}

impl Default for Material {
    fn default() -> Material {
        Material {
            colour: Colour::new(1.0,1.0,1.0),
            ambient: 0.1,
            diffuse: 0.9,
            specular: 0.9,
            shininess: 200.0
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_of_sphere() {
        let s = Sphere::default();
        let n = s.normal_at(&Tuple::point_new(1.0, 0.0, 0.0));
        assert_eq!(n, Tuple::vector_new(1.0, 0.0, 0.0));
    }

    #[test]
    fn normal_of_translated_sphere() {
        let mut s = Sphere::default();
        let m = Matrix::translation(0.0, 1.0, 0.0);
        s.set_transform(m);
        let n = s.normal_at(&Tuple::point_new(0.0, 1.70711, -0.70711));
        assert_eq!(n, Tuple::vector_new(0.0, 0.70711, -0.70711));
    }

    #[test]
    fn normal_of_transformed_sphere() {
        use std::f64::consts::{FRAC_1_SQRT_2, PI};
        let mut s = Sphere::default();
        let m = Matrix::rotation_z(PI / 5.0).scale(1.0, 0.5, 1.0);
        s.set_transform(m);
        let n = s.normal_at(&Tuple::point_new(0.0, FRAC_1_SQRT_2, -FRAC_1_SQRT_2));
        assert_eq!(n, Tuple::vector_new(0.0, 0.97014, -0.24254));
    }
}
