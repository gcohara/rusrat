use crate::canvas::Colour;
use crate::matrices::Matrix;
use crate::rays::{Intersection, Ray};
use crate::tuple::Tuple;

#[derive(Debug, PartialEq)]
pub struct Sphere {
    pub material: Material,
    pub transform: Matrix<f64, 4, 4>,
}
#[derive(Debug, PartialEq)]
pub struct Plane {
    pub material: Material,
    pub transform: Matrix<f64, 4, 4>,
}

// eventually refactor this way
// pub struct Shape {
//     pub material: Material,
//     pub transform: Matrix<f64, 4, 4>,
//     pub shape_type: ShapeType
// }

pub enum Shape {
    Sphere(Sphere),
    Plane(Plane),
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

// pub trait Shape
// where
//     Self: Sized,
// {
//     fn new(m: Material, t: Matrix<f64, 4, 4>) -> Self;
//     fn normal_at(&self, point: &Tuple) -> Tuple;
//     fn intersects<'a>(&'a self, r: &Ray) -> Vec<Intersection<'a, Self>>;
// }

impl Sphere {
    fn new(m: Material, t: Matrix<f64, 4, 4>) -> Sphere {
        Sphere {
            material: m,
            transform: t,
        }
    }

    fn normal_at(&self, point: &Tuple) -> Tuple {
        let transform_inverse = self.transform.inverse();
        let object_space_point = &transform_inverse * point;
        let object_normal = object_space_point - Tuple::point_new(0.0, 0.0, 0.0);
        let world_normal = transform_inverse.transpose() * &object_normal;
        world_normal.normalise()
    }

    fn intersects<'a>(&'a self, r: &Ray) -> Vec<Intersection<'a, Sphere>> {
        let transformed_ray = r.transform(&self.transform.inverse());
        let sphere_to_ray = transformed_ray.origin - Tuple::point_new(0.0, 0.0, 0.0);
        let a = transformed_ray.direction.dot(&transformed_ray.direction);
        let b = 2.0 * transformed_ray.direction.dot(&sphere_to_ray);
        let c = sphere_to_ray.dot(&sphere_to_ray) - 1.0;
        let discriminant = b.powi(2) - (4.0 * a * c);
        match discriminant < 0.0 {
            true => Vec::new(),
            false => {
                let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
                let t2 = (-b + discriminant.sqrt()) / (2.0 * a);
                vec![Intersection::new(t1, &self), Intersection::new(t2, &self)]
            }
        }
    }
}

impl Plane {
    fn new(m: Material, t: Matrix<f64, 4, 4>) -> Plane {
        Plane {
            material: m,
            transform: t,
        }
    }

    fn normal_at(&self, _point: &Tuple) -> Tuple {
        let transform_inverse = self.transform.inverse();
        let object_normal = Tuple::point_new(0.0, 1.0, 0.0);
        let world_normal = transform_inverse.transpose() * &object_normal;
        world_normal.normalise()
    }

    fn intersects<'a>(&'a self, r: &Ray) -> Vec<Intersection<'a, Plane>> {
        const EPSILON: f64 = 0.00001;
        let local_ray = r.transform(&self.transform.inverse());
        println! {"Local ray {:#?}", local_ray};
        if local_ray.direction.y.abs() < EPSILON {
            vec![]
        } else {
            vec![Intersection::new(
                -local_ray.origin.y / local_ray.direction.normalise().y,
                &self,
            )]
        }
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

impl Default for Plane {
    fn default() -> Plane {
        Plane {
            material: Material::default(),
            transform: Matrix::identity(),
        }
    }
}

impl Material {
    pub fn new(
        colour: Colour,
        ambient: f64,
        diffuse: f64,
        specular: f64,
        shininess: f64,
        reflectivity: f64,
    ) -> Material {
        Material {
            colour,
            ambient,
            diffuse,
            specular,
            shininess,
            reflectivity,
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
        s.transform = m;
        let n = s.normal_at(&Tuple::point_new(0.0, 1.70711, -0.70711));
        assert_eq!(n, Tuple::vector_new(0.0, 0.70711, -0.70711));
    }

    #[test]
    fn normal_of_transformed_sphere() {
        use std::f64::consts::{FRAC_1_SQRT_2, PI};
        let mut s = Sphere::default();
        let m = Matrix::rotation_z(PI / 5.0).scale(1.0, 0.5, 1.0);
        s.transform = m;
        let n = s.normal_at(&Tuple::point_new(0.0, FRAC_1_SQRT_2, -FRAC_1_SQRT_2));
        assert_eq!(n, Tuple::vector_new(0.0, 0.97014, -0.24254));
    }

    #[test]
    fn normal_of_plane() {
        let p = Plane::new(Material::default(), Matrix::identity());
        let n = p.normal_at(&Tuple::point_new(0.21, 0.543, 0.438294));
        assert_eq!(n, Tuple::vector_new(0.0, 1.0, 0.0))
    }

    #[test]
    fn normal_of_rotated_plane() {
        let p = Plane::new(
            Material::default(),
            Matrix::rotation_x(std::f64::consts::PI / 2.0),
        );
        let n = p.normal_at(&Tuple::point_new(0.21, 0.543, 0.438294));
        assert_eq!(n, Tuple::vector_new(0.0, 0.0, 1.0))
    }

    #[test]
    fn intersection_with_ray_parallel_to_plane() {
        let p = Plane::default();
        let r = Ray::new(
            Tuple::point_new(0.0, 10.0, 0.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let xs = p.intersects(&r);
        assert_eq!(xs, Vec::new());
    }

    #[test]
    fn intersection_with_ray_coplanar_to_plane() {
        let p = Plane::default();
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, 0.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let xs = p.intersects(&r);
        assert_eq!(xs, Vec::new());
    }

    #[test]
    fn ray_intersecting_plane_from_above() {
        let p = Plane::default();
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
        let p = Plane::default();
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
        let p = Plane::new(
            Material::default(),
            // Matrix::scaling(2.0,2.0,2.0),
            Matrix::rotation_x(std::f64::consts::PI / 2.0),
        );
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, -2.0),
            Tuple::vector_new(0.0, 1.0, 1.0),
        );
        let xs = p.intersects(&r);
        assert_eq!(xs.len(), 1);
        assert_eq!(xs[0].t, 2.0 * std::f64::consts::SQRT_2);
        assert_eq!(xs[0].object, &p);
    }
}
