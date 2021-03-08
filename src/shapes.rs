use crate::canvas::Colour;
use crate::matrices::Matrix;
use crate::rays::{Intersection, Ray};
use crate::tuple::Tuple;
use serde::Serialize;
use erased_serde;

#[derive(Debug, PartialEq, Serialize)]
pub enum ShapeType {
    Sphere,
    Plane,
    // Cube
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Shape {
    pub material: Material,
    pub transform: Matrix<f64, 4, 4>,
    pub shape: ShapeType,
}

#[derive(Debug, Serialize)]
// go through and annotate all these attributes
pub struct Material {
    pub colour: Colour,
    pub ambient: f64,
    pub diffuse: f64,
    pub specular: f64,
    pub shininess: f64,
    pub reflectivity: f64,
    pub transparency: f64,
    pub refractive_index: f64,
    pub pattern: Option<Box<dyn Pattern>>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct StripePattern {
    pub colour_a: Colour,
    pub colour_b: Colour,
    pub transform: Matrix<f64, 4, 4>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct CheckPattern3D {
    pub colour_a: Colour,
    pub colour_b: Colour,
    pub transform: Matrix<f64, 4, 4>,
}

#[derive(Debug, PartialEq, Default, Serialize)]
pub struct TestPattern {
    pub transform: Matrix<f64, 4, 4>,
}

impl Pattern for CheckPattern3D {
    fn pattern_at(&self, point: &Tuple) -> Colour {
        if (point.x.floor() + point.y.floor() + point.z.floor()) as i32 % 2 == 0 {
            self.colour_a
        } else {
            self.colour_b
        }
    }
    fn pattern_at_object(&self, object: &Shape, point: &Tuple) -> Colour {
        let object_space_point = object.transform.inverse() * point;
        let pattern_point = self.transform.inverse() * &object_space_point;
        self.pattern_at(&pattern_point)
    }
}

impl Pattern for TestPattern {
    fn pattern_at(&self, point: &Tuple) -> Colour {
        Colour::new(point.x, point.y, point.z)
    }

    fn pattern_at_object(&self, object: &Shape, point: &Tuple) -> Colour {
        let object_space_point = object.transform.inverse() * point;
        let pattern_point = self.transform.inverse() * &object_space_point;
        self.pattern_at(&pattern_point)
    }
}

impl PartialEq for Material {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

pub trait Pattern: erased_serde::Serialize {
    fn pattern_at(&self, point: &Tuple) -> Colour;
    fn pattern_at_object(&self, object: &Shape, point: &Tuple) -> Colour;
}
erased_serde::serialize_trait_object!(Pattern);

impl std::fmt::Debug for dyn Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", "derp")
    }
}

impl Pattern for StripePattern {
    fn pattern_at(&self, point: &Tuple) -> Colour {
        if point.x.floor() as i32 % 2 == 0 {
            self.colour_a
        } else {
            self.colour_b
        }
    }

    fn pattern_at_object(&self, object: &Shape, point: &Tuple) -> Colour {
        let object_space_point = object.transform.inverse() * point;
        let pattern_point = self.transform.inverse() * &object_space_point;
        self.pattern_at(&pattern_point)
    }
}

impl Default for StripePattern {
    fn default() -> StripePattern {
        StripePattern {
            colour_a: Colour::white(),
            colour_b: Colour::black(),
            transform: Matrix::identity(),
        }
    }
}

impl Default for CheckPattern3D {
    fn default() -> CheckPattern3D {
        CheckPattern3D {
            colour_a: Colour::white(),
            colour_b: Colour::black(),
            transform: Matrix::identity(),
        }
    }
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
    pub(super) fn normal_at() -> Tuple {
        let object_normal = Tuple::point_new(0.0, 1.0, 0.0);
        object_normal
    }

    pub fn default() -> Shape {
        Shape {
            shape: ShapeType::Plane,
            ..Default::default()
        }
    }

