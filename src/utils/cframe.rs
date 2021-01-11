use rbx_types::{CFrame, Matrix3, Vector2, Vector3};

use super::TupleComponent;

pub trait MatrixExt {
    fn default() -> Self;
}

pub trait CFrameExt {
    fn default() -> Self;
    fn components(&self) -> TupleComponent;
    fn from_components(components: &[f32]) -> Self;
    fn mult(&self, b: CFrame) -> Self;
    fn from_xyz(x: f32, y: f32, z: f32) -> Self;
    fn from_axis_angle(axis: Vector3, theta: f32) -> Self;
    fn angles(x: f32, y: f32, z: f32) -> Self;
}

pub trait Vector3Ext {
    fn UP() -> Vector3;
    fn BACK() -> Vector3;
    fn RIGHT() -> Vector3;
    fn add(&self, b: Vector3) -> Vector3;
    fn sub(&self, b: Vector3) -> Vector3;
    fn mult(&self, b: f32) -> Vector3;
    fn mult_vec(&self, b: Vector3) -> Vector3;
    fn cross(&self, b: Vector3) -> Vector3;
    fn dot(&self, b: Vector3) -> f32;
    fn normalize(&self) -> Vector3;
    fn axis_angle(&self, v: Vector3, t: f32) -> Vector3;
}

pub trait Vector2Ext {
    fn dot(&self, b: Self) -> f32;
    fn normalize(&self) -> Self;
}

impl Vector2Ext for Vector2 {
    fn dot(&self, b: Self) -> f32 {
        self.x * b.x + self.y * b.y
    }

    fn normalize(&self) -> Self {
        let m = self.dot(self.clone());
        Vector2 {
            x: self.x / m,
            y: self.y / m,
        }
    }
}

impl Vector3Ext for Vector3 {
    fn UP() -> Vector3 {
        Vector3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }
    }

    fn BACK() -> Vector3 {
        Vector3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        }
    }

    fn RIGHT() -> Vector3 {
        Vector3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }
    }

    fn add(&self, b: Vector3) -> Vector3 {
        Vector3 {
            x: self.x + b.x,
            y: self.y + b.y,
            z: self.z + b.z,
        }
    }

    fn sub(&self, b: Vector3) -> Vector3 {
        Vector3 {
            x: self.x - b.x,
            y: self.y - b.y,
            z: self.z - b.z,
        }
    }

    fn mult(&self, i: f32) -> Vector3 {
        Vector3 {
            x: self.x * i,
            y: self.y * i,
            z: self.z * i,
        }
    }

    fn mult_vec(&self, b: Vector3) -> Vector3 {
        Vector3 {
            x: self.x * b.x,
            y: self.y * b.y,
            z: self.z * b.z,
        }
    }

    fn cross(&self, b: Vector3) -> Vector3 {
        Vector3 {
            x: self.y * b.z - b.y * self.z,
            y: self.z * b.x - b.z * self.x,
            z: self.x * b.y - b.x * self.y,
        }
    }

    fn dot(&self, b: Vector3) -> f32 {
        self.x * b.x + self.y * b.y + self.z * b.z
    }

    fn normalize(&self) -> Vector3 {
        let m = self.dot(self.clone());
        Vector3 {
            x: self.x / m,
            y: self.y / m,
            z: self.z / m,
        }
    }

    fn axis_angle(&self, axis: Vector3, t: f32) -> Vector3 {
        let unit = self.normalize();

        let cos = t.cos();
        let sin = t.sin();

        let a1 = axis.mult(cos);
        let a2 = unit.mult(axis.dot(unit) * (1.0 - cos));
        let a3 = unit.cross(axis).mult(sin);

        a1.add(a2.add(a3))
    }
}

impl CFrameExt for CFrame {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            orientation: Matrix3::default(),
        }
    }

    fn from_components(c: &[f32]) -> Self {
        assert!(c.len() >= 12);

        Self {
            position: Vector3::new(c[0], c[1], c[2]),
            orientation: Matrix3 {
                x: Vector3::new(c[3], c[4], c[5]),
                y: Vector3::new(c[6], c[7], c[8]),
                z: Vector3::new(c[9], c[10], c[11]),
            },
        }
    }

    fn components(&self) -> TupleComponent {
        let m = self.orientation;
        let a = self.position;

        (
            a.x, a.y, a.z, m.x.x, m.x.y, m.x.z, m.y.x, m.y.y, m.y.z, m.z.x, m.z.y, m.z.z, 0.0, 0.0,
            0.0, 1.0,
        )
    }

    fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: Vector3::new(x, y, z),
            orientation: Matrix3::default(),
        }
    }

    fn from_axis_angle(axis: Vector3, theta: f32) -> Self {
        let r = axis.axis_angle(Vector3::RIGHT(), theta);
        let u = axis.axis_angle(Vector3::UP(), theta);
        let b = axis.axis_angle(Vector3::BACK(), theta);
        CFrame {
            position: Vector3::new(0.0, 0.0, 0.0),
            orientation: Matrix3 {
                x: Vector3::new(r.x, u.x, b.x),
                y: Vector3::new(r.y, u.y, b.y),
                z: Vector3::new(r.z, u.z, b.z),
            },
        }
    }

    fn angles(x: f32, y: f32, z: f32) -> Self {
        let cfx = CFrame::from_axis_angle(Vector3::RIGHT(), x);
        let cfy = CFrame::from_axis_angle(Vector3::UP(), y);
        let cfz = CFrame::from_axis_angle(Vector3::BACK(), z);
        cfx.mult(cfy).mult(cfz)
    }

    fn mult(&self, b: Self) -> Self {
        let m1 = self.components();
        let m2 = b.components();

        let (ax, ay, az, a11, a12, a13, a21, a22, a23, a31, a32, a33, _, _, _, _) = m1;
        let (bx, by, bz, b11, b12, b13, b21, b22, b23, b31, b32, b33, _, _, _, _) = m2;

        let m11 = a11 * b11 + a12 * b21 + a13 * b31;
        let m12 = a11 * b12 + a12 * b22 + a13 * b32;
        let m13 = a11 * b13 + a12 * b23 + a13 * b33;
        let x = a11 * bx + a12 * by + a13 * bz + ax;
        let m21 = a21 * b11 + a22 * b21 + a23 * b31;
        let m22 = a21 * b12 + a22 * b22 + a23 * b32;
        let m23 = a21 * b13 + a22 * b23 + a23 * b33;
        let y = a21 * bx + a22 * by + a23 * bz + ay;
        let m31 = a31 * b11 + a32 * b21 + a33 * b31;
        let m32 = a31 * b12 + a32 * b22 + a33 * b32;
        let m33 = a31 * b13 + a32 * b23 + a33 * b33;
        let z = a31 * bx + a32 * by + a33 * bz + az;

        CFrame {
            position: Vector3::new(x, y, z),
            orientation: Matrix3 {
                x: Vector3::new(m11, m12, m13),
                y: Vector3::new(m21, m22, m23),
                z: Vector3::new(m31, m32, m33),
            },
        }
    }
}

impl MatrixExt for Matrix3 {
    fn default() -> Self {
        Self {
            x: Vector3::RIGHT(),
            y: Vector3::UP(),
            z: Vector3::BACK(),
        }
    }
}
