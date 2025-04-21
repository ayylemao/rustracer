use crate::math::ApproxEq;
use crate::vec4::Vec4;
use std::fmt;
use std::ops::{Add, AddAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};

pub type SqMatrix<const N: usize> = Matrix<N, N>;

#[derive(Debug, Clone)]
pub struct Matrix<const ROWS: usize, const COLS: usize> {
    data: Vec<f64>,
}

impl<const ROWS: usize, const COLS: usize> Matrix<ROWS, COLS> {
    pub fn new() -> Self {
        let rows = ROWS;
        let cols = COLS;
        let total = rows * cols;
        Matrix {
            data: vec![0.0; total],
        }
    }
    pub fn from_array(array: [[f64; COLS]; ROWS]) -> Self {
        let mut mat: Matrix<ROWS, COLS> = Matrix::new();
        for row in 0..ROWS {
            for col in 0..COLS {
                mat[(row, col)] = array[row][col];
            }
        }
        mat
    }
    pub fn transpose(&self) -> Matrix<COLS, ROWS> {
        let mut transposed: Matrix<COLS, ROWS> = Matrix::new();
        for i in 0..ROWS {
            for j in 0..COLS {
                transposed[(j, i)] = self[(i, j)];
            }
        }
        transposed
    }
}

impl<const N: usize> Matrix<N, N> {
    pub fn eye() -> Self {
        let mut mat: Matrix<N, N> = Matrix::new();
        for i in 0..N {
            mat[(i, i)] = 1.0;
        }
        mat
    }
}

impl Matrix<4, 4> {
    fn submatrix(&self, row: usize, col: usize) -> Matrix<3, 3> {
        let mut result = Matrix::<3, 3>::new();
        let mut dst_row = 0;
        for src_row in 0..4 {
            if src_row == row {
                continue;
            }
            let mut dst_col = 0;
            for src_col in 0..4 {
                if src_col == col {
                    continue;
                }
                result[(dst_row, dst_col)] = self[(src_row, src_col)];
                dst_col += 1;
            }
            dst_row += 1;
        }
        result
    }
}

impl Matrix<3, 3> {
    fn submatrix(&self, row: usize, col: usize) -> Matrix<2, 2> {
        let mut result = Matrix::<2, 2>::new();
        let mut dst_row = 0;
        for src_row in 0..3 {
            if src_row == row {
                continue;
            }
            let mut dst_col = 0;
            for src_col in 0..3 {
                if src_col == col {
                    continue;
                }
                result[(dst_row, dst_col)] = self[(src_row, src_col)];
                dst_col += 1;
            }
            dst_row += 1;
        }
        result
    }

    fn minor(&self, row: usize, col: usize) -> f64 {
        self.submatrix(row, col).det()
    }

    fn cofactor(&self, row: usize, col: usize) -> f64 {
        if row + col % 2 == 0 {
            return self.minor(row, col);
        }
        -self.minor(row, col)
    }
}

impl Matrix<2, 2> {
    pub fn det(&self) -> f64 {
        self[(0,0)]*self[(1,1)] - self[(0,1)]*self[(1,0)]
    }
}

impl<const ROWS: usize, const COLS: usize> Index<(usize, usize)> for Matrix<ROWS, COLS> {
    type Output = f64;
    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        debug_assert!(row < ROWS && col < COLS, "Index out of bounds");
        &self.data[row * COLS + col]
    }
}

impl<const ROWS: usize, const COLS: usize> IndexMut<(usize, usize)> for Matrix<ROWS, COLS> {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        debug_assert!(row < ROWS && col < COLS, "Index out of bounds");
        &mut self.data[row * COLS + col]
    }
}

impl<const ROWS: usize, const COLS: usize> fmt::Display for Matrix<ROWS, COLS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in 0..ROWS {
            write!(f, "[")?;
            for col in 0..COLS {
                let val = self[(row, col)];
                write!(f, "{:8.3}", val)?;
                if col < COLS - 1 {
                    write!(f, ", ")?;
                }
            }
            writeln!(f, "]")?;
        }
        Ok(())
    }
}

