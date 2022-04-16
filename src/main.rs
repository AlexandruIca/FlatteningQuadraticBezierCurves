mod color;
mod font;
mod geometry;
mod rasterizer;

use color::Color;
use font::{glyph_test, RendererColors};
use geometry::{Point, QuadraticBezier};
use rasterizer::{f32_to_u8, Rasterizer, SubdivisionMethod};

fn render_to(rasterizer: &Rasterizer, name: &str, colors: RendererColors) {
    let buffer = rasterizer
        .render(colors.fg_color, colors.bg_color)
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

        render_to(
            &rasterizer,
            "smart_subdivision_simple_test.png",
            RendererColors {
                fg_color: Color::black(),
                bg_color: Color::white(),
            },
        );
    }
    {
        let mut rasterizer = Rasterizer {
            tolerance,
            ..Default::default()
        };

        rasterizer.draw_quadratic(quadratic, SubdivisionMethod::DeCasteljau);

        render_to(
            &rasterizer,
            "DeCasteljau_subdivision_simple_test.png",
            RendererColors {
                fg_color: Color::black(),
                bg_color: Color::white(),
            },
        );
    }
}

struct GlyphTestDesc<'a> {
    font_path: &'a str,
    glyph_index: u16,
    colors: RendererColors,
    tolerance: f32,
}

fn main() {
    let glyph_test_data = [
        GlyphTestDesc {
            font_path: "media/Roboto-MediumItalic.ttf",
            glyph_index: 36, // '@'
            colors: RendererColors {
                fg_color: Color::black(),
                bg_color: Color::white(),
            },
            tolerance: 0.25,
        },
        GlyphTestDesc {
            font_path: "media/Jfwildwood-ldYZ.ttf",
            glyph_index: 42, // 'F'
            colors: RendererColors {
                fg_color: Color::white(),
                bg_color: Color::yellow_green(),
            },
            tolerance: 1.5,
        },
        GlyphTestDesc {
            font_path: "media/Jfwildwood-ldYZ.ttf",
            glyph_index: 59, // 'W'
            colors: RendererColors {
                fg_color: Color::white(),
                bg_color: Color::steel_blue(),
            },
            tolerance: 2.5,
        },
    ];

    simple_output_comparison_test();
    simple_quadratic_curve_image_test();

    for test in glyph_test_data {
        glyph_test(
            test.font_path,
            test.glyph_index,
            test.tolerance,
            test.colors,
            render_to,
        );
    }
}
