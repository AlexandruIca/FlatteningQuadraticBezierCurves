use crate::{geometry::QuadraticBezier, Point};

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

// https://stackoverflow.com/a/56842762/8622014
pub fn f32_to_u8(value: f32) -> u8 {
    let value = value as f64;
    const FACTOR: f64 = (u8::MAX as f64) - f64::EPSILON * 128_f64;

    (value * FACTOR) as u8
}

#[derive(Debug, Clone, Copy)]
pub enum SubdivisionMethod {
    DeCasteljau,
    ParabolaApprox,
}

pub struct Rasterizer {
    pub width: usize,
    pub height: usize,
    pub accumulation_buffer: Vec<f32>,
    pub tolerance: f32, // used when drawing quadratic Bézier curves
}

impl Default for Rasterizer {
    fn default() -> Self {
        let (w, h) = (700_usize, 500_usize);

        Self {
            width: w,
            height: h,
            accumulation_buffer: vec![0.0_f32; w * h],
            tolerance: 1.0,
        }
    }
}

impl Rasterizer {
    // Thanks to: https://github.com/raphlinus/font-rs/blob/master/src/raster.rs
    pub fn draw_line(&mut self, p0: Point, p1: Point) {
        if (p0.y - p1.y).abs() <= f32::EPSILON {
            return;
        }

        let (dir, p0, p1) = if p0.y < p1.y {
            (1.0, p0, p1)
        } else {
            (-1.0, p1, p0)
        };

        let dxdy = (p1.x - p0.x) / (p1.y - p0.y);
        let mut x = p0.x;
        let y0 = p0.y as usize;

        for y in y0..usize::min(self.height, p1.y.ceil() as usize) {
            let linestart = y * self.width;
            let dy = ((y + 1) as f32).min(p1.y) - (y as f32).max(p0.y);
            let xnext = x + dxdy * dy;
            let d = dy * dir;
            let (x0, x1) = if x < xnext { (x, xnext) } else { (xnext, x) };
            let x0floor = x0.floor();
            let x0i = x0floor as i32;
            let x1ceil = x1.ceil();
            let x1i = x1ceil as i32;

            if x1i <= x0i + 1 {
                let xmf = 0.5 * (x + xnext) - x0floor;
                let linestart_x0i = linestart as isize + x0i as isize;

                if linestart_x0i < 0 {
                    continue;
                }

                self.accumulation_buffer[linestart_x0i as usize] += d - d * xmf;
                self.accumulation_buffer[linestart_x0i as usize + 1] += d * xmf;
            } else {
                let s = (x1 - x0).recip();
                let x0f = x0 - x0floor;
                let a0 = 0.5 * s * (1.0 - x0f) * (1.0 - x0f);
                let x1f = x1 - x1ceil + 1.0;
                let am = 0.5 * s * x1f * x1f;
                let linestart_x0i = linestart as isize + x0i as isize;

                if linestart_x0i < 0 {
                    continue;
                }

                self.accumulation_buffer[linestart_x0i as usize] += d * a0;

                if x1i == x0i + 2 {
                    self.accumulation_buffer[linestart_x0i as usize + 1] += d * (1.0 - a0 - am);
                } else {
                    let a1 = s * (1.5 - x0f);
                    self.accumulation_buffer[linestart_x0i as usize + 1] += d * (a1 - a0);

                    for xi in x0i + 2..x1i - 1 {
                        self.accumulation_buffer[linestart + xi as usize] += d * s;
                    }

                    let a2 = a1 + (x1i - x0i - 3) as f32 * s;
                    self.accumulation_buffer[linestart + (x1i - 1) as usize] += d * (1.0 - a2 - am);
                }

                self.accumulation_buffer[linestart + x1i as usize] += d * am;
            }

            x = xnext;
        }
    }

    pub fn draw_quadratic(&mut self, q: QuadraticBezier, method: SubdivisionMethod) {
        match method {
            SubdivisionMethod::DeCasteljau => {
                let points = q
                    .recursive_subdivide(self.tolerance)
                    .iter()
                    .map(|t| q.eval(*t))
                    .collect::<Vec<Point>>();

                points.windows(2).for_each(|p| {
                    self.draw_line(p[0], p[1]);
                });
            }
            SubdivisionMethod::ParabolaApprox => {
                let points = q
                    .smart_subdivide(self.tolerance)
                    .iter()
                    .map(|t| q.eval(*t))
                    .collect::<Vec<Point>>();

                points.windows(2).for_each(|p| {
                    self.draw_line(p[0], p[1]);
                });
            }
        }
    }

    ///
    /// Outputs an RGBA-encoded buffer with values between 0.0 and 1.0 for each component.
    ///
    pub fn render(&self, fg_color: Color, bg_color: Color) -> Vec<f32> {
        const NUM_CHANNELS: usize = 4;
        let mut result = vec![0.0_f32; self.width * self.height * NUM_CHANNELS];

        result
            .as_mut_slice()
            .chunks_mut(NUM_CHANNELS)
            .for_each(|chunk| {
                chunk[0] = bg_color.r;
                chunk[1] = bg_color.g;
                chunk[2] = bg_color.b;
                chunk[3] = bg_color.a;
            });

        for y in 0..self.height {
            let mut acc = 0.0_f32;

            for x in 0..self.width {
                acc += self.accumulation_buffer[y * self.width + x];

                let dest = bg_color;
                let src = Color {
                    r: fg_color.r,
                    g: fg_color.g,
                    b: fg_color.b,
                    a: acc.abs(),
                };
                let resulting_color = Color {
                    r: src.r * src.a + dest.r * (1.0_f32 - src.a),
                    g: src.g * src.a + dest.g * (1.0_f32 - src.a),
                    b: src.b * src.a + dest.b * (1.0_f32 - src.a),
                    a: dest.a,
                };
                let buffer_index: usize = y * self.width * NUM_CHANNELS + x * NUM_CHANNELS;

                result[buffer_index] = resulting_color.r;
                result[buffer_index + 1] = resulting_color.g;
                result[buffer_index + 2] = resulting_color.b;
                result[buffer_index + 3] = resulting_color.a;
            }
        }

        result
    }
}
