use std::path::PathBuf;

use clap::Parser;
use glam::DVec3;
use opencascade::{
    primitives::{IntoShape, Shape},
    text::{Font, FontAspect},
};

#[derive(Parser, Debug)]
struct Args {
    font: String,

    #[arg(short, long, value_name = "FILE", default_value = "output")]
    output: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    std::fs::create_dir_all(&args.output)?;

    let mut font = Font::from_name("Sans", FontAspect::Regular, 10.0);

    let chars: String = ('A'..'Z').chain('0'..'9').collect();

    for char_1 in chars.chars() {
        let shape_1 = make_glyph_shape(&mut font, char_1);
        for char_2 in chars.chars() {
            let name = format!("{char_1}{char_2}");
            println!("Preparing shape for {name}...");
            let path = args.output.join(format!("{name}.stl"));
            let shape_2 = make_glyph_shape(&mut font, char_2);
            let shape = shape_1.union(&shape_2).into_shape();
            shape.write_stl(path)?;
        }
    }

    Ok(())
}

fn make_glyph_shape(font: &mut Font, c: char) -> Shape {
    let face = font.render_glyph(c);
    let shape = face.extrude(DVec3::new(0.0, 1.0, 0.0));
    shape
}
