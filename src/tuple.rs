use std::ops::{Add, Mul, Sub};

#[derive(Debug)]
struct Tuple {
    x: f64,
    y: f64,
    z: f64,
    w: f64,
}

fn equal(x: f64, y: f64) -> bool {
    const EPSILON: f64 = 0.001;
    (x - y).abs() <= EPSILON
}

impl Tuple {
    fn new(x: f64, y: f64, z: f64, w: f64) -> Tuple {
        Tuple { x, y, z, w }
    }

    fn point_new(x: f64, y: f64, z: f64) -> Tuple {
        Tuple::new(x, y, z, 1.0)
    }

    fn vector_new(x: f64, y: f64, z: f64) -> Tuple {
        Tuple::new(x, y, z, 0.0)
    }

    fn is_point(&self) -> bool {
        equal(self.w, 1.0)
    }

    fn is_vector(&self) -> bool {
        equal(self.w, 0.0)
    }

    fn negate(&self) -> Tuple {
        Tuple::new(-self.x, -self.y, -self.z, -self.w)
    }

    fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    fn normalise(&self) -> Tuple {
        let mag = self.magnitude();
        Tuple::vector_new(self.x / mag, self.y / mag, self.z / mag)
    }

    fn dot(&self, other: &Tuple) -> f64 {
        (self.x * other.x) + (self.y * other.y) + (self.z * other.z) + (self.w * other.w)
    }

    fn cross(&self, other: &Tuple) -> Tuple {
        Tuple::vector_new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }
}

impl PartialEq for Tuple {
    fn eq(&self, other: &Self) -> bool {
        equal(self.w, other.w)
            && equal(self.x, other.x)
            && equal(self.y, other.y)
            && equal(self.z, other.z)
    }
}

impl Add for Tuple {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl Sub for Tuple {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

impl Mul<f64> for Tuple {
    type Output = Self;
    fn mul(self, other: f64) -> Self {
        Tuple::new(
            other * self.x,
            other * self.y,
            other * self.z,
            other * self.w,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tuple_with_4th_eq_1_is_point() {
        let a = Tuple::new(4.3, -4.2, 3.1, 1.0);
        assert!(a.is_point());
    }

    #[test]
    fn tuple_with_4th_eq_0_is_vector() {
        let a = Tuple::new(4.3, -4.2, 3.1, 0.0);
        assert!(a.is_vector());
    }

    #[test]
    fn new_point_works() {
        let a = Tuple::point_new(4.0, -4.0, 3.0);
        let b = Tuple::new(4.0, -4.0, 3.0, 1.0);
        assert_eq!(a, b);
    }

    #[test]
    fn new_vector_works() {
        let a = Tuple::vector_new(4.0, -4.0, 3.0);
        let b = Tuple::new(4.0, -4.0, 3.0, 0.0);
        assert_eq!(a, b);
    }

    #[test]
    fn add_tuples() {
        let a = Tuple::new(3.0, -2.0, 5.0, 1.0);
        let b = Tuple::new(-2.0, 3.0, 1.0, 0.0);
        assert_eq!(a + b, Tuple::new(1.0, 1.0, 6.0, 1.0));
    }

    #[test]
    fn sub_two_points() {
        let a = Tuple::point_new(3.0, 2.0, 1.0);
        let b = Tuple::point_new(5.0, 6.0, 7.0);
        assert_eq!(a - b, Tuple::vector_new(-2.0, -4.0, -6.0));
    }

    #[test]
    fn sub_vector_from_point() {
        let a = Tuple::point_new(3.0, 2.0, 1.0);
        let b = Tuple::vector_new(5.0, 6.0, 7.0);
        assert_eq!(a - b, Tuple::point_new(-2.0, -4.0, -6.0));
    }

    #[test]
    fn sub_two_vector() {
        let a = Tuple::vector_new(3.0, 2.0, 1.0);
        let b = Tuple::vector_new(5.0, 6.0, 7.0);
        assert_eq!(a - b, Tuple::vector_new(-2.0, -4.0, -6.0));
    }

    #[test]
    fn negate_tuple() {
        let a = Tuple::new(1.0, -2.0, 3.0, -4.0);
        assert_eq!(a.negate(), Tuple::new(-1.0, 2.0, -3.0, 4.0));
    }

    #[test]
    fn scalar_mult() {
        let a = Tuple::new(1.0, -2.0, 3.0, -4.0);
        assert_eq!(a * 3.5, Tuple::new(3.5, -7.0, 10.5, -14.0));
    }

    #[test]
    fn scalar_mult_by_fraction() {
        let a = Tuple::new(1.0, -2.0, 3.0, -4.0);
        assert_eq!(a * 0.5, Tuple::new(0.5, -1.0, 1.5, -2.0));
    }

    #[test]
    fn magnitude_of_vector() {
        let a = Tuple::vector_new(1.0, 2.0, 3.0);
        assert_eq!(a.magnitude(), 14.0_f64.sqrt())
    }

    #[test]
    fn normalise_vector() {
        let a = Tuple::vector_new(1.0, 2.0, 3.0);
        assert_eq!(a.normalise(), Tuple::vector_new(0.26726, 0.53452, 0.80178));
    }

    #[test]
    fn normalise_vector_has_mag_1() {
        let a = Tuple::vector_new(1.0, 2.0, 3.0);
        assert_eq!(a.normalise().magnitude(), 1.0);
    }

    #[test]
    fn dot_product() {
        let a = Tuple::vector_new(1.0, 2.0, 3.0);
        let b = Tuple::vector_new(2.0, 3.0, 4.0);
        assert_eq!(a.dot(&b), 20.0);
    }

    #[test]
    fn cross_product() {
        let a = Tuple::vector_new(1.0, 2.0, 3.0);
        let b = Tuple::vector_new(2.0, 3.0, 4.0);
        assert_eq!(a.cross(&b), Tuple::vector_new(-1.0, 2.0, -1.0));
    }

    #[test]
    fn cross_product_produces_negations() {
        let a = Tuple::vector_new(1.0, 2.0, 3.0);
        let b = Tuple::vector_new(2.0, 3.0, 4.0);
        assert_eq!(a.cross(&b), b.cross(&a).negate());
    }
}
