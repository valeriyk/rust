use crate::Point4d;

pub struct Mat4f {
    pub raw: [[f32; 4]; 4],
}

impl Mat4f {
    pub fn new() -> Self {
        Mat4f { raw: [[0.0; 4]; 4] }
    }
    pub fn identity() -> Self {
        Mat4f {
            raw: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
    pub fn from_rows(a: [f32; 4], b: [f32; 4], c: [f32; 4], d: [f32; 4]) -> Self {
        Mat4f { raw: [a, b, c, d] }
    }
    pub fn rotate_about_x(&self, angle_deg: f32) -> Self {
        let sin = angle_deg.to_radians().sin();
        let cos = angle_deg.to_radians().cos();
        let rx = Mat4f {
            raw: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, cos, -sin, 0.0],
                [0.0, sin, cos, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };
        self * &rx
    }

    pub fn rotate_about_y(&self, angle_deg: f32) -> Self {
        let sin = angle_deg.to_radians().sin();
        let cos = angle_deg.to_radians().cos();
        let ry = Mat4f {
            raw: [
                [cos, 0.0, sin, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [-sin, 0.0, cos, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };
        self * &ry
    }

    pub fn rotate_about_z(&self, angle_deg: f32) -> Self {
        let sin = angle_deg.to_radians().sin();
        let cos = angle_deg.to_radians().cos();
        let rz = Mat4f {
            raw: [
                [cos, -sin, 0.0, 0.0],
                [sin, cos, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };
        self * &rz
    }

    pub fn translate_xyz(&self, translation: &[f32]) -> Self {
        let t = Mat4f {
            raw: [
                [1.0, 0.0, 0.0, translation[0]],
                [0.0, 1.0, 0.0, translation[1]],
                [0.0, 0.0, 1.0, translation[2]],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };
        self * &t
    }

    pub fn scale_xyz(&self, scale: &[f32]) -> Self {
        let s = Mat4f {
            raw: [
                [scale[0], 0.0, 0.0, 0.0],
                [0.0, scale[1], 0.0, 0.0],
                [0.0, 0.0, scale[2], 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };
        self * &s
    }
}

impl<'a, 'b> core::ops::Mul<&'b Mat4f> for &'a Mat4f {
    type Output = Mat4f;

    fn mul(self, other: &'b Mat4f) -> Self::Output {
        let mut m = Mat4f::new();
        for i in 0..4 {
            for j in 0..4 {
                m.raw[i][j] = 0.0;
                for k in 0..4 {
                    m.raw[i][j] += self.raw[i][k] * other.raw[k][j];
                }
            }
        }
        m
    }
}

impl<'a> core::ops::Mul<Point4d> for &'a Mat4f {
    type Output = Point4d;

    fn mul(self, other: Point4d) -> Self::Output {
        let mut p = Point4d::new();
        for i in 0..4 {
            p[i] = 0.0;
            for j in 0..4 {
                p[i] += self.raw[i][j] * other[j];
            }
        }
        p
    }
}
