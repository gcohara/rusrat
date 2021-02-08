use std::ops::{Add, Mul, Sub};

// This struct is used to represent both points and vectors.
// Vectors will have w == 0.0, while tuples will have w == 1.0.
// All other values of w are invalid, and indicate a problem.
#[derive(Debug, Copy, Clone)]
pub struct Tuple {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    w: f64,
}

// We don't require floating point numbers to be exactly equal - just that they
// are very close (i.e within epsilon).
fn equal(x: f64, y: f64) -> bool {
    const EPSILON: f64 = 0.0001;
    (x - y).abs() <= EPSILON
}

impl Tuple {
    // Create a new tuple
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Tuple {
        Tuple { x, y, z, w }
    }
    // Create a new point (where w = 1)
    pub fn point_new(x: f64, y: f64, z: f64) -> Tuple {
        Tuple::new(x, y, z, 1.0)
    }
    // Create a new vector (where w = 0)
    pub fn vector_new(x: f64, y: f64, z: f64) -> Tuple {
        Tuple::new(x, y, z, 0.0)
    }
    // Check if the tuple represents a point
    pub fn is_point(&self) -> bool {
        equal(self.w, 1.0)
    }
    // Check if the tuple represents a vector
    pub fn is_vector(&self) -> bool {
        equal(self.w, 0.0)
    }
    // Get the negation of a tuple, including of its w component.
    // This is only used internally, to implement the Sub trait (i.e overload '-')
    pub fn negate(&self) -> Tuple {
        Tuple::new(-self.x, -self.y, -self.z, -self.w)
    }
    // Get the magnitude of a tuple.
    pub fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }
    // Normalise a tuple so that its magnitude == 1.
    pub fn normalise(&self) -> Tuple {
        let mag = self.magnitude();
        Tuple::vector_new(self.x / mag, self.y / mag, self.z / mag)
    }
    // Get the dot product of two vectors. Panics if given a point.
    pub fn dot(&self, other: &Tuple) -> f64 {
        assert!(
            self.is_vector() && other.is_vector(),
            "Attempted to take the dot product of a point/points!"
        );
        (self.x * other.x) + (self.y * other.y) + (self.z * other.z) + (self.w * other.w)
    }
    // Get the cross product of two vectors. Panics if given point.
    pub fn cross(&self, other: &Tuple) -> Tuple {
        assert!(
            self.is_vector() && other.is_vector(),
            "Attempted to take the cross product of a point/points!"
        );
        Tuple::vector_new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }
    // Get a vector copy of the tuple's values. Used for iterators.
    pub fn vector_copy(&self) -> Vec<f64> {
        vec![self.x, self.y, self.z, self.w]
    }

    pub fn reflect(&self, other: &Tuple) -> Tuple {
        assert!(
            self.is_vector() && other.is_vector(),
            "Attempted to take the vector reflection of a point/points!"
        );
        *other - (2.0 * self * other.dot(self))
    }
}

// This trait allows us to use the == operator for tuples.
impl PartialEq for Tuple {
    fn eq(&self, other: &Self) -> bool {
        equal(self.w, other.w)
            && equal(self.x, other.x)
            && equal(self.y, other.y)
            && equal(self.z, other.z)
    }
}

// This trait overloads the + operator for tuples.
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

// This trait overloads the '-' operator for tuples.
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

impl Mul<&Tuple> for f64 {
    type Output = Tuple;
    fn mul(self, other: &Tuple) -> Tuple {
        Tuple::new(
            self * other.x,
            self * other.y,
            self * other.z,
            self * other.w,
        )
    }
}

// This trait allows us to multiply a tuple by an f64 (with the f64 on the right)
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

    #[test]
    fn reflecting_a_vector_about_normal() {
        let v = Tuple::vector_new(1.0, -1.0, 0.0);
        let n = Tuple::vector_new(0.0, 1.0, 0.0);
        assert_eq!(n.reflect(&v), Tuple::vector_new(1.0, 1.0, 0.0));
    }

    #[test]
    fn reflecting_a_vector_about_normal_again() {
        use std::f64::consts::FRAC_1_SQRT_2;
        let v = Tuple::vector_new(0.0, -1.0, 0.0);
        let n = Tuple::vector_new(FRAC_1_SQRT_2, FRAC_1_SQRT_2, 0.0);
        assert_eq!(n.reflect(&v), Tuple::vector_new(1.0, 0.0, 0.0));
    }
}
