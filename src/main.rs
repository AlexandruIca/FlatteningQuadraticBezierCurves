mod geometry;
mod rasterizer;

use geometry::{Point, QuadraticBezier};
use rasterizer::{f32_to_u8, Color, Rasterizer, SubdivisionMethod};

fn render_to(rasterizer: &Rasterizer, name: &str) {
    let buffer = rasterizer
        .render(Color::black(), Color::white())
        .iter()
        .map(|value| f32_to_u8(*value))
        .collect::<Vec<u8>>();

    image::save_buffer(
        name,
        buffer.as_slice(),
        rasterizer.width as u32,
        rasterizer.height as u32,
        image::ColorType::Rgba8,
    )
    .unwrap();
}

fn simple_output_comparison_test() {
    let points = [
        Point { x: 100.0, y: 400.0 },
        Point { x: 300.0, y: 400.0 },
        Point { x: 500.0, y: 100.0 },
    ];
    let quadratic = QuadraticBezier::new(points[0], points[1], points[2]);

    let lines = quadratic
        .smart_subdivide(1.0)
        .iter()
        .map(|t| quadratic.eval(*t))
        .collect::<Vec<Point>>();

    let lines_dc = quadratic
        .recursive_subdivide(1.0)
        .iter()
        .map(|t| quadratic.eval(*t))
        .collect::<Vec<Point>>();

    println!(
        "[Smart subdivision]: {} line segments:\n{:?}",
        lines.len(),
        lines
    );
    println!(
        "[De Casteljau subdivision]: {} line segments:\n{:?}",
        lines_dc.len(),
        lines_dc
    );
}

fn simple_quadratic_curve_image_test() {
    let tolerance = 0.25_f32;
    let points = [
        Point { x: 100.0, y: 400.0 },
        Point { x: 300.0, y: 400.0 },
        Point { x: 500.0, y: 100.0 },
    ];
    let quadratic = QuadraticBezier::new(points[0], points[1], points[2]);

    {
        let mut rasterizer = Rasterizer {
            tolerance,
            ..Default::default()
        };

        rasterizer.draw_quadratic(quadratic, SubdivisionMethod::ParabolaApprox);

        render_to(&rasterizer, "smart_subdivision_simple_test.png");
    }
    {
        let mut rasterizer = Rasterizer {
            tolerance,
            ..Default::default()
        };

        rasterizer.draw_quadratic(quadratic, SubdivisionMethod::DeCasteljau);

        render_to(&rasterizer, "DeCasteljau_subdivision_simple_test.png");
    }
}

fn main() {
    simple_output_comparison_test();
    simple_quadratic_curve_image_test();
}
