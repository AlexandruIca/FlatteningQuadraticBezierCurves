pub fn clamp<T: std::cmp::PartialOrd>(value: T, min: T, max: T) -> T {
    if value > max {
        max
    } else if value < min {
        min
    } else {
        value
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn lerp(&self, p2: Point, t: f32) -> Point {
        Point {
            x: self.x + (p2.x - self.x) * t,
            y: self.y + (p2.y - self.y) * t,
        }
    }

    pub fn distance(&self, p2: Point) -> f32 {
        f32::hypot(p2.x - self.x, p2.y - self.y)
    }
}

pub fn approximate_integral(x: f32) -> f32 {
    const D: f32 = 0.67;
    x / (1.0 - D + f32::powf(f32::powf(D, 4.0) + 0.25 * x * x, 0.25))
}

pub fn approximate_inverse_integral(x: f32) -> f32 {
    const B: f32 = 0.39;

    x * (1.0 - B + f32::sqrt(B * B - 0.25 * x * x))
}

pub struct ParabolaParams {
    x0: f32,
    x2: f32,
    scale: f32,
    cross: f32,
}

pub struct QuadraticBezier {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

impl QuadraticBezier {
    pub fn new(p0: Point, p1: Point, p2: Point) -> Self {
        Self {
            x0: p0.x,
            y0: p0.y,
            x1: p1.x,
            y1: p1.y,
            x2: p2.x,
            y2: p2.y,
        }
    }

    pub fn eval(&self, t: f32) -> Point {
        let one_minus_t = 1.0 - t;
        let x =
            self.x0 * one_minus_t * one_minus_t + 2.0 * self.x1 * t * one_minus_t + self.x2 * t * t;
        let y =
            self.y0 * one_minus_t * one_minus_t + 2.0 * self.y1 * t * one_minus_t + self.y2 * t * t;

        Point { x, y }
    }

    pub fn subsegment(&self, t0: f32, t1: f32) -> Self {
        let (p0, p2) = (self.eval(t0), self.eval(t1));
        let dt = t1 - t0;

        let p1x = p0.x + (self.x1 - self.x0 + t0 * (self.x2 - 2.0 * self.x1 + self.x0)) * dt;
        let p1y = p0.y + (self.y1 - self.y0 + t0 * (self.y2 - 2.0 * self.y1 + self.y0)) * dt;

        Self {
            x0: p0.x,
            y0: p0.y,
            x1: p1x,
            y1: p1y,
            x2: p2.x,
            y2: p2.y,
        }
    }

    pub fn error(&self) -> f32 {
        let x1 = self.x1 - self.x0;
        let y1 = self.y1 - self.y0;
        let x2 = self.x2 - self.x0;
        let y2 = self.y2 - self.y0;
        let t = (x1 * x2 + y1 * y2) / (x2 * x2 + y2 * y2);
        let u = clamp(t, 0.0, 1.0);
        let p = Point {
            x: self.x0,
            y: self.y0,
        }
        .lerp(
            Point {
                x: self.x2,
                y: self.y2,
            },
            u,
        );

        0.5 * p.distance(Point {
            x: self.x1,
            y: self.y1,
        })
    }

    pub fn recursive_subdivide_impl(&self, err: f32, t0: f32, t1: f32, result: &mut Vec<f32>) {
        let q = self.subsegment(t0, t1);

        if q.error() <= err {
            result.push(t1);
        } else {
            let t_mid = (t0 + t1) * 0.5;
            self.recursive_subdivide_impl(err, t0, t_mid, result);
            self.recursive_subdivide_impl(err, t_mid, t1, result);
        }
    }

    pub fn recursive_subdivide(&self, err: f32) -> Vec<f32> {
        let mut result = vec![0_f32];
        self.recursive_subdivide_impl(err, 0.0, 1.0, &mut result);

        result
    }

    pub fn map_to_basic(&self) -> ParabolaParams {
        let ddx = 2.0 * self.x1 - self.x0 - self.x2;
        let ddy = 2.0 * self.y1 - self.y0 - self.y2;
        let u0 = (self.x1 - self.x0) * ddx + (self.y1 - self.y0) * ddy;
        let u2 = (self.x2 - self.x1) * ddx + (self.y2 - self.y1) * ddy;
        let cross = (self.x2 - self.x0) * ddy - (self.y2 - self.y0) * ddx;
        let x0 = u0 / cross;
        let x2 = u2 / cross;
        let scale = f32::abs(cross) / (f32::hypot(ddx, ddy) * f32::abs(x2 - x0));

        ParabolaParams {
            x0,
            x2,
            scale,
            cross,
        }
    }

    pub fn smart_subdivide(&self, err: f32) -> Vec<f32> {
        let params = self.map_to_basic();
        let a0 = approximate_integral(params.x0);
        let a2 = approximate_integral(params.x2);
        let count = 0.5 * f32::abs(a2 - a0) * f32::sqrt(params.scale / err);
        let n = f32::ceil(count);
        let u0 = approximate_inverse_integral(a0);
        let u2 = approximate_inverse_integral(a2);
        let mut result = vec![0_f32];

        for i in 1..(n as i32) {
            let u = approximate_inverse_integral(a0 + ((a2 - a0) * (i as f32)) / n);
            let t = (u - u0) / (u2 - u0);
            result.push(t);
        }

        result.push(1.0);
        result
    }
}
