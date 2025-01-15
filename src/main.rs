use std::path::PathBuf;

use clap::Parser;
use glam::{DMat3, DVec2, DVec3};
use opencascade::primitives::{BooleanShape, Edge, Face, IntoShape, Shape, Wire};
use ttf_parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(
        short,
        long,
        value_name = "FILE",
        default_value = "assets/FiraSans-Regular.ttf"
    )]
    font: PathBuf,

    #[arg(short, long, value_name = "FILE", default_value = "output")]
    output: PathBuf,
}

struct GlyphBuilder {
    p0: Option<DVec2>,
    pub contours: Vec<Contour>,
}

type Contour = Vec<Curve>;

#[derive(Debug, Clone, Copy)]
enum Curve {
    Line(DVec2, DVec2),
    Bezier2(DVec2, DVec2, DVec2),
    Bezier3(DVec2, DVec2, DVec2, DVec2),
}

impl Curve {
    pub fn first_point(&self) -> DVec2 {
        match self {
            Curve::Line(p, _) => *p,
            Curve::Bezier2(p, _, _) => *p,
            Curve::Bezier3(p, _, _, _) => *p,
        }
    }
}

impl GlyphBuilder {
    pub fn new() -> Self {
        Self {
            p0: None,
            contours: vec![],
        }
    }
}

impl ttf_parser::OutlineBuilder for GlyphBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.p0 = Some(DVec2::new(x as f64, y as f64));
        self.contours.push(vec![]);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let p0 = self.p0.unwrap();
        let p1 = DVec2::new(x as f64, y as f64);
        self.contours.last_mut().unwrap().push(Curve::Line(p0, p1));
        self.p0 = Some(p1);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let p0 = self.p0.unwrap();
        let p1 = DVec2::new(x1 as f64, y1 as f64);
        let p2 = DVec2::new(x as f64, y as f64);
        self.contours
            .last_mut()
            .unwrap()
            .push(Curve::Bezier2(p0, p1, p2));
        self.p0 = Some(p2);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let p0 = self.p0.unwrap();
        let p1 = DVec2::new(x1 as f64, y1 as f64);
        let p2 = DVec2::new(x2 as f64, y2 as f64);
        let p3 = DVec2::new(x as f64, y as f64);
        self.contours
            .last_mut()
            .unwrap()
            .push(Curve::Bezier3(p0, p1, p2, p3));
        self.p0 = Some(p3);
    }

    fn close(&mut self) {
        let p0 = self.p0.unwrap();
        let p1 = self.contours.last().unwrap().first().unwrap().first_point();
        if p0 != p1 {
            self.contours.last_mut().unwrap().push(Curve::Line(p0, p1));
        }
        self.p0 = None;
    }
}

const EXTRUSION_DEPTH: f64 = 10_000.0;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let data = std::fs::read(args.font).unwrap();
    let face = ttf_parser::Face::parse(&data, 0).unwrap();

    std::fs::create_dir_all(&args.output)?;

    let chars: String = ('A'..'Z').chain('0'..'9').collect();

    for char_1 in chars.chars() {
        let shape_1 = render_glyph_to_brep(&face, char_1, EXTRUSION_DEPTH, DMat3::IDENTITY);

        for char_2 in chars.chars() {
            let name = format!("{char_1}{char_2}");
            println!("Preparing shape for {name}...");
            let path = args.output.join(format!("{name}.step"));

            let shape_2 = render_glyph_to_brep(
                &face,
                char_2,
                10000.0,
                DMat3::from_rotation_y(std::f64::consts::FRAC_PI_2),
            );

            let shape = shape_1.intersect(&shape_2).into_shape();
            shape.write_step(path)?;
        }
    }

    Ok(())
}

fn render_glyph_to_brep(
    face: &ttf_parser::Face,
    code_point: char,
    thickness: f64,
    transform: DMat3,
) -> Shape {
    let mut builder = GlyphBuilder::new();
    let glyph_id = face.glyph_index(code_point).unwrap();
    let bbox = face.outline_glyph(glyph_id, &mut builder).unwrap();

    // need to horizontally center glyphs
    let h_center = DVec2::new((bbox.x_min + bbox.x_max) as f64 * 0.5, 0.0);

    let mut parts: Vec<Shape> = vec![];
    for contour in builder.contours {
        let mut edges: Vec<Edge> = vec![];
        for curve in contour {
            match curve {
                Curve::Line(p1, p2) => edges.push(Edge::segment(
                    transform * (p1 - h_center).extend(-0.5 * thickness),
                    transform * (p2 - h_center).extend(-0.5 * thickness),
                )),
                Curve::Bezier2(p1, p2, p3) => edges.push(Edge::bezier([
                    transform * (p1 - h_center).extend(-0.5 * thickness),
                    transform * (p2 - h_center).extend(-0.5 * thickness),
                    transform * (p3 - h_center).extend(-0.5 * thickness),
                ])),
                Curve::Bezier3(p1, p2, p3, p4) => edges.push(Edge::bezier([
                    transform * (p1 - h_center).extend(-0.5 * thickness),
                    transform * (p2 - h_center).extend(-0.5 * thickness),
                    transform * (p3 - h_center).extend(-0.5 * thickness),
                    transform * (p4 - h_center).extend(-0.5 * thickness),
                ])),
            }
        }
        let wire = Wire::from_edges(&edges);
        let face = Face::from_wire(&wire);
        let solid = face.extrude(transform * DVec3::new(0.0, 0.0, thickness));
        parts.push(solid.into_shape());
    }

    let shape = parts
        .into_iter()
        .reduce(|acc, x| symmetric_difference(&acc, &x).into_shape())
        .unwrap();

    shape
}

fn symmetric_difference(a: &Shape, b: &Shape) -> BooleanShape {
    a.union(b).subtract(&a.intersect(b))
}
