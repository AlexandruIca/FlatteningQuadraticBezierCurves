mod geometry;
mod rasterizer;

use geometry::{Point, QuadraticBezier};
use rasterizer::{f32_to_u8, Color, Rasterizer, SubdivisionMethod};
use ttf_parser as ttf;

#[derive(Debug, Clone, Copy)]
struct RendererColors {
    fg_color: Color,
    bg_color: Color,
}

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

const FONT_SIZE: f64 = 128.0;

///
/// Only used to compute the bounding box of a glyph
///
struct BboxOutlineBuilder {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

fn fmin3(a: f32, b: f32, c: f32) -> f32 {
    f32::min(a, f32::min(b, c))
}

fn fmax3(a: f32, b: f32, c: f32) -> f32 {
    f32::max(a, f32::max(b, c))
}

impl ttf::OutlineBuilder for BboxOutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.min_x = f32::min(self.min_x, x);
        self.min_y = f32::min(self.min_y, y);
        self.max_x = f32::max(self.max_x, x);
        self.max_y = f32::max(self.max_y, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.min_x = f32::min(self.min_x, x);
        self.min_y = f32::min(self.min_y, y);
        self.max_x = f32::max(self.max_x, x);
        self.max_y = f32::max(self.max_y, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.min_x = fmin3(self.min_x, x1, x);
        self.min_y = fmin3(self.min_y, y1, y);
        self.max_x = fmax3(self.max_x, x1, x);
        self.max_y = fmax3(self.max_y, y1, y);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        println!(
            "Cubic to: (x1={}, y1={}), (x2={}, y2={}), (x={}, y={})",
            x1, y1, x2, y2, x, y
        );
    }

    fn close(&mut self) {}
}

struct OutlineBuilder {
    rasterizer: Rasterizer,
    prev_point: Point,
    min_x: f32,
    min_y: f32,
    starting_point: Point,
    subdivision_method: SubdivisionMethod,
}

impl Default for OutlineBuilder {
    fn default() -> Self {
        Self {
            rasterizer: Default::default(),
            prev_point: Point { x: 0.0, y: 0.0 },
            min_x: 0.0,
            min_y: 0.0,
            starting_point: Point { x: 0.0, y: 0.0 },
            subdivision_method: SubdivisionMethod::ParabolaApprox,
        }
    }
}

impl OutlineBuilder {
    fn new(tolerance: f32, bbox: &BboxOutlineBuilder, method: SubdivisionMethod) -> Self {
        let width = (bbox.max_x - bbox.min_x).ceil() as usize + 1;
        let height = (bbox.max_y - bbox.min_y).ceil() as usize + 1;

        Self {
            rasterizer: Rasterizer {
                width,
                height,
                tolerance,
                accumulation_buffer: vec![0.0_f32; width * height],
            },
            prev_point: Point { x: 0.0, y: 0.0 },
            min_x: bbox.min_x,
            min_y: bbox.min_y,
            starting_point: Point {
                x: f32::MIN,
                y: f32::MIN,
            },
            subdivision_method: method,
        }
    }
}

impl ttf::OutlineBuilder for OutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        let new_point = Point {
            x: x - self.min_x,
            y: self.rasterizer.height as f32 - (y - self.min_y),
        };
        self.prev_point = new_point;
        self.starting_point = new_point;

        println!("Move to: (x={}, y={})", new_point.x, new_point.y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let new_point = Point {
            x: x - self.min_x,
            y: self.rasterizer.height as f32 - (y - self.min_y),
        };
        self.rasterizer.draw_line(self.prev_point, new_point);
        self.prev_point = new_point;

        println!("Line to: (x={}, y={})", new_point.x, new_point.y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let p1 = Point {
            x: x1 - self.min_x,
            y: self.rasterizer.height as f32 - (y1 - self.min_y),
        };
        let p = Point {
            x: x - self.min_x,
            y: self.rasterizer.height as f32 - (y - self.min_y),
        };
        let q = QuadraticBezier::new(self.prev_point, p1, p);
        self.rasterizer.draw_quadratic(q, self.subdivision_method);
        self.prev_point = p;

        println!(
            "Quad to: (x1={}, y1={}), (x={}, y={})",
            p1.x, p1.y, p.x, p.y
        );
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        println!(
            "Cubic to: (x1={}, y1={}), (x2={}, y2={}), (x={}, y={})",
            x1, y1, x2, y2, x, y
        );
    }

    fn close(&mut self) {
        self.rasterizer
            .draw_line(self.prev_point, self.starting_point);
        self.prev_point = self.starting_point;

        println!("CLOSE");
    }
}

fn glyph_test(font_path: &str, glyph_index: u16, tolerance: f32, colors: RendererColors) {
    let font_data = std::fs::read(font_path).unwrap();
    let face = ttf::Face::from_slice(&font_data, 0).unwrap();

    let units_per_em = face.units_per_em();
    let scale = FONT_SIZE / units_per_em as f64;

    println!("--------------------------------------");
    println!(
        "Number of glyphs: {}, units per em: {}, scale: {}",
        face.number_of_glyphs(),
        units_per_em,
        scale,
    );

    let glyph_id = ttf::GlyphId(glyph_index);
    let glyph_to_path = |face: &ttf::Face, glyph_id: ttf::GlyphId, method: SubdivisionMethod| {
        let mut bbox_builder = BboxOutlineBuilder {
            min_x: 0.0,
            max_x: 0.0,
            min_y: 0.0,
            max_y: 0.0,
        };
        let _ = match face.outline_glyph(glyph_id, &mut bbox_builder) {
            Some(v) => v,
            None => return,
        };
        let mut builder = OutlineBuilder::new(tolerance, &bbox_builder, method);
        let bbox = match face.outline_glyph(glyph_id, &mut builder) {
            Some(v) => v,
            None => return,
        };

        println!("BBOX: {:?}", bbox);

        let subdivision_str = match method {
            SubdivisionMethod::DeCasteljau => "recursive",
            SubdivisionMethod::ParabolaApprox => "smart",
        };

        let output = format!(
            "glyph_{}_{}_subdivision_test.png",
            glyph_index, subdivision_str,
        );
        render_to(&builder.rasterizer, output.as_str(), colors);
    };

    glyph_to_path(&face, glyph_id, SubdivisionMethod::ParabolaApprox);
    glyph_to_path(&face, glyph_id, SubdivisionMethod::DeCasteljau);
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
        );
    }
}
