use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    font: String,

    #[arg(short, long, value_name = "FILE", default_value = "output")]
    output: PathBuf,
}

fn main() {
    let args = Args::parse();

    let chars: String = ('A'..'Z').chain('0'..'9').collect();

    for char_1 in chars.chars() {
        for char_2 in chars.chars() {
            let name = format!("{char_1}{char_2}");
            println!("Preparing shape for {name}...");
        }
    }
}
