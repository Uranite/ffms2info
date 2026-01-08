use clap::Parser;
use std::path::PathBuf;

mod ffms;

#[derive(Parser)]
struct Cli {
    path: PathBuf,
}

fn main() {
    let args = Cli::parse();
    match ffms::VidIdx::new(&args.path) {
        Ok(idx) => match ffms::get_vidinf(&idx) {
            Ok(inf) => println!("{:#?}", inf),
            Err(e) => eprintln!("Error getting video info: {}", e),
        },
        Err(e) => eprintln!("Error indexing file: {}", e),
    }
}
