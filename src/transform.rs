use geometry::Vector;

#[derive(Debug, Clone)]
pub struct Matrix4x4 {
    m: [[f32; 4]; 4]
}

impl ::std::convert::From<[[f32; 4]; 4]> for Matrix4x4 {
    fn from(mat: [[f32; 4]; 4]) -> Matrix4x4 {
        Matrix4x4 { m: mat }
    }
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
        panic!("Not implemented!")
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

#[derive(Debug, Clone)]
pub struct Transform {
    // Transform private data
    m: Matrix4x4,
    m_inv: Matrix4x4
}

impl Transform {
    // Transform public methods
    fn new() -> Transform {
        Transform { m: Matrix4x4::new(), m_inv: Matrix4x4::new() }
    }

    fn new_with(_m: Matrix4x4, _inv: Matrix4x4) -> Transform {
        Transform { m: _m, m_inv: _inv }
    }

    fn invert(self) -> Transform {
        Transform::new_with(self.m_inv, self.m)
    }

    fn inverse(&self) -> Transform {
        self.clone().invert()
    }

    fn translate(v: &Vector) -> Transform {
        let m = Matrix4x4::new_with(
            1f32, 0f32, 0f32, v.x,
            0f32, 1f32, 0f32, v.y,
            0f32, 0f32, 1f32, v.z,
            0f32, 0f32, 0f32, 1f32);
        let m_inv = Matrix4x4::new_with(
            1f32, 0f32, 0f32, -v.x,
            0f32, 1f32, 0f32, -v.y,
            0f32, 0f32, 1f32, -v.z,
            0f32, 0f32, 0f32, 1f32);
        Transform::new_with(m, m_inv)
    }
}

impl ::std::convert::From<Matrix4x4> for Transform {
    fn from(m: Matrix4x4) -> Transform {
        let inv = m.inverse();
        Transform::new_with(m, inv)
    }
}

impl ::std::convert::From<[[f32; 4]; 4]> for Transform {
    fn from(mat: [[f32; 4]; 4]) -> Transform {
        Transform::from(Matrix4x4::from(mat))
    }
}
