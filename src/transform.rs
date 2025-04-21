use crate::matrix::SqMatrix;

impl SqMatrix<4> {
    pub fn translation(x: f64, y: f64, z: f64) -> SqMatrix<4> {
        let mut mat = SqMatrix::<4>::eye();
        mat[(0, 3)] = x;
        mat[(1, 3)] = y;
        mat[(2, 3)] = z;
        mat[(3, 3)] = 1.0;
        mat
    }
    pub fn scaling(x: f64, y: f64, z: f64) -> SqMatrix<4> {
        let mut mat = SqMatrix::<4>::new();
        mat[(0, 0)] = x;
        mat[(1, 1)] = y;
        mat[(2, 2)] = z;
        mat[(3, 3)] = 1.0;
        mat
    }
    pub fn rotation_x(r: f64) -> SqMatrix<4> {
        let mut mat = SqMatrix::<4>::eye();
        mat[(1, 1)] = f64::cos(r);
        mat[(1, 2)] = -f64::sin(r);
        mat[(2, 1)] = f64::sin(r);
        mat[(2, 2)] = f64::cos(r);
        mat
    }
    pub fn rotation_y(r: f64) -> SqMatrix<4> {
        let mut mat = SqMatrix::<4>::eye();
        mat[(0, 0)] = f64::cos(r);
        mat[(0, 2)] = f64::sin(r);
        mat[(2, 0)] = -f64::sin(r);
        mat[(2, 2)] = f64::cos(r);
        mat
    }
    pub fn rotation_z(r: f64) -> SqMatrix<4> {
        let mut mat = SqMatrix::<4>::eye();
        mat[(0, 0)] = f64::cos(r);
        mat[(0, 1)] = -f64::sin(r);
        mat[(1, 0)] = f64::sin(r);
        mat[(1, 1)] = f64::cos(r);
        mat
    }
    pub fn shearing(xy: f64, xz: f64, yx: f64, yz: f64, zx: f64, zy: f64) -> SqMatrix<4> {
        let mut mat = SqMatrix::<4>::eye();
        mat[(0, 1)] = xy;
        mat[(0, 2)] = xz;
        mat[(1, 0)] = yx;
        mat[(1, 2)] = yz;
        mat[(2, 0)] = zx;
        mat[(2, 1)] = zy;
        mat
    }
}

#[cfg(test)]
pub mod tests {
    use crate::matrix::Matrix;
    use crate::vec4::Vec4;
    use std::f64::consts::PI;

    #[test]
    fn translate() {
        let t = Matrix::translation(5.0, -3.0, 2.0);
        let m = Vec4::point(-3.0, 4.0, 5.0);
        assert!(&t * &m == Vec4::point(2.0, 1.0, 7.0));
        let m = Vec4::vector(-3.0, 4.0, 5.0);
        assert!(&t * &m == m);
    }
    #[test]
    fn scale() {
        let t = Matrix::scaling(2.0, 3.0, 4.0);
        let p = Vec4::point(-4.0, 6.0, 8.0);
        assert!(&t * &p == Vec4::point(-8.0, 18.0, 32.0));
        let p = Vec4::vector(-4.0, 6.0, 8.0);
        assert!(&t * &p == Vec4::vector(-8.0, 18.0, 32.0))
    }
    #[test]
    fn rotation_x() {
        let mut p = Vec4::point(0.0, 1.0, 0.0);
        let r = Matrix::rotation_x(2.0 * PI / 3.0);
        p = &r * &p;
        p = &r * &p;
        p = &r * &p;

        assert!(p == Vec4::point(0.0, 1.0, 0.0));
    }

    #[test]
    fn test_individual_and_chained_transformations_rotation_x() {
        let p = Vec4::point(1.0, 0.0, 1.0);

        let a = Matrix::rotation_x(std::f64::consts::FRAC_PI_2); // π / 2
        let b = Matrix::scaling(5.0, 5.0, 5.0);
        let c = Matrix::translation(10.0, 5.0, 7.0);

        let p2 = &a * &p;
        assert_eq!(p2, Vec4::point(1.0, -1.0, 0.0));

        let p3 = &b * &p2;
        assert_eq!(p3, Vec4::point(5.0, -5.0, 0.0));

        let p4 = &c * &p3;
        assert_eq!(p4, Vec4::point(15.0, 0.0, 7.0));

        let t = &(c * b) * &a;
        let p_result = &t * &p;
        assert_eq!(p_result, Vec4::point(15.0, 0.0, 7.0));
    }

    #[test]
    fn test_individual_and_chained_transformations_rotation_y() {
        let p = Vec4::point(0.0, 0.0, 1.0);

        // Transformation matrices
        let a = Matrix::rotation_y(std::f64::consts::FRAC_PI_2); // π / 2
        let b = Matrix::scaling(5.0, 5.0, 5.0);
        let c = Matrix::translation(10.0, 5.0, 7.0);

        // Apply transformations one at a time
        let p2 = &a * &p;
        assert_eq!(p2, Vec4::point(1.0, 0.0, 0.0));

        let p3 = &b * &p2;
        assert_eq!(p3, Vec4::point(5.0, 0.0, 0.0));

        let p4 = &c * &p3;
        assert_eq!(p4, Vec4::point(15.0, 5.0, 7.0));

        // Apply chained transformation
        let t = &(c * b) * &a;
        let p_result = &t * &p;
        assert_eq!(p_result, Vec4::point(15.0, 5.0, 7.0));
    }

    #[test]
    fn test_individual_and_chained_transformations_rotation_z() {
        let p = Vec4::point(0.0, 1.0, 0.0);

        // Transformation matrices
        let a = Matrix::rotation_z(std::f64::consts::FRAC_PI_2); // π / 2
        let b = Matrix::scaling(5.0, 5.0, 5.0);
        let c = Matrix::translation(10.0, 5.0, 7.0);

        // Apply transformations one at a time
        let p2 = &a * &p;
        assert_eq!(p2, Vec4::point(-1.0, 0.0, 0.0));

        let p3 = &b * &p2;
        assert_eq!(p3, Vec4::point(-5.0, 0.0, 0.0));

        let p4 = &c * &p3;
        assert_eq!(p4, Vec4::point(5.0, 5.0, 7.0));

        // Apply chained transformation
        let t = &(c * b) * &a;
        let p_result = &t * &p;
        assert_eq!(p_result, Vec4::point(5.0, 5.0, 7.0));
    }
}
