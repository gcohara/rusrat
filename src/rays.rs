use crate::matrices::Matrix;
use crate::shapes::Shape;
use crate::tuple::Tuple;
use crate::world::World;
use std::cmp::Ordering;
use std::f64::EPSILON;

#[derive(Debug)]
pub struct Ray {
    pub origin: Tuple,
    pub direction: Tuple,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Intersection<'a> {
    pub t: f64,
    pub object: &'a Shape,
}

impl<'a> Intersection<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let eq = (self.t - other.t).abs() < EPSILON;
        if eq {
            Some(Ordering::Equal)
        } else {
            match self.t > other.t {
                true => Some(Ordering::Greater),
                false => Some(Ordering::Less),
            }
        }
    }

    pub fn new(t: f64, object: &'a Shape) -> Intersection<'a> {
        Intersection { t, object }
    }

    pub fn hit(intersections: Vec<Intersection<'a>>) -> Option<Intersection<'a>> {
        intersections
            .into_iter()
            .filter(|x| x.t >= 0.0)
            .min_by(|i1, i2| i1.partial_cmp(i2).unwrap())
    }
}

impl Ray {
    pub fn new(point: Tuple, vector: Tuple) -> Ray {
        Ray {
            origin: point,
            direction: vector,
        }
    }

    pub fn position(&self, t: f64) -> Tuple {
        self.origin + (t * &self.direction)
    }

    pub fn intersects_world<'a>(&self, w: &'a World) -> Vec<Intersection<'a>> {
        let mut out = Vec::new();
        for shape in w.objects.iter() {
            out.append(&mut shape.intersects(&self))
        }
        out.sort_by(|i, j| i.partial_cmp(j).unwrap());
        out
    }

    pub fn transform(&self, m: &Matrix<f64, 4, 4>) -> Ray {
        Ray {
            origin: m * &self.origin,
            direction: m * &self.direction,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::sphere;
    #[test]
    fn computing_point_from_distance() {
        let r = Ray::new(
            Tuple::point_new(2.0, 3.0, 4.0),
            Tuple::vector_new(1.0, 0.0, 0.0),
        );
        assert_eq!(r.position(2.5), Tuple::point_new(4.5, 3.0, 4.0));
        assert_eq!(r.position(0.0), Tuple::point_new(2.0, 3.0, 4.0));
        assert_eq!(r.position(-1.0), Tuple::point_new(1.0, 3.0, 4.0));
    }

    #[test]
    fn ray_intersecting_sphere_at_two_points() {
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, -5.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let s = sphere::default();
        let xs = s.intersects(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 4.0);
        assert_eq!(xs[1].t, 6.0);
    }

    #[test]
    fn ray_intersecting_sphere_at_tangent() {
        let r = Ray::new(
            Tuple::point_new(0.0, 1.0, -5.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let s = sphere::default();
        let xs = s.intersects(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 5.0);
        assert_eq!(xs[1].t, 5.0);
    }

    #[test]
    fn ray_misses_sphere() {
        let r = Ray::new(
            Tuple::point_new(0.0, 2.0, -5.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let s = sphere::default();
        let xs = s.intersects(&r);
        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn ray_originates_inside_sphere() {
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, 0.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let s = sphere::default();
        let xs = s.intersects(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, -1.0);
        assert_eq!(xs[1].t, 1.0);
    }

    #[test]
    fn sphere_is_behind_ray() {
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, 5.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let s = sphere::default();
        let xs = s.intersects(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, -6.0);
        assert_eq!(xs[1].t, -4.0);
    }

    #[test]
    fn sphere_intersect_fn_returns_intersects_with_correct_sphere() {
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, 5.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let s = sphere::default();
        let xs = s.intersects(&r);
        assert_eq!(xs[0].object, &s);
        assert_eq!(xs[1].object, &s);
    }

    #[test]
    fn hit_point_when_t_both_positive() {
        let s = sphere::default();
        let i1 = Intersection::new(1.0, &s);
        let i2 = Intersection::new(2.0, &s);
        let xs = vec![i1, i2];
        let i = Intersection::hit(xs).unwrap();
        assert_eq!(i, Intersection::new(1.0, &s));
    }

    #[test]
    fn hit_point_when_one_t_negative() {
        let s = sphere::default();
        let i1 = Intersection::new(-1.0, &s);
        let i2 = Intersection::new(1.0, &s);
        let xs = vec![i1, i2];
        let i = Intersection::hit(xs).unwrap();
        assert_eq!(i, Intersection::new(1.0, &s));
    }

    #[test]
    fn hit_point_when_t_both_negative() {
        let s = sphere::default();
        let i1 = Intersection::new(-1.0, &s);
        let i2 = Intersection::new(-2.0, &s);
        let xs = vec![i1, i2];
        let i = Intersection::hit(xs);
        assert_eq!(i, Option::None);
    }

    #[test]
    fn translating_a_ray() {
        let r = Ray::new(
            Tuple::point_new(1.0, 2.0, 3.0),
            Tuple::vector_new(0.0, 1.0, 0.0),
        );
        let m = Matrix::translation(3.0, 4.0, 5.0);
        let r2 = r.transform(&m);
        assert_eq!(r2.origin, Tuple::point_new(4.0, 6.0, 8.0));
        assert_eq!(r2.direction, Tuple::vector_new(0.0, 1.0, 0.0));
    }

    #[test]
    fn scaling_a_ray() {
        let r = Ray::new(
            Tuple::point_new(1.0, 2.0, 3.0),
            Tuple::vector_new(0.0, 1.0, 0.0),
        );
        let m = Matrix::scaling(2.0, 3.0, 4.0);
        let r2 = r.transform(&m);
        assert_eq!(r2.origin, Tuple::point_new(2.0, 6.0, 12.0));
        assert_eq!(r2.direction, Tuple::vector_new(0.0, 3.0, 0.0));
    }

    #[test]
    fn changing_a_spheres_transformation() {
        let mut s = sphere::default();
        s.transform = Matrix::translation(2.0, 3.0, 4.0);
        assert_eq!(s.transform, Matrix::translation(2.0, 3.0, 4.0));
    }

    #[test]
    fn intersecting_scaled_sphere_with_ray() {
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, -5.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let mut s = sphere::default();
        s.transform = Matrix::scaling(2.0, 2.0, 2.0);
        let xs = s.intersects(&r);
        assert_eq!(xs[0].t, 3.0);
        assert_eq!(xs[1].t, 7.0);
    }

    #[test]
    fn intersecting_translated_sphere_with_ray() {
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, -5.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let mut s = sphere::default();
        s.transform = Matrix::translation(5.0, 0.0, 0.0);
        let xs = s.intersects(&r);
        assert_eq!(xs.len(), 0);
    }
}