    pub(super) fn intersects<'a>(plane: &'a Shape, r: &Ray) -> Vec<Intersection<'a>> {
        const EPSILON: f64 = 0.00001;
        if r.direction.y.abs() < EPSILON {
            vec![]
        } else {
            vec![Intersection::new(
                -r.origin.y / r.direction.normalise().y,
                plane,
            )]
        }
    }
}

pub mod sphere {
    use super::*;
    pub(super) fn normal_at(point: &Tuple) -> Tuple {
        let object_normal = point - &Tuple::point_new(0.0, 0.0, 0.0);
        object_normal
    }

    pub fn default() -> Shape {
        Shape {
            shape: ShapeType::Sphere,
            ..Default::default()
        }
    }

    pub fn glass_sphere() -> Shape {
        Shape {
            shape: ShapeType::Sphere,
            material: Material {
                transparency: 1.0,
                refractive_index: 1.5,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub(super) fn intersects<'a>(sphere: &'a Shape, r: &Ray) -> Vec<Intersection<'a>> {
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
            refractive_index: 1.0,
            transparency: 0.0,
            pattern: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::float_eq;
    use crate::lighting::ShadowInformation;
    use crate::lighting::{calculate_lighting, PointLight};

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
    #[test]
    fn stripe_pattern_constant_in_y() {
        let pat = StripePattern::default();
        let p1 = Tuple::point_new(0.0, 0.0, 0.0);
        let p2 = Tuple::point_new(0.0, 1.0, 0.0);
        let p3 = Tuple::point_new(0.0, 2.0, 0.0);
        assert_eq!(pat.pattern_at(&p1), Colour::white());
        assert_eq!(pat.pattern_at(&p2), Colour::white());
        assert_eq!(pat.pattern_at(&p3), Colour::white());
    }

    #[test]
    fn stripe_pattern_constant_in_z() {
        let pat = StripePattern {
            colour_a: Colour::black(),
            colour_b: Colour::white(),
            ..Default::default()
        };
        let p1 = Tuple::point_new(0.0, 0.0, 0.0);
        let p2 = Tuple::point_new(0.0, 0.0, 1.0);
        let p3 = Tuple::point_new(0.0, 0.0, 2.0);
        assert_eq!(pat.pattern_at(&p1), Colour::black());
        assert_eq!(pat.pattern_at(&p2), Colour::black());
        assert_eq!(pat.pattern_at(&p3), Colour::black());
    }

    #[test]
    fn stripe_pattern_changes_in_x() {
        let pat = StripePattern {
            colour_a: Colour::black(),
            colour_b: Colour::white(),
            ..Default::default()
        };
        let p1 = Tuple::point_new(0.0, 0.0, 0.0);
        let p2 = Tuple::point_new(1.01, 0.0, 0.0);
        let p3 = Tuple::point_new(-0.1, 0.0, 0.0);
        let p4 = Tuple::point_new(-1.0000001, 0.0, 0.0);
        let p5 = Tuple::point_new(-1.1, 0.0, 0.0);
        assert_eq!(pat.pattern_at(&p1), Colour::black());
        assert_eq!(pat.pattern_at(&p2), Colour::white());
        assert_eq!(pat.pattern_at(&p3), Colour::white());
        assert_eq!(pat.pattern_at(&p4), Colour::black());
        assert_eq!(pat.pattern_at(&p5), Colour::black());
    }

    #[test]
    fn lighting_with_pattern() {
        let s = Shape::default();
        let m = Material {
            pattern: Some(Box::new(StripePattern::default())),
            ambient: 1.0,
            diffuse: 0.0,
            specular: 0.0,
            ..Default::default()
        };
        let eyevec = Tuple::vector_new(0.0, 0.0, -1.0);
        let normalvec = Tuple::vector_new(0.0, 0.0, -1.0);
        let light = PointLight::new(Colour::white(), Tuple::point_new(0.0, 0.0, -10.0));
        let c1 = calculate_lighting(
            &m,
            &s,
            &light,
            &Tuple::point_new(0.9, 0.0, 0.0),
            &eyevec,
            &normalvec,
            &ShadowInformation::default(),
        );
        let c2 = calculate_lighting(
            &m,
            &s,
            &light,
            &Tuple::point_new(1.1, 0.0, 0.0),
            &eyevec,
            &normalvec,
            &ShadowInformation::default(),
        );
        assert_eq!(c1, Colour::white());
        assert_eq!(c2, Colour::black());
    }

    #[test]
    fn stripes_with_object_transformation() {
        let object = Shape {
            transform: Matrix::scaling(2.0, 2.0, 2.0),
            ..sphere::default()
        };
        let pattern = StripePattern {
            colour_a: Colour::white(),
            colour_b: Colour::black(),
            ..Default::default()
        };
        let c = pattern.pattern_at_object(&object, &Tuple::point_new(1.5, 0.0, 0.0));
        assert_eq!(c, Colour::white());
    }

    #[test]
    fn stripes_with_pattern_transformation() {
        let object = Shape {
            ..sphere::default()
        };
        let pattern = StripePattern {
            colour_a: Colour::white(),
            colour_b: Colour::black(),
            transform: Matrix::scaling(2.0, 2.0, 2.0),
        };
        let c = pattern.pattern_at_object(&object, &Tuple::point_new(1.5, 0.0, 0.0));
        assert_eq!(c, Colour::white());
    }

    #[test]
    fn stripes_with_pattern_and_object_transformation() {
        let object = Shape {
            transform: Matrix::scaling(2.0, 2.0, 2.0),
            ..sphere::default()
        };
        let pattern = StripePattern {
            colour_a: Colour::white(),
            colour_b: Colour::black(),
            transform: Matrix::translation(0.5, 0.0, 0.0),
        };
        let c = pattern.pattern_at_object(&object, &Tuple::point_new(2.5, 0.0, 0.0));
        assert_eq!(c, Colour::white());
    }

    #[test]
    fn checks_repeat_in_x() {
        let pattern = CheckPattern3D::default();
        assert_eq!(
            pattern.pattern_at(&Tuple::point_new(0.0, 0.0, 0.0)),
            Colour::white()
        );
        assert_eq!(
            pattern.pattern_at(&Tuple::point_new(0.99, 0.0, 0.0)),
            Colour::white()
        );
        assert_eq!(
            pattern.pattern_at(&Tuple::point_new(1.01, 0.0, 0.0)),
            Colour::black()
        );
    }

    #[test]
    fn checks_repeat_in_y() {
        let pattern = CheckPattern3D::default();
        assert_eq!(
            pattern.pattern_at(&Tuple::point_new(0.0, 0.0, 0.0)),
            Colour::white()
        );
        assert_eq!(
            pattern.pattern_at(&Tuple::point_new(0.0, 0.99, 0.0)),
            Colour::white()
        );
        assert_eq!(
            pattern.pattern_at(&Tuple::point_new(0.0, 1.01, 0.0)),
            Colour::black()
        );
    }

    #[test]
    fn checks_repeat_in_z() {
        let pattern = CheckPattern3D::default();
        assert_eq!(
            pattern.pattern_at(&Tuple::point_new(0.0, 0.0, 0.0)),
            Colour::white()
        );
        assert_eq!(
            pattern.pattern_at(&Tuple::point_new(0.0, 0.0, 0.99)),
            Colour::white()
        );
        assert_eq!(
            pattern.pattern_at(&Tuple::point_new(0.0, 0.0, 1.01)),
            Colour::black()
        );
    }

    // #[test]
    // fn serialise_a_shape() {
    //     let s = Shape::default();
    //     let yaml = serde_yaml::to_string(&s).unwrap();
    //     println!("{}", yaml);
    //     assert!(false);
    // }
}
