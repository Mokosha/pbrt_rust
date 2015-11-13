use quaternion::Quaternion;
use utils::Lerp;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Matrix4x4 {
    pub m: [[f32; 4]; 4]
}

impl Matrix4x4 {
    pub fn new() -> Matrix4x4 {
        Matrix4x4 {
            m: [[1f32, 0f32, 0f32, 0f32],
                [0f32, 1f32, 0f32, 0f32],
                [0f32, 0f32, 1f32, 0f32],
                [0f32, 0f32, 0f32, 1f32]]
        }
    }

    pub fn new_with(
        t00: f32, t01: f32, t02: f32, t03: f32,
        t10: f32, t11: f32, t12: f32, t13: f32,
        t20: f32, t21: f32, t22: f32, t23: f32,
        t30: f32, t31: f32, t32: f32, t33: f32) -> Matrix4x4 {
        Matrix4x4 {
            m: [[t00, t01, t02, t03],
                [t10, t11, t12, t13],
                [t20, t21, t22, t23],
                [t30, t31, t32, t33]]
        }
    }

    pub fn transpose(self) -> Matrix4x4 {
        Matrix4x4::new_with(
            self.m[0][0], self.m[1][0], self.m[2][0], self.m[3][0],
            self.m[0][1], self.m[1][1], self.m[2][1], self.m[3][1],
            self.m[0][2], self.m[1][2], self.m[2][2], self.m[3][2],
            self.m[0][3], self.m[1][3], self.m[2][3], self.m[3][3])        
    }

    pub fn invert(self) -> Matrix4x4 {
        // Book says to use a numerically stable Gauss-Jordan elimination routine
        unimplemented!()
    }

    pub fn inverse(&self) -> Matrix4x4 {
        self.clone().invert()
    }
}

impl<'a, 'b> ::std::ops::Mul<&'a Matrix4x4> for &'b Matrix4x4 {
    type Output = Matrix4x4;
    fn mul(self, m: &'a Matrix4x4) -> Matrix4x4 {
        let mut r = Matrix4x4::new();
        for i in 0..4 {
            for j in 0..4 {
                r.m[i][j] =
                    self.m[i][0] * m.m[0][j] +
                    self.m[i][1] * m.m[1][j] +
                    self.m[i][2] * m.m[2][j] +
                    self.m[i][3] * m.m[3][j];
            }
        }
        r
    }
}

impl ::std::ops::Mul for Matrix4x4 {
    type Output = Matrix4x4;
    fn mul(self, m: Matrix4x4) -> Matrix4x4 {
        &self * &m
    }
}

impl<'a> ::std::ops::Mul<&'a Matrix4x4> for Matrix4x4 {
    type Output = Matrix4x4;
    fn mul(self, m: &'a Matrix4x4) -> Matrix4x4 {
        &self * m
    }
}

impl<'a> ::std::ops::Mul<Matrix4x4> for &'a Matrix4x4 {
    type Output = Matrix4x4;
    fn mul(self, m: Matrix4x4) -> Matrix4x4 {
        self * &m
    }
}

impl<'a, 'b> ::std::ops::Add<&'b Matrix4x4> for &'a Matrix4x4 {
    type Output = Matrix4x4;
    fn add(self, m: &'b Matrix4x4) -> Matrix4x4 {
        Matrix4x4::new_with(
            &self[0][0] + m[0][0], &self[0][1] + m[0][1], &self[0][2] + m[0][2], &self[0][3] + m[0][3],
            &self[1][0] + m[1][0], &self[1][1] + m[1][1], &self[1][2] + m[1][2], &self[1][3] + m[1][3],
            &self[2][0] + m[2][0], &self[2][1] + m[2][1], &self[2][2] + m[2][2], &self[2][3] + m[2][3],
            &self[3][0] + m[3][0], &self[3][1] + m[3][1], &self[3][2] + m[3][2], &self[3][3] + m[3][3])
    }
}

impl<'a> ::std::ops::Add<Matrix4x4> for &'a Matrix4x4 {
    type Output = Matrix4x4;
    fn add(self, m: Matrix4x4) -> Matrix4x4 { self + &m }
}

