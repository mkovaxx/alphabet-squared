use std::path::PathBuf;

use clap::Parser;
use glam::DVec2;
use opencascade::primitives::{Edge, Face, IntoShape, Wire};
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

    char: char,
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
        let p1 = self
            .contours
            .first()
            .unwrap()
            .first()
            .unwrap()
            .first_point();
        if p0 != p1 {
            self.contours.last_mut().unwrap().push(Curve::Line(p0, p1));
        }
        self.p0 = None;
    }
}

fn main() {
    let args = Args::parse();

    let data = std::fs::read(args.font).unwrap();
    let face = ttf_parser::Face::parse(&data, 0).unwrap();

    let mut builder = GlyphBuilder::new();
    let glyph_id = face.glyph_index(args.char).unwrap();
    let bbox = face.outline_glyph(glyph_id, &mut builder).unwrap();

    let mut faces: Vec<Face> = vec![];
    for contour in builder.contours {
        println!("contour: {contour:?}");
        let mut edges: Vec<Edge> = vec![];
        for curve in contour {
            match curve {
                Curve::Line(p1, p2) => edges.push(Edge::segment(p1.extend(0.0), p2.extend(0.0))),
                Curve::Bezier2(p1, p2, p3) => edges.push(Edge::bezier([
                    p1.extend(0.0),
                    p2.extend(0.0),
                    p3.extend(0.0),
                ])),
                Curve::Bezier3(p1, p2, p3, p4) => edges.push(Edge::bezier([
                    p1.extend(0.0),
                    p2.extend(0.0),
                    p3.extend(0.0),
                    p4.extend(0.0),
                ])),
            }
            let wire = Wire::from_edges(&edges);
            let face = Face::from_wire(&wire);
            faces.push(face);
        }
    }

    let shape = faces
        .iter()
        .map(|x| x.into_shape())
        .reduce(|acc, x| acc.union(&x).into_shape())
        .unwrap();

    shape.write_step("out.step").unwrap();
}
