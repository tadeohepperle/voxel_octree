#[macro_export]
macro_rules! pos {
    ($x:expr,$y:expr,$z:expr) => {{
        PosU8 {
            x: $x,
            y: $y,
            z: $z,
        }
    }};
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PosU8 {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

impl PosU8 {
    pub const X: Self = Self { x: 1, y: 0, z: 0 };
    pub const Y: Self = Self { x: 0, y: 1, z: 0 };
    pub const Z: Self = Self { x: 0, y: 0, z: 1 };
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };

    pub fn new(x: u8, y: u8, z: u8) -> Self {
        PosU8 { x, y, z }
    }

    pub fn plus_x(&self) -> Self {
        PosU8 {
            x: self.x + 1,
            y: self.y,
            z: self.z,
        }
    }

    pub fn minus_x(&self) -> Self {
        PosU8 {
            x: self.x - 1,
            y: self.y,
            z: self.z,
        }
    }

    pub fn plus_y(&self) -> Self {
        PosU8 {
            x: self.x,
            y: self.y + 1,
            z: self.z,
        }
    }

    pub fn minus_y(&self) -> Self {
        PosU8 {
            x: self.x,
            y: self.y - 1,
            z: self.z,
        }
    }

    pub fn plus_z(&self) -> Self {
        PosU8 {
            x: self.x,
            y: self.y,
            z: self.z + 1,
        }
    }

    pub fn minus_z(&self) -> Self {
        PosU8 {
            x: self.x,
            y: self.y,
            z: self.z - 1,
        }
    }

    pub fn plus_xy(&self) -> Self {
        PosU8 {
            x: self.x + 1,
            y: self.y + 1,
            z: self.z,
        }
    }

    pub fn minus_xy(&self) -> Self {
        PosU8 {
            x: self.x - 1,
            y: self.y - 1,
            z: self.z,
        }
    }

    pub fn plus_xz(&self) -> Self {
        PosU8 {
            x: self.x + 1,
            y: self.y,
            z: self.z + 1,
        }
    }

    pub fn minus_xz(&self) -> Self {
        PosU8 {
            x: self.x - 1,
            y: self.y,
            z: self.z - 1,
        }
    }

    pub fn plus_yz(&self) -> Self {
        PosU8 {
            x: self.x,
            y: self.y + 1,
            z: self.z + 1,
        }
    }

    pub fn plus_xyz(&self) -> Self {
        PosU8 {
            x: self.x + 1,
            y: self.y + 1,
            z: self.z + 1,
        }
    }

    pub fn minus_xyz(&self) -> Self {
        PosU8 {
            x: self.x - 1,
            y: self.y - 1,
            z: self.z - 1,
        }
    }
}

impl From<PosU8> for [f32; 3] {
    fn from(pos: PosU8) -> Self {
        [pos.x as f32, pos.y as f32, pos.z as f32]
    }
}

impl std::ops::Add for PosU8 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        PosU8 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::AddAssign for PosU8 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl std::ops::Sub for PosU8 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        PosU8 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::SubAssign for PosU8 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

// #[macro_export]
// macro_rules! pos {
//     ($x:expr, $y:expr, $z:expr ) => {
//         u83 {
//             x: $x,
//             y: $y,
//             z: $z,
//         }
//     };
// }