impl<const ROWS: usize, const COLS: usize> PartialEq for Matrix<ROWS, COLS> {
    fn eq(&self, other: &Self) -> bool {
        for i in 0..self.data.len() {
            if !self.data[i].approx_eq(&other.data[i]) {
                return false;
            }
        }
        true
    }
}

impl<const N: usize> Mul for Matrix<N, N> {
    type Output = Matrix<N, N>;
    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = Matrix::<N, N>::new();
        for row in 0..N {
            for col in 0..N {
                let mut sum = 0.0;
                for k in 0..N {
                    sum += self[(row, k)] * rhs[(k, col)];
                }
                result[(row, col)] = sum;
            }
        }
        result
    }
}

impl Mul<Vec4> for Matrix<4, 4> {
    type Output = Vec4;
    fn mul(self, rhs: Vec4) -> Self::Output {
        let components = [rhs.x, rhs.y, rhs.z, rhs.w];
        let mut result = [0.0; 4];

        for row in 0..4 {
            for col in 0..4 {
                result[row] += self[(row, col)] * components[col];
            }
        }
        Vec4::from_array(result)
    }
}

#[cfg(test)]
pub mod tests {
    use std::ffi::c_char;

    use super::*;

    #[test]
    fn constructing_and_inspecting_a_4x4_matrix() {
        let mut m: Matrix<4, 4> = Matrix::new();

        // Row 0
        m[(0, 0)] = 1.0;
        m[(0, 1)] = 2.0;
        m[(0, 2)] = 3.0;
        m[(0, 3)] = 4.0;

        // Row 1
        m[(1, 0)] = 5.5;
        m[(1, 1)] = 6.5;
        m[(1, 2)] = 7.5;
        m[(1, 3)] = 8.5;

        // Row 2
        m[(2, 0)] = 9.0;
        m[(2, 1)] = 10.0;
        m[(2, 2)] = 11.0;
        m[(2, 3)] = 12.0;

        // Row 3
        m[(3, 0)] = 13.5;
        m[(3, 1)] = 14.5;
        m[(3, 2)] = 15.5;
        m[(3, 3)] = 16.5;

        // Assertions matching the scenario
        assert_eq!(m[(0, 0)], 1.0);
        assert_eq!(m[(0, 3)], 4.0);
        assert_eq!(m[(1, 0)], 5.5);
        assert_eq!(m[(1, 2)], 7.5);
        assert_eq!(m[(2, 2)], 11.0);
        assert_eq!(m[(3, 0)], 13.5);
        assert_eq!(m[(3, 2)], 15.5);
    }

    #[test]
    fn a_2x2_matrix_ought_to_be_representable() {
        let mut m: Matrix<2, 2> = Matrix::new();

        m[(0, 0)] = -3.0;
        m[(0, 1)] = 5.0;
        m[(1, 0)] = 1.0;
        m[(1, 1)] = -2.0;

        assert_eq!(m[(0, 0)], -3.0);
        assert_eq!(m[(0, 1)], 5.0);
        assert_eq!(m[(1, 0)], 1.0);
        assert_eq!(m[(1, 1)], -2.0);
    }

    #[test]
    fn a_3x3_matrix_ought_to_be_representable() {
        let mut m: Matrix<3, 3> = Matrix::new();

        m[(0, 0)] = -3.0;
        m[(0, 1)] = 5.0;
        m[(0, 2)] = 0.0;

        m[(1, 0)] = 1.0;
        m[(1, 1)] = -2.0;
        m[(1, 2)] = -7.0;

        m[(2, 0)] = 0.0;
        m[(2, 1)] = 1.0;
        m[(2, 2)] = 1.0;

        assert_eq!(m[(0, 0)], -3.0);
        assert_eq!(m[(1, 1)], -2.0);
        assert_eq!(m[(2, 2)], 1.0);
    }

    #[test]
    fn matrix_equality_with_identical_matrices() {
        let mut a: Matrix<4, 4> = Matrix::new();
        let mut b: Matrix<4, 4> = Matrix::new();

        // Fill both matrices with the same values
        let values = [
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 8.0, 7.0, 6.0],
            [5.0, 4.0, 3.0, 2.0],
        ];

