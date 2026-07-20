use std::{ffi::OsString, fs::File, io::BufReader};

use thar::schema::HAR;

use clap::Parser;

#[derive(Parser)]
struct Args {
    path: Option<OsString>,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let har: HAR = if let Some(path) = args.path {
        let file = File::open(path)?;
        serde_json::from_reader(BufReader::new(file))?
    } else {
        serde_json::from_reader(std::io::stdin())?
    };

    println!("{har:#?}");

    Ok(())
}
