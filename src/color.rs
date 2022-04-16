///
/// Colors in range 0.0-1.0
///
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn white() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }

    pub fn black() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }

    pub fn yellow_green() -> Self {
        Self {
            r: 154.0 / 256.0,
            g: 205.0 / 256.0,
            b: 50.0 / 256.0,
            a: 1.0,
        }
    }

    pub fn steel_blue() -> Self {
        Self {
            r: 70.0 / 256.0,
            g: 130.0 / 256.0,
            b: 180.0 / 256.0,
            a: 1.0,
        }
    }
}