impl<'a> ::std::ops::Add<&'a Matrix4x4> for Matrix4x4 {
    type Output = Matrix4x4;
    fn add(self, m: &'a Matrix4x4) -> Matrix4x4 { &self + m }
}

impl ::std::ops::Add for Matrix4x4 {
    type Output = Matrix4x4;
    fn add(self, m: Matrix4x4) -> Matrix4x4 { &self + &m }
}

impl<'a> ::std::ops::Mul<f32> for &'a Matrix4x4 {
    type Output = Matrix4x4;
    fn mul(self, s: f32) -> Matrix4x4 {
        Matrix4x4::new_with(
            &self[0][0] * s, &self[0][1] * s, &self[0][2] * s, &self[0][3] * s,
            &self[1][0] * s, &self[1][1] * s, &self[1][2] * s, &self[1][3] * s,
            &self[2][0] * s, &self[2][1] * s, &self[2][2] * s, &self[2][3] * s,
            &self[3][0] * s, &self[3][1] * s, &self[3][2] * s, &self[3][3] * s)
    }
}

impl<'a> ::std::ops::Mul<&'a Matrix4x4> for f32 {
    type Output = Matrix4x4;
    fn mul(self, m: &'a Matrix4x4) -> Matrix4x4 { m * self }
}

impl ::std::ops::Mul<f32> for Matrix4x4 {
    type Output = Matrix4x4;
    fn mul(self, s: f32) -> Matrix4x4 { &self * s }
}

impl ::std::ops::Mul<Matrix4x4> for f32 {
    type Output = Matrix4x4;
    fn mul(self, m: Matrix4x4) -> Matrix4x4 { &m * self }
}

impl ::std::ops::Index<i32> for Matrix4x4 {
    type Output = [f32; 4];
    fn index<'a>(&'a self, index: i32) -> &'a [f32; 4] {
        match index {
            0 => &self.m[0],
            1 => &self.m[1],
            2 => &self.m[2],
            3 => &self.m[3],
            _ => panic!("Error - Matrix4x4 index out of bounds!")
        }
    }
}

impl ::std::ops::IndexMut<i32> for Matrix4x4 {
    fn index_mut<'a>(&'a mut self, index: i32) -> &'a mut [f32; 4] {
        match index {
            0 => &mut self.m[0],
            1 => &mut self.m[1],
            2 => &mut self.m[2],
            3 => &mut self.m[3],
            _ => panic!("Error - Matrix4x4 index out of bounds!")
        }
    }
}

impl Lerp<f32> for Matrix4x4 {
    fn lerp(&self, b: &Matrix4x4, t: f32) -> Matrix4x4 {
        (1f32 - t) * self + t * b
    }
}

impl ::std::convert::From<[[f32; 4]; 4]> for Matrix4x4 {
    fn from(mat: [[f32; 4]; 4]) -> Matrix4x4 {
        Matrix4x4 { m: mat }
    }
}

