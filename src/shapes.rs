use crate::canvas::Colour;
use crate::matrices::Matrix;
use crate::rays::{Intersection, Ray};
use crate::tuple::Tuple;

#[derive(Debug, PartialEq)]
pub enum ShapeType {
    Sphere,
    Plane,
}

#[derive(Debug, PartialEq)]
pub struct Shape {
    pub material: Material,
    pub transform: Matrix<f64, 4, 4>,
    pub shape: ShapeType,
}

#[derive(Debug, PartialEq)]
pub struct Material {
    pub colour: Colour,
    pub ambient: f64,
    pub diffuse: f64,
    pub specular: f64,
    pub shininess: f64,
    pub reflectivity: f64,
}

impl Shape {
    pub fn normal_at(&self, point: &Tuple) -> Tuple {
        let transform_inverse = &self.transform.inverse();
        let object_space_point = transform_inverse * point;
        let object_space_normal = match self.shape {
            ShapeType::Sphere => sphere::normal_at(&object_space_point),
            ShapeType::Plane => plane::normal_at(),
        };
        let world_space_normal = transform_inverse.transpose() * &object_space_normal;
        world_space_normal.normalise()
    }

    pub fn intersects<'a>(&'a self, r: &Ray) -> Vec<Intersection<'a>> {
        let transform_inverse = &self.transform.inverse();
        let object_space_ray = r.transform(transform_inverse);
        match self.shape {
            ShapeType::Sphere => sphere::intersects(self, &object_space_ray),
            ShapeType::Plane => plane::intersects(self, &object_space_ray),
        }
    }
}

pub mod plane {
    use super::*;
    pub (in super) fn normal_at() -> Tuple {
        let object_normal = Tuple::point_new(0.0, 1.0, 0.0);
        object_normal
    }

    pub fn default() -> Shape {
        Shape {
            shape: ShapeType::Plane,
            ..Default::default()
        }
    }

    pub (in super) fn intersects<'a>(plane: &'a Shape, r: &Ray) -> Vec<Intersection<'a>> {
        const EPSILON: f64 = 0.00001;
        if r.direction.y.abs() < EPSILON {
            vec![]
        } else {
            vec![Intersection::new(
                - r.origin.y / r.direction.normalise().y,
                plane,
            )]
        }
    }
}

pub mod sphere {
    use super::*;
    pub (in super) fn normal_at(point: &Tuple) -> Tuple {
        let object_normal = point - &Tuple::point_new(0.0, 0.0, 0.0);
        object_normal
    }

    pub fn default() -> Shape {
        Shape {
            shape: ShapeType::Sphere,
            ..Default::default()
        }
    }

    pub (in super) fn intersects<'a>(sphere: &'a Shape, r: &Ray) -> Vec<Intersection<'a>> {
        let sphere_to_ray = r.origin - Tuple::point_new(0.0, 0.0, 0.0);
        let a = r.direction.dot(&r.direction);
        let b = 2.0 * r.direction.dot(&sphere_to_ray);
        let c = sphere_to_ray.dot(&sphere_to_ray) - 1.0;
        let discriminant = b.powi(2) - (4.0 * a * c);
        match discriminant < 0.0 {
            true => Vec::new(),
            false => {
                let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
                let t2 = (-b + discriminant.sqrt()) / (2.0 * a);
                vec![Intersection::new(t1, sphere), Intersection::new(t2, sphere)]
            }
        }
    }
}
impl Default for Shape {
    fn default() -> Shape {
        Shape {
            material: Material::default(),
            transform: Matrix::identity(),
            shape: ShapeType::Sphere,
        }
    }
}

impl Default for Material {
    fn default() -> Material {
        Material {
            colour: Colour::new(1.0, 1.0, 1.0),
            ambient: 0.1,
            diffuse: 0.9,
            specular: 0.9,
            shininess: 200.0,
            reflectivity: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::float_eq;
    
    #[test]
    fn normal_of_sphere() {
        let s = sphere::default();
        let n = s.normal_at(&Tuple::point_new(1.0, 0.0, 0.0));
        assert_eq!(n, Tuple::vector_new(1.0, 0.0, 0.0));
    }

    #[test]
    fn normal_of_translated_sphere() {
        let s = Shape {
            transform: Matrix::translation(0.0, 1.0, 0.0),
            ..sphere::default()
        };
        let n = s.normal_at(&Tuple::point_new(0.0, 1.70711, -0.70711));
        assert_eq!(n, Tuple::vector_new(0.0, 0.70711, -0.70711));
    }

    #[test]
    fn normal_of_transformed_sphere() {
        use std::f64::consts::{FRAC_1_SQRT_2, PI};
        let s = Shape {
            transform: Matrix::rotation_z(PI / 5.0).scale(1.0, 0.5, 1.0),
            ..sphere::default()
        };
        let n = s.normal_at(&Tuple::point_new(0.0, FRAC_1_SQRT_2, -FRAC_1_SQRT_2));
        assert_eq!(n, Tuple::vector_new(0.0, 0.97014, -0.24254));
    }

    #[test]
    fn normal_of_plane() {
        let p = plane::default();
        let n = p.normal_at(&Tuple::point_new(0.21, 0.543, 0.438294));
        assert_eq!(n, Tuple::vector_new(0.0, 1.0, 0.0))
    }

    #[test]
    fn normal_of_rotated_plane() {
        let p = Shape {
            shape: ShapeType::Plane,
            transform: Matrix::rotation_x(std::f64::consts::PI / 2.0),
            ..Default::default()
        };
        let n = p.normal_at(&Tuple::point_new(0.21, 0.543, 0.438294));
        assert_eq!(n, Tuple::vector_new(0.0, 0.0, 1.0))
    }

    #[test]
    fn intersection_with_ray_parallel_to_plane() {
        let p = plane::default();
        let r = Ray::new(
            Tuple::point_new(0.0, 10.0, 0.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let xs = p.intersects(&r);
        assert_eq!(xs, Vec::new());
    }

    #[test]
    fn intersection_with_ray_coplanar_to_plane() {
        let p = plane::default();
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, 0.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let xs = p.intersects(&r);
        assert_eq!(xs, Vec::new());
    }

    #[test]
    fn ray_intersecting_plane_from_above() {
        let p = plane::default();
        let r = Ray::new(
            Tuple::point_new(0.0, 1.0, 0.0),
            Tuple::vector_new(0.0, -1.0, 0.0),
        );
        let xs = p.intersects(&r);
        assert_eq!(xs.len(), 1);
        assert_eq!(xs[0].t, 1.0);
        assert_eq!(xs[0].object, &p);
    }

    #[test]
    fn ray_intersecting_plane_from_below() {
        let p = plane::default();
        let r = Ray::new(
            Tuple::point_new(0.0, -1.0, 0.0),
            Tuple::vector_new(0.0, 1.0, 0.0),
        );
        let xs = p.intersects(&r);
        assert_eq!(xs.len(), 1);
        assert_eq!(xs[0].t, 1.0);
        assert_eq!(xs[0].object, &p);
    }

    #[test]
    fn ray_intersecting_transformed_plane() {
        let p = Shape {
            transform: Matrix::rotation_x(std::f64::consts::PI / 2.0),
            ..plane::default()
        };
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, -2.0),
            Tuple::vector_new(0.0, 1.0, 1.0),
        );
        let xs = p.intersects(&r);
        assert_eq!(xs.len(), 1);
        assert!(float_eq(xs[0].t, 2.0 * std::f64::consts::SQRT_2));
        assert_eq!(xs[0].object, &p);
    }
}
