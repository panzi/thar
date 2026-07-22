use std::{ffi::OsString, fs::File, io::{Read, Write}};

use clap::Parser;
use thar::{colorize::colorize_sgml, rich_text::RichText};

#[derive(Parser)]
struct Args {
    path: OsString,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let mut buf = String::new();

    {
        let mut file = File::open(args.path)?;
        file.read_to_string(&mut buf)?;
    }

    let mut rich_text = RichText::new();

    colorize_sgml(&buf, &mut rich_text);

    let mut stdout = std::io::stdout();
    rich_text.print(&mut stdout)?;
    stdout.flush()?;

    Ok(())
}