impl ::std::convert::From<Matrix4x4> for Quaternion {
    fn from(m: Matrix4x4) -> Quaternion {
        // According to the text, the implementation of this function, along
        // with numerical stability problems, can be found in:
        // "Quaternions and 4x4 matrices" By K. Shoemake (1991)
        // Graphics Gems II, pp. 351-54
        let r = &m;
        let trace = r[0][0] + r[1][1] + r[2][2];
        if (trace > 0f32) {
            let s = 0.5f32 / ((trace + 1f32).sqrt());
            Quaternion::new_with(
                (r[2][1] - r[1][2]) * s,
                (r[0][2] - r[2][0]) * s,
                (r[1][0] - r[0][1]) * s,
                0.25f32 / s)
        } else {
            if (r[0][0] > r[1][1] && r[0][0] > r[2][2]) {
                let s = 0.5f32 / ((1f32 + r[0][0] - r[1][1] - r[2][2]).sqrt());
                Quaternion::new_with(
                    0.25f32 / s,
                    (r[0][1] + r[1][0]) * s,
                    (r[0][2] + r[2][0]) * s,
                    (r[2][1] - r[1][2]) * s)
            } else if (r[1][1] > r[2][2]) {
                let s = 0.5f32 / ((1f32 + r[1][1] - r[0][0] - r[2][2]).sqrt());
                Quaternion::new_with(
                    (r[0][1] + r[1][0]) * s,
                    0.25f32 / s,
                    (r[1][2] + r[2][1]) * s,
                    (r[0][2] - r[2][0]) * s)
            } else {
                let s = 0.5f32 / ((1f32 + r[2][2] - r[0][0] - r[1][1]).sqrt());
                Quaternion::new_with(
                    (r[0][2] + r[2][0]) * s,
                    (r[1][2] + r[2][1]) * s,
                    0.25f32 / s,
                    (r[1][0] - r[0][1]) * s)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_be_created() {
        assert_eq!(Matrix4x4::new(),
                   Matrix4x4 {
                       m: [[1.0, 0.0, 0.0, 0.0],
                           [0.0, 1.0, 0.0, 0.0],
                           [0.0, 0.0, 1.0, 0.0],
                           [0.0, 0.0, 0.0, 1.0]]});
    }

    #[test]
    fn it_can_be_created_with_values() {
        assert_eq!(Matrix4x4::new(),
                   Matrix4x4::new_with(
                       1.0, 0.0, 0.0, 0.0,
                       0.0, 1.0, 0.0, 0.0,
                       0.0, 0.0, 1.0, 0.0,
                       0.0, 0.0, 0.0, 1.0));

        assert_eq!(Matrix4x4::new_with(1.0, 2.0, 3.0, 4.0,
                                       4.0, 3.0, 2.0, 1.0,
                                       -1.0, 2.0, -3.0, 4.0,
                                       0.0, 0.0, 0.0, 0.0),
                   Matrix4x4 { m: [[1.0, 2.0, 3.0, 4.0],
                                   [4.0, 3.0, 2.0, 1.0],
                                   [-1.0, 2.0, -3.0, 4.0],
                                   [0.0, 0.0, 0.0, 0.0]]});
    }

    #[test]
    fn it_can_be_created_from_arrays() {
        assert_eq!(Matrix4x4::new(),
                   Matrix4x4::from(
                       [[1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0]]));

        assert_eq!(Matrix4x4::new_with(1.0, 2.0, 3.0, 4.0,
                                       4.0, 3.0, 2.0, 1.0,
                                       -1.0, 2.0, -3.0, 4.0,
                                       0.0, 0.0, 0.0, 0.0),
                   Matrix4x4::from([[1.0, 2.0, 3.0, 4.0],
                                    [4.0, 3.0, 2.0, 1.0],
                                    [-1.0, 2.0, -3.0, 4.0],
                                    [0.0, 0.0, 0.0, 0.0]]));
    }

    #[test]
    fn it_can_be_transposed() {
        assert_eq!(Matrix4x4::new().transpose(), Matrix4x4::new());
        assert_eq!(Matrix4x4::new_with(1.0, 2.0, 3.0, 4.0,
                                       4.0, 3.0, 2.0, 1.0,
                                       -1.0, 2.0, -3.0, 4.0,
                                       0.0, 0.0, 0.0, 0.0).transpose(),
                   Matrix4x4::new_with(1.0, 4.0, -1.0, 0.0,
                                       2.0, 3.0, 2.0, 0.0,
                                       3.0, 2.0, -3.0, 0.0,
                                       4.0, 1.0, 4.0, 0.0));
    }

    #[test]
    fn they_can_be_multiplied() {
        let id = Matrix4x4::new();
        assert_eq!(&id * &id, id);
        assert_eq!(id.clone() * &id, id);
        assert_eq!(&id * id.clone(), id);
        assert_eq!(id.clone() * id.clone(), id);

        let m1 = Matrix4x4::new_with(1.0, 4.0, -1.0, 0.0,
                                     2.0, 3.0, 2.0, 0.0,
                                     3.0, 2.0, -3.0, 0.0,
                                     4.0, 1.0, 4.0, 0.0);

        let m2 = Matrix4x4::new_with(3.0, -2.0, -1.0, 0.0,
                                     0.0, 0.1, -2.0, 3.0,
                                     2.0, 6.0, 3.0, 0.0,
                                     6.0, 6.0, 1.0, 1.0);

        assert_eq!(&m1 * &id, m1);
        assert_eq!(m1.clone() * &id, m1);
        assert_eq!(&m1 * id.clone(), m1);
        assert_eq!(m1.clone() * id.clone(), m1);

        assert_eq!(&id * &m2, m2);
        assert_eq!(id.clone() * &m2, m2);
        assert_eq!(&id * m2.clone(), m2);
        assert_eq!(id.clone() * m2.clone(), m2);

        let result = Matrix4x4::new_with(1.0, -7.6, -12.0, 12.0,
                                         10.0, 8.3, -2.0, 9.0,
                                         3.0, -23.8, -16.0, 6.0,
                                         20.0, 16.1, 6.0, 3.0);

        assert_eq!(&m1 * &m2, result);
        assert_eq!(m1.clone() * &m2, result);
        assert_eq!(&m1 * m2.clone(), result);
        assert_eq!(m1.clone() * m2.clone(), result);

        assert!((m2 * m1).ne(&result));
    }
}
