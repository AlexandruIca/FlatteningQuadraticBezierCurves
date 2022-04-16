mod geometry;

use geometry::{Point, QuadraticBezier};

fn main() {
    let points = [
        Point { x: 200.0, y: 400.0 },
        Point { x: 400.0, y: 400.0 },
        Point { x: 600.0, y: 100.0 },
    ];

    let quadratic = QuadraticBezier::new(points[0], points[1], points[2]);
    let lines = quadratic
        .smart_subdivide(1.0)
        .iter()
        .map(|t| quadratic.eval(*t))
        .collect::<Vec<Point>>();

    println!("{} line segments: {:?}", lines.len(), lines);
}
