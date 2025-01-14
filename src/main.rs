use std::path::PathBuf;

use clap::Parser;
use freetype as ft;
use glam::DVec3;
use opencascade::primitives::{Edge, IntoShape};

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

    let mut edges: Vec<Edge> = vec![];

    println!("<?xml version=\"1.0\" standalone=\"no\"?>");
    println!("<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1//EN\"");
    println!("\"http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd\">");
    println!(
        "<svg viewBox=\"{} {} {} {}\" xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\">",
        xmin, ymin, width, height
    );

    for contour in outline.contours_iter() {
        let start = contour.start();
        println!(
            "<path fill=\"none\" stroke=\"black\" stroke-width=\"1\" d=\"M {} {}",
            start.x, -start.y
        );
        let mut p_old = DVec3::new(start.x as f64, start.y as f64, 0.0);
        for curve in contour {
            draw_curve(curve);
            let (new_edges, p_new) = curve_to_edge(p_old, curve);
            edges.extend(new_edges);
            p_old = p_new;
        }
        println!("Z \" />");
    }
    println!("</svg>");

    let shape = edges
        .into_iter()
        .map(|edge| edge.into_shape())
        .reduce(|acc, x| acc.union(&x).into_shape())
        .unwrap();

    shape.write_step("out.step").unwrap();
}

fn curve_to_edge(p0: DVec3, curve: ft::outline::Curve) -> (Vec<Edge>, DVec3) {
    match curve {
        ft::outline::Curve::Line(p1) => {
            let p1 = DVec3::new(p1.x as f64, p1.y as f64, 0.0);
            let edges = vec![Edge::segment(p0, p1)];
            (edges, p1)
        }
        ft::outline::Curve::Bezier2(p1, p2) => {
            // TODO: make quadratic Bezier edge
            let p1 = DVec3::new(p1.x as f64, p1.y as f64, 0.0);
            let p2 = DVec3::new(p2.x as f64, p2.y as f64, 0.0);
            let edges = vec![Edge::segment(p0, p1), Edge::segment(p1, p2)];
            (edges, p2)
        }
        ft::outline::Curve::Bezier3(p1, p2, p3) => {
            // TODO: make cubic Bezier edge
            let p1 = DVec3::new(p1.x as f64, p1.y as f64, 0.0);
            let p2 = DVec3::new(p2.x as f64, p2.y as f64, 0.0);
            let p3 = DVec3::new(p3.x as f64, p3.y as f64, 0.0);
            let edges = vec![
                Edge::segment(p0, p1),
                Edge::segment(p1, p2),
                Edge::segment(p2, p3),
            ];
            (edges, p3)
        }
    }
}

fn draw_curve(curve: ft::outline::Curve) {
    match curve {
        ft::outline::Curve::Line(pt) => println!("L {} {}", pt.x, -pt.y),
        ft::outline::Curve::Bezier2(pt1, pt2) => {
            println!("Q {} {} {} {}", pt1.x, -pt1.y, pt2.x, -pt2.y)
        }
        ft::outline::Curve::Bezier3(pt1, pt2, pt3) => println!(
            "C {} {} {} {} {} {}",
            pt1.x, -pt1.y, pt2.x, -pt2.y, pt3.x, -pt3.y
        ),
    }
}
