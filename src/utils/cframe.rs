use rbx_types::{CFrame, Matrix3, Vector3};

pub trait CFrameExt {
    // fn components(&self) -> [u8; 12];
    fn mult(&self, b: CFrame) -> CFrame;

    fn from_axis_angle(axis: Vector3, theta: f32) -> CFrame;
    fn angles(x: f32, y: f32, z: f32) -> CFrame;
}

pub trait Vector3Ext {
    fn UP() -> Vector3;
    fn BACK() -> Vector3;
    fn RIGHT() -> Vector3;

    fn add(&self, b: Vector3) -> Vector3;
    fn mult(&self, b: f32) -> Vector3;
    fn mult_vec(&self, b: Vector3) -> Vector3;
    fn cross(&self, b: Vector3) -> Vector3;

    fn dot(&self, b: Vector3) -> f32;
    fn normalize(&self) -> Vector3;
    fn axis_angle(&self, v: Vector3, t: f32) -> Vector3;
}

impl Vector3Ext for Vector3 {
    fn UP() -> Vector3 {
        Vector3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }
    }

    fn BACK() -> Vector3 {
        Vector3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }
    }

    fn RIGHT() -> Vector3 {
        Vector3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        }
    }

    fn add(&self, b: Vector3) -> Vector3 {
        Vector3 {
            x: self.x + b.x,
            y: self.y + b.y,
            z: self.z + b.z,
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
            x: self.x * b.z - b.y * self.z,
            y: self.z * b.x - b.z * self.x,
            z: self.x * b.y - b.x * self.y,
        }
    }

    fn dot(&self, b: Vector3) -> f32 {
        self.x * b.x + self.y * b.y + self.z * b.z
    }

    fn normalize(&self) -> Vector3 {
        let m = self.dot(self.clone());
        let nx = self.x / m;
        let ny = self.y / m;
        let nz = self.z / m;
        Vector3 {
            x: nx,
            y: nz,
            z: nz,
        }
    }

    fn axis_angle(&self, v: Vector3, t: f32) -> Vector3 {
        let n = self.normalize();
        (v.mult(t.cos()))
            .add(n.mult(self.dot(n)))
            .add(n.cross(v).mult(t.sin()))
    }
}

impl CFrameExt for CFrame {
    // fn components(&self) -> [u8; 12] {
    // [0; 12]
    // }

    fn from_axis_angle(axis: Vector3, theta: f32) -> CFrame {
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

    fn angles(x: f32, y: f32, z: f32) -> CFrame {
        let cfx = CFrame::from_axis_angle(Vector3::RIGHT(), x);
        let cfy = CFrame::from_axis_angle(Vector3::UP(), y);
        let cfz = CFrame::from_axis_angle(Vector3::BACK(), z);
        cfx.mult(cfy).mult(cfz)
    }

    fn mult(&self, b: CFrame) -> CFrame {
        let m1 = self.orientation;
        let m2 = b.orientation;

        let m11 = m1.x.x * m2.x.x + m1.x.y * m2.y.x + m1.x.z * m2.z.x;
        let m12 = m1.x.x * m2.x.y + m1.x.y * m2.y.y + m1.x.z * m2.z.y;
        let m13 = m1.x.x * m2.x.z + m1.x.y * m2.y.z + m1.x.z * m2.z.z;
        let x =
            m1.x.x * b.position.x + m1.x.y * b.position.y + m1.x.z * b.position.z + self.position.x;
        let m21 = m1.y.x * m2.x.x + m1.y.y * m2.y.x + m1.y.z * m2.z.x;
        let m22 = m1.y.x * m2.x.y + m1.y.y * m2.y.y + m1.y.z * m2.z.y;
        let m23 = m1.y.x * m2.x.z + m1.y.y * m2.y.z + m1.y.z * m2.z.z;
        let y =
            m1.y.x * b.position.x + m1.y.y * b.position.y + m1.y.z * b.position.z + self.position.y;
        let m31 = m1.z.x * m2.x.x + m1.z.y * m2.y.x + m1.z.z * m2.z.x;
        let m32 = m1.z.x * m2.x.y + m1.z.y * m2.y.y + m1.z.z * m2.z.y;
        let m33 = m1.z.x * m2.x.z + m1.z.y * m2.y.z + m1.z.z * m2.z.z;
        let z =
            m1.z.x * b.position.x + m1.z.y * b.position.y + m1.z.z * b.position.z + self.position.z;

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