        for row in 0..4 {
            for col in 0..4 {
                a[(row, col)] = values[row][col];
                b[(row, col)] = values[row][col];
            }
        }

        assert_eq!(a, b);
    }

    #[test]
    fn construct_from_array() {
        let values = [
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 8.0, 7.0, 6.0],
            [5.0, 4.0, 3.0, 2.0],
        ];
        let mat = Matrix::from_array(values);
        println!("{}", mat);
    }
    #[test]
    fn mat_mul() {
        let a_values = [
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 8.0, 7.0, 6.0],
            [5.0, 4.0, 3.0, 2.0],
        ];

        let b_values = [
            [-2.0, 1.0, 2.0, 3.0],
            [3.0, 2.0, 1.0, -1.0],
            [4.0, 3.0, 6.0, 5.0],
            [1.0, 2.0, 7.0, 8.0],
        ];

        let expected_values = [
            [20.0, 22.0, 50.0, 48.0],
            [44.0, 54.0, 114.0, 108.0],
            [40.0, 58.0, 110.0, 102.0],
            [16.0, 26.0, 46.0, 42.0],
        ];

        let a = Matrix::from_array(a_values);
        let b = Matrix::from_array(b_values);
        let c = Matrix::from_array(expected_values);

        let mult = a * b;
        assert_eq!(mult, c);
    }

    #[test]
    fn mat_vec_mult() {
        let mat_values = [
            [1.0, 2.0, 3.0, 4.0],
            [2.0, 4.0, 4.0, 2.0],
            [8.0, 6.0, 4.0, 1.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let mat_a = Matrix::from_array(mat_values);
        let vec_b = Vec4::from_array([1.0, 2.0, 3.0, 1.0]);
        let vec_result = Vec4::from_array([18.0, 24.0, 33.0, 1.0]);
        assert_eq!(mat_a * vec_b, vec_result);
    }

    #[test]
    fn eye_mult() {
        let mat_values = [
            [1.0, 2.0, 3.0, 4.0],
            [2.0, 4.0, 4.0, 2.0],
            [8.0, 6.0, 4.0, 1.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let mat_a = Matrix::from_array(mat_values);
        let mat_eye: SqMatrix<4> = Matrix::eye();
        let mat_b = mat_a.clone();
        assert_eq!((mat_a * mat_eye), mat_b);
    }

    #[test]
    fn transpose() {
        let val = [
            [1.0, 2.0, 3.0],
            [4.0, 5.0, 6.0]
        ];

        let t_val = [
            [1.0, 4.0],
            [2.0, 5.0],
            [3.0, 6.0]
        ];
        let mat = Matrix::from_array(val);
        let t = mat.transpose();
        let t_test = Matrix::from_array(t_val);
        assert_eq!(t_test, t);
        assert_eq!(mat.transpose().transpose(), mat);
    }

    #[test]
    fn submatrix() {
        let val = [
            [1.0, 2.0, 3.0],
            [4.0, 5.0, 6.0],
            [7.0, 8.0, 9.0]
        ];
        let mat = Matrix::from_array(val);
        let sub = mat.submatrix(1, 1);
        assert_eq!(Matrix::from_array([[1.0, 3.0], [7.0, 9.0]]), sub);
    }
    #[test]
    fn minor() {
        let val = [
            [3.0, 5.0, 0.0],
            [2.0, -1.0, -7.0],
            [6.0, -1.0, 5.0]
        ];
        let mat = Matrix::from_array(val);
        let det = mat.minor(1, 0);
        assert_eq!(det, 25.0);
    }
    #[test]
    fn cofactor() {
        let val = [
            [3.0, 5.0, 0.0],
            [2.0, -1.0, -7.0],
            [6.0, -1.0, 5.0]
        ];
        let mat = Matrix::from_array(val);
        let minor1 = mat.minor(0, 0);
        assert_eq!(minor1, -12.0);
        let cofactor1 = mat.cofactor(0, 0);
        assert_eq!(cofactor1, -12.0);
        let minor2 = mat.minor(1, 0);
        assert_eq!(minor2, 25.0);
        let cofactor2 = mat.cofactor(1, 0);
        assert_eq!(cofactor2, -25.0);


    }
}
