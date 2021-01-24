use crate::tuple::Tuple;
use itertools::iproduct;
use std::ops::{Index, IndexMut, Mul};

#[derive(Debug)]
struct Matrix<T, const ROWS: usize, const COLUMNS: usize> {
    rows: usize,
    columns: usize,
    data: [[T; ROWS]; COLUMNS],
}

fn translation(x: f64, y: f64, z: f64) -> Matrix<f64, 4, 4> {
    let mut out: Matrix<f64, 4, 4> = Matrix::identity();
    for i in 0..3 {
        out[i][3] = [x, y, z][i];
    }
    out
}

fn scale(x: f64, y: f64, z: f64) -> Matrix<f64, 4, 4> {
    let mut out: Matrix<f64, 4, 4> = Matrix::identity();
    for i in 0..3 {
        out[i][i] = [x, y, z][i];
    }
    out
}

fn rotation_x(radians: f64) -> Matrix<f64, 4, 4> {
    Matrix::from_array(&[
        [1.0, 0.0, 0.0, 0.0],
        [0.0, radians.cos(), -radians.sin(), 0.0],
        [0.0, radians.sin(), radians.cos(), 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

fn rotation_y(radians: f64) -> Matrix<f64, 4, 4> {
    Matrix::from_array(&[
        [radians.cos(), 0.0, radians.sin(), 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [-radians.sin(), 0.0, radians.cos(), 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

fn rotation_z(radians: f64) -> Matrix<f64, 4, 4> {
    Matrix::from_array(&[
        [radians.cos(), -radians.sin(), 0.0, 0.0],
        [radians.sin(), radians.cos(), 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

fn shear(x_y: f64, x_z: f64, y_x: f64, y_z: f64, z_x: f64, z_y: f64) -> Matrix<f64, 4, 4> {
    Matrix::from_array(&[
        [1.0, x_y, x_z, 0.0],
        [y_x, 1.0, y_z, 0.0],
        [z_x, z_y, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

// Implementations for floating point square matrix types
impl<const SIZE: usize> Matrix<f64, SIZE, SIZE> {
    fn from_array(values: &[[f64; SIZE]; SIZE]) -> Self {
        Matrix {
            rows: SIZE,
            columns: SIZE,
            data: values.clone(),
        }
    }
    fn new() -> Self {
        Matrix::from_array(&[[f64::default(); SIZE]; SIZE])
    }

    fn transpose(&self) -> Self {
        let mut out = Matrix::new();
        for (i, j) in iproduct!(0..SIZE, 0..SIZE) {
            out.data[i][j] = self.data[j][i];
        }
        out
    }

    fn identity() -> Self {
        let mut out = Matrix::new();
        for i in 0..SIZE {
            out[i][i] = 1.0;
        }
        out
    }
}

// Implementations for specific square matrices
impl Matrix<f64, 2, 2> {
    fn determinant(&self) -> f64 {
        self.data[0][0] * self.data[1][1] - self.data[0][1] * self.data[1][0]
    }
}

// Annoyingly const generics aren't at the stage where we can have ROW - 1 and
// COLUMN - 1 in the submatrix function's return type. So, we have to implement
// these seperately.
impl Matrix<f64, 3, 3> {
    fn submatrix(&self, row: usize, column: usize) -> Matrix<f64, 2, 2> {
        const SIZE: usize = 3;
        let mut out = Matrix::new();
        let row_indices: Vec<_> = (0..SIZE).filter(|i| *i != row).collect();
        let column_indices: Vec<_> = (0..SIZE).filter(|j| *j != column).collect();
        for (i, j) in iproduct!(0..(SIZE - 1), 0..(SIZE - 1)) {
            out[i][j] = self.data[row_indices[i]][column_indices[j]];
        }
        out
    }

    // could do these two seperately tbf
    fn minor(&self, row: usize, column: usize) -> f64 {
        self.submatrix(row, column).determinant()
    }

    fn cofactor(&self, row: usize, column: usize) -> f64 {
        match (row + column) % 2 {
            0 => self.minor(row, column),
            1 => -self.minor(row, column),
            _ => panic!(),
        }
    }

    fn determinant(&self) -> f64 {
        const SIZE: usize = 3;
        (0..SIZE).map(|i| self[0][i] * self.cofactor(0, i)).sum()
    }
}

impl Matrix<f64, 4, 4> {
    fn submatrix(&self, row: usize, column: usize) -> Matrix<f64, 3, 3> {
        const SIZE: usize = 4;
        let mut out = Matrix::new();
        let row_indices: Vec<_> = (0..SIZE).filter(|i| *i != row).collect();
        let column_indices: Vec<_> = (0..SIZE).filter(|j| *j != column).collect();
        for (i, j) in iproduct!(0..(SIZE - 1), 0..(SIZE - 1)) {
            out[i][j] = self.data[row_indices[i]][column_indices[j]];
        }
        out
    }

    fn minor(&self, row: usize, column: usize) -> f64 {
        self.submatrix(row, column).determinant()
    }

    fn cofactor(&self, row: usize, column: usize) -> f64 {
        match (row + column) % 2 {
            0 => self.minor(row, column),
            1 => -self.minor(row, column),
            _ => panic!(),
        }
    }

    fn determinant(&self) -> f64 {
        const SIZE: usize = 4;
        (0..SIZE).map(|i| self[0][i] * self.cofactor(0, i)).sum()
    }

    fn is_invertible(&self) -> bool {
        self.determinant() != 0.0
    }

    fn inverse(&self) -> Self {
        assert!(
            self.is_invertible(),
            "Attempted to take the inverse of a non-invertible matrix!"
        );
        const SIZE: usize = 4;
        let det = self.determinant();
        let mut out = Matrix::new();
        for (i, j) in iproduct!(0..SIZE, 0..SIZE) {
            out[j][i] = self.cofactor(i, j) / det;
        }
        out
    }

    fn translate(&self, x: f64, y: f64, z: f64) -> Self {
        translation(x, y, z) * self
    }

    fn scale(&self, x: f64, y: f64, z: f64) -> Self {
        scale(x, y, z) * self
    }

    fn rotate_x(&self, radians: f64) -> Self {
        rotation_x(radians) * self
    }

    fn rotate_y(&self, radians: f64) -> Self {
        rotation_y(radians) * self
    }

    fn rotate_z(&self, radians: f64) -> Self {
        rotation_z(radians) * self
    }

    fn shear(&self, x_y: f64, x_z: f64, y_x: f64, y_z: f64, z_x: f64, z_y: f64) -> Self {
        shear(x_y, x_z, y_x, y_z, z_x, z_y) * self
    }
}

/*
**TRAIT IMPLEMENTATIONS
**/

// This trait allows us to index into our matrix's data, and get an immutable reference.
impl<T: Copy, const ROWS: usize, const COLUMNS: usize> Index<usize> for Matrix<T, ROWS, COLUMNS> {
    type Output = [T; ROWS];
    fn index(&self, i: usize) -> &Self::Output {
        &self.data[i]
    }
}
// This trait allows us to index into the matrix's data and get a mutable reference.
impl<T: Copy, const ROWS: usize, const COLUMNS: usize> IndexMut<usize>
    for Matrix<T, ROWS, COLUMNS>
{
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.data[i]
    }
}
// This allows us to multiply matrices of the same size together.
impl<const SIZE: usize> Mul<&Matrix<f64, SIZE, SIZE>> for Matrix<f64, SIZE, SIZE> {
    type Output = Self;

    fn mul(self, rhs: &Self) -> Self {
        let mut out = Matrix::new();
        for (i, j) in iproduct!(0..SIZE, 0..SIZE) {
            let row = self.data[i].iter();
            let column = rhs.data.iter().map(|r| r[j]);
            out[i][j] = row.zip(column).map(|(a, b)| a * b).sum();
        }
        out
    }
}

// This allows us to multiply matrices of the same size together.
impl<const SIZE: usize> Mul for Matrix<f64, SIZE, SIZE> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let mut out = Matrix::new();
        for (i, j) in iproduct!(0..SIZE, 0..SIZE) {
            let row = self.data[i].iter();
            let column = rhs.data.iter().map(|r| r[j]);
            out[i][j] = row.zip(column).map(|(a, b)| a * b).sum();
        }
        out
    }
}
// Allows us to multiply a 4x4 matrix by a 4-tuple, returning a tuple.
// This can be implemented much more elegantly, but will do for now.
impl Mul<&Tuple> for Matrix<f64, 4, 4> {
    type Output = Tuple;

    fn mul(self, rhs: &Tuple) -> Tuple {
        const SIZE: usize = 4;
        let mut out = Vec::new();
        for i in 0..SIZE {
            let row = self.data[i].iter();
            let tuple_iterator = rhs.vector_copy();
            out.push(row.zip(tuple_iterator).map(|(a, b)| a * b).sum());
        }
        Tuple::new(out[0], out[1], out[2], out[3])
    }
}

impl<const ROWS: usize, const COLUMNS: usize> PartialEq for Matrix<f64, ROWS, COLUMNS> {
    fn eq(&self, other: &Self) -> bool {
        const EPSILON: f64 = 0.00001;
        let floats_close = |(a, b): (&f64, &f64)| (a - b).abs() < EPSILON;
        let lhs = self.data.iter().flatten();
        match other
            .data
            .iter()
            .flatten()
            .zip(lhs)
            .map(floats_close)
            .position(|b| b == false)
        {
            None => true,
            Some(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn construct_and_inspect_4x4() {
        let m = Matrix::from_array(&[
            [1.0, 2.0, 3.0, 4.0],
            [5.5, 6.5, 7.5, 8.5],
            [9.0, 10.0, 11.0, 12.0],
            [13.5, 14.5, 15.5, 16.5],
        ]);
        assert_eq!(m[0][0], 1.0);
        assert_eq!(m[1][0], 5.5);
        assert_eq!(m[3][0], 13.5);
        assert_eq!(m[3][2], 15.5);
    }
    #[test]
    fn construct_and_inspect_3x3() {
        let m = Matrix::from_array(&[[1.0, 2.0, 3.0], [5.5, 6.5, 7.5], [9.0, 10.0, 11.0]]);
        assert_eq!(m[0][0], 1.0);
        assert_eq!(m[1][1], 6.5);
        assert_eq!(m[2][1], 10.0);
    }
    #[test]
    fn construct_and_inspect_2x2() {
        let m = Matrix::from_array(&[[1.0, 2.0], [5.5, 6.5]]);
        assert_eq!(m[0][0], 1.0);
        assert_eq!(m[1][0], 5.5);
        assert_eq!(m[1][1], 6.5);
    }

    #[test]
    fn matrix_equality() {
        let m1 = Matrix::from_array(&[[1.0, 2.0, 3.0], [5.5, 6.5, 7.5], [9.0, 10.0, 11.0]]);
        let m2 = Matrix::from_array(&[[1.0, 2.0, 3.0], [5.5, 6.5, 7.5], [9.0, 10.0, 11.0]]);
        assert_eq!(m1, m2);
    }

    #[test]
    fn matrix_inequality() {
        let m1 = Matrix::from_array(&[[1.0, 2.0, 3.4], [5.5, 6.5, 7.5], [9.0, 10.0, 11.0]]);
        let m2 = Matrix::from_array(&[[1.0, 2.0, 3.0], [5.5, 6.5, 7.5], [9.0, 10.0, 11.0]]);
        assert_ne!(m1, m2);
    }

    #[test]
    fn matrix_multiplication() {
        let m1 = Matrix::from_array(&[
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 8.0, 7.0, 6.0],
            [5.0, 4.0, 3.0, 2.0],
        ]);
        let m2 = Matrix::from_array(&[
            [-2.0, 1.0, 2.0, 3.0],
            [3.0, 2.0, 1.0, -1.0],
            [4.0, 3.0, 6.0, 5.0],
            [1.0, 2.0, 7.0, 8.0],
        ]);
        let m3 = Matrix::from_array(&[
            [20.0, 22.0, 50.0, 48.0],
            [44.0, 54.0, 114.0, 108.0],
            [40.0, 58.0, 110.0, 102.0],
            [16.0, 26.0, 46.0, 42.0],
        ]);
        assert_eq!(m1 * m2, m3);
    }

    #[test]
    fn multiply_matrix_by_tuple() {
        let m1 = Matrix::from_array(&[
            [1.0, 2.0, 3.0, 4.0],
            [2.0, 4.0, 4.0, 2.0],
            [8.0, 6.0, 4.0, 1.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        let t = Tuple::new(1.0, 2.0, 3.0, 1.0);
        assert_eq!(m1 * &t, Tuple::new(18.0, 24.0, 33.0, 1.0));
    }

    #[test]
    fn transpose_matrix() {
        let m1 = Matrix::from_array(&[
            [1.0, 2.0, 3.0, 4.0],
            [2.0, 4.0, 4.0, 2.0],
            [8.0, 6.0, 4.0, 1.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        let m1_tran = Matrix::from_array(&[
            [1.0, 2.0, 8.0, 0.0],
            [2.0, 4.0, 6.0, 0.0],
            [3.0, 4.0, 4.0, 0.0],
            [4.0, 2.0, 1.0, 1.0],
        ]);
        assert_eq!(m1.transpose(), m1_tran);
    }

    #[test]
    fn determinant_2x2() {
        let m = Matrix::from_array(&[[1.0, 2.0], [3.0, 4.0]]);
        assert_eq!(m.determinant(), -2.0);
    }

    #[test]
    fn submatrix_3x3() {
        let m = Matrix::from_array(&[[1.0, 5.0, 0.0], [-3.0, 2.0, 7.0], [0.0, 6.0, -3.0]]);
        let subm = Matrix::from_array(&[[-3.0, 2.0], [0.0, 6.0]]);
        assert_eq!(m.submatrix(0, 2), subm);
    }

    #[test]
    fn submatrix_4x4() {
        let m = Matrix::from_array(&[
            [6.0, 1.0, 1.0, 6.0],
            [8.0, 5.0, 8.0, 6.0],
            [1.0, 0.0, 8.0, 2.0],
            [7.0, 1.0, 1.0, 1.0],
        ]);
        let subm = Matrix::from_array(&[[6.0, 1.0, 6.0], [8.0, 8.0, 6.0], [7.0, 1.0, 1.0]]);
        assert_eq!(m.submatrix(2, 1), subm);
    }

    #[test]
    fn minor_3x3() {
        let m = Matrix::from_array(&[[3.0, 5.0, 0.0], [2.0, -1.0, -7.0], [6.0, -1.0, 5.0]]);
        assert_eq!(m.minor(1, 0), 25.0);
    }

    #[test]
    fn cofactor_3x3() {
        let m = Matrix::from_array(&[[3.0, 5.0, 0.0], [2.0, -1.0, -7.0], [6.0, -1.0, 5.0]]);
        assert_eq!(m.cofactor(0, 0), -12.0);
        assert_eq!(m.cofactor(1, 0), -25.0);
    }

    #[test]
    fn determinant_3x3() {
        let m = Matrix::from_array(&[[1.0, 2.0, 6.0], [-5.0, 8.0, -4.0], [2.0, 6.0, 4.0]]);
        assert_eq!(m.determinant(), -196.0);
    }

    #[test]
    fn determinant_4x4() {
        let m = Matrix::from_array(&[
            [-2.0, -8.0, 3.0, 5.0],
            [-3.0, 1.0, 7.0, 3.0],
            [1.0, 2.0, -9.0, 6.0],
            [-6.0, 7.0, 7.0, -9.0],
        ]);
        assert_eq!(m.determinant(), -4071.0);
    }

    #[test]
    fn invertibility_check() {
        let m1 = Matrix::from_array(&[
            [-2.0, -8.0, 3.0, 5.0],
            [-3.0, 1.0, 7.0, 3.0],
            [1.0, 2.0, -9.0, 6.0],
            [-6.0, 7.0, 7.0, -9.0],
        ]);
        let m2 = Matrix::from_array(&[
            [-4.0, 2.0, -2.0, -3.0],
            [9.0, 6.0, 2.0, 6.0],
            [0.0, -5.0, 1.0, -5.0],
            [0.0, 0.0, 0.0, 0.0],
        ]);
        assert!(m1.is_invertible());
        assert!(!m2.is_invertible())
    }

    #[test]
    fn invert_4x4_matrix() {
        let m = Matrix::from_array(&[
            [-5.0, 2.0, 6.0, -8.0],
            [1.0, -5.0, 1.0, 8.0],
            [7.0, 7.0, -6.0, -7.0],
            [1.0, -3.0, 7.0, 4.0],
        ]);
        let inv = Matrix::from_array(&[
            [0.21805, 0.45113, 0.24060, -0.04511],
            [-0.80827, -1.45677, -0.44361, 0.52068],
            [-0.07895, -0.22368, -0.05263, 0.19737],
            [-0.52256, -0.81391, -0.30075, 0.30639],
        ]);
        assert_eq!(m.inverse(), inv);
    }

    #[test]
    fn translate_point() {
        let m = translation(5.0, -3.0, 2.0);
        let p = Tuple::point_new(-3.0, 4.0, 5.0);
        assert_eq!(m * &p, Tuple::point_new(2.0, 1.0, 7.0));
    }

    #[test]
    fn inverse_translate_point() {
        let m = translation(5.0, -3.0, 2.0);
        let p = Tuple::point_new(-3.0, 4.0, 5.0);
        assert_eq!(m.inverse() * &p, Tuple::point_new(-8.0, 7.0, 3.0));
    }

    #[test]
    fn translation_doesnt_change_vector() {
        let m = translation(5.0, -3.0, 2.0);
        let v = Tuple::vector_new(-3.0, 4.0, 5.0);
        assert_eq!(m * &v, v);
    }

    #[test]
    fn scale_point() {
        let m = scale(2.0, 3.0, 4.0);
        let p = Tuple::point_new(-4.0, 6.0, 8.0);
        assert_eq!(m * &p, Tuple::point_new(-8.0, 18.0, 32.0));
    }

    #[test]
    fn scale_vector() {
        let m = scale(2.0, 3.0, 4.0);
        let v = Tuple::vector_new(-4.0, 6.0, 8.0);
        assert_eq!(m * &v, Tuple::vector_new(-8.0, 18.0, 32.0));
    }

    #[test]
    fn inverse_scale_vector() {
        let m = scale(2.0, 3.0, 4.0);
        let v = Tuple::vector_new(-4.0, 6.0, 8.0);
        assert_eq!(m.inverse() * &v, Tuple::vector_new(-2.0, 2.0, 2.0));
    }

    #[test]
    fn rotate_point_about_x_axis() {
        use std::f64::consts::{PI, SQRT_2};
        let p = Tuple::point_new(0.0, 1.0, 0.0);
        let eigth_turn = rotation_x(PI / 4.0);
        let quarter_turn = rotation_x(PI / 2.0);
        assert_eq!(
            eigth_turn * &p,
            Tuple::point_new(0.0, SQRT_2 / 2.0, SQRT_2 / 2.0)
        );
        assert_eq!(quarter_turn * &p, Tuple::point_new(0.0, 0.0, 1.0));
    }

    #[test]
    fn rotate_point_about_y_axis() {
        use std::f64::consts::{PI, SQRT_2};
        let p = Tuple::point_new(0.0, 0.0, 1.0);
        let eigth_turn = rotation_y(PI / 4.0);
        let quarter_turn = rotation_y(PI / 2.0);
        assert_eq!(
            eigth_turn * &p,
            Tuple::point_new(SQRT_2 / 2.0, 0.0, SQRT_2 / 2.0)
        );
        assert_eq!(quarter_turn * &p, Tuple::point_new(1.0, 0.0, 0.0));
    }

    #[test]
    fn rotate_point_about_z_axis() {
        use std::f64::consts::{PI, SQRT_2};
        let p = Tuple::point_new(0.0, 1.0, 0.0);
        let eigth_turn = rotation_z(PI / 4.0);
        let quarter_turn = rotation_z(PI / 2.0);
        assert_eq!(
            eigth_turn * &p,
            Tuple::point_new(-SQRT_2 / 2.0, SQRT_2 / 2.0, 0.0)
        );
        assert_eq!(quarter_turn * &p, Tuple::point_new(-1.0, 0.0, 0.0));
    }

    #[test]
    fn shearing() {
        let p = Tuple::point_new(2.0, 3.0, 4.0);
        let s1 = shear(0.0, 1.0, 0.0, 0.0, 0.0, 0.0);
        let s2 = shear(0.0, 0.0, 1.0, 0.0, 0.0, 0.0);
        let s3 = shear(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let s4 = shear(0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
        let s5 = shear(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        assert_eq!(s1 * &p, Tuple::point_new(6.0, 3.0, 4.0));
        assert_eq!(s2 * &p, Tuple::point_new(2.0, 5.0, 4.0));
        assert_eq!(s3 * &p, Tuple::point_new(2.0, 7.0, 4.0));
        assert_eq!(s4 * &p, Tuple::point_new(2.0, 3.0, 6.0));
        assert_eq!(s5 * &p, Tuple::point_new(2.0, 3.0, 7.0));
    }

    #[test]
    fn transformation_sequence() {
        use std::f64::consts::PI;
        let p = Tuple::point_new(1.0, 0.0, 1.0);
        let rot = rotation_x(PI / 2.0);
        let scale = scale(5.0, 5.0, 5.0);
        let tran = translation(10.0, 5.0, 7.0);
        assert_eq!(tran * scale * rot * &p, Tuple::point_new(15.0, 0.0, 7.0));
        let transform = Matrix::identity()
            .rotate_x(PI / 2.0)
            .scale(5.0, 5.0, 5.0)
            .translate(10.0, 5.0, 7.0);
        assert_eq!(transform * &p, Tuple::point_new(15.0, 0.0, 7.0));
    }
}
