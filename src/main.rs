use std::path::PathBuf;

use clap::Parser;
use glam::DVec3;
use opencascade::{
    primitives::{IntoShape, Solid},
    workplane::Workplane,
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

    let chars: String = ('A'..'Z').chain('0'..'9').collect();

    for char_1 in chars.chars() {
        let shape_1 = make_glyph_shape("Sans", char_1, 0.0);
        for char_2 in chars.chars() {
            let name = format!("{char_1}{char_2}");
            println!("Preparing shape for {name}...");
            let path = args.output.join(format!("{name}.stl"));
            let shape_2 = make_glyph_shape("Sans", char_2, 90.0);
            let shape = shape_1.union(&shape_2).into_shape();
            shape.write_stl(path)?;
        }
    }

    Ok(())
}

fn make_glyph_shape(font: &str, c: char, angle_deg: f64) -> Solid {
    let sketch = Workplane::xz().rect(1.0, 1.0);
    let shape = sketch.to_face().extrude(DVec3::new(0.0, 1.0, 0.0));
    shape
}
