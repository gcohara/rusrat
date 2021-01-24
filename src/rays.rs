use crate::shapes::{Shape, Sphere};
use crate::tuple::Tuple;

struct Ray {
    origin: Tuple,
    direction: Tuple,
}

struct Intersection<T: Shape> {
    t: f64,
    object: T
}

impl Ray {
    fn new(point: Tuple, vector: Tuple) -> Ray {
        Ray {
            origin: point,
            direction: vector,
        }
    }

    fn position(&self, t: f64) -> Tuple {
        self.origin + (t * &self.direction)
    }

    fn intersects(&self, _s: Sphere) -> Vec<f64> {
        let sphere_to_ray = self.origin - Tuple::point_new(0.0, 0.0, 0.0);
        let a = self.direction.dot(&self.direction);
        let b = 2.0 * self.direction.dot(&sphere_to_ray);
        let c = sphere_to_ray.dot(&sphere_to_ray) - 1.0;
        let discriminant = b.powi(2) - (4.0 * a * c);
        match discriminant < 0.0 {
            true => Vec::new(),
            false => {
                let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
                let t2 = (-b + discriminant.sqrt()) / (2.0 * a);
                vec![t1, t2]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let s = Sphere::new();
        let xs = r.intersects(s);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs, vec![4.0, 6.0]);
    }

    #[test]
    fn ray_intersecting_sphere_at_tangent() {
        let r = Ray::new(
            Tuple::point_new(0.0, 1.0, -5.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let s = Sphere::new();
        let xs = r.intersects(s);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs, vec![5.0, 5.0]);
    }

    #[test]
    fn ray_misses_sphere() {
        let r = Ray::new(
            Tuple::point_new(0.0, 2.0, -5.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let s = Sphere::new();
        let xs = r.intersects(s);
        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn ray_originates_inside_sphere() {
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, 0.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let s = Sphere::new();
        let xs = r.intersects(s);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs, vec![-1.0, 1.0]);
    }

    #[test]
    fn sphere_is_behind_ray() {
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, 5.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let s = Sphere::new();
        let xs = r.intersects(s);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs, vec![-6.0, -4.0]);
    }
}
