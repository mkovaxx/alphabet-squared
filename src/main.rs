use std::path::PathBuf;

use clap::Parser;
use freetype as ft;
use glam::DVec3;
use opencascade::primitives::{Edge, Face, IntoShape, Wire};

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

fn main() {
    let args = Args::parse();

    let font = args.font;
    let character = args.char as usize;
    let library = ft::Library::init().unwrap();
    let face = library.new_face(font, 0).unwrap();

    face.set_char_size(40 * 64, 0, 50, 0).unwrap();
    face.load_char(character, ft::face::LoadFlag::NO_SCALE)
        .unwrap();

    let glyph = face.glyph();
    let metrics = glyph.metrics();
    let xmin = metrics.horiBearingX - 5;
    let width = metrics.width + 10;
    let ymin = -metrics.horiBearingY - 5;
    let height = metrics.height + 10;
    let outline = glyph.outline().unwrap();

    let mut faces = vec![];

    for contour in outline.contours_iter() {
        let mut p0 = *contour.start();
        let mut edges = vec![];
        for curve in contour {
            match curve {
                ft::outline::Curve::Line(p1) => {
                    edges.push((p0, p1));
                    p0 = p1;
                }
                ft::outline::Curve::Bezier2(p1, p2) => {
                    // TODO: make quadratic Bezier edge
                    edges.push((p0, p1));
                    edges.push((p1, p2));
                    p0 = p2;
                }
                ft::outline::Curve::Bezier3(p1, p2, p3) => {
                    // TODO: make cubic Bezier edge
                    edges.push((p0, p1));
                    edges.push((p1, p2));
                    edges.push((p2, p3));
                    p0 = p3;
                }
            }
        }

        let edges: Vec<Edge> = edges
            .iter()
            .filter(|(p1, p2)| p1 != p2)
            .map(|(p1, p2)| {
                Edge::segment(
                    DVec3::new(p1.x as f64, p1.y as f64, 0.0),
                    DVec3::new(p2.x as f64, p2.y as f64, 0.0),
                )
            })
            .collect();
        let wire = Wire::from_edges(&edges);
        let face = Face::from_wire(&wire);

        faces.push(face);
    }

    let shape = faces
        .iter()
        .map(|x| x.into_shape())
        .reduce(|acc, x| acc.union(&x).into_shape())
        .unwrap();

    shape.write_step("out.step").unwrap();
}
