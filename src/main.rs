use std::{ffi::OsString, fs::File, io::BufReader};

use crate::{event::{Event, Key}, schema::HAR};

use clap::Parser;

pub mod schema;
pub mod termio;
pub mod event;
pub mod epoll;
pub mod borrowed_fd;
pub mod color;
pub mod char_width;
pub mod widgets;

#[derive(Parser)]
struct Args {
    path: Option<OsString>,
}

fn main() -> Result<(), std::io::Error> {
//  println!("{har:#?}");
    let args = Args::parse();

    let har: HAR = if let Some(path) = args.path {
        let file = File::open(path)?;
        serde_json::from_reader(BufReader::new(file))?
    } else {
        serde_json::from_reader(std::io::stdin())?
    };

    let mut app = termio::TermIO::from_stdio()?;
    let mut size = app.window_size()?;

    while let Some(event) = app.wait()? {

        //println!("{event}");

        match event {
            Event::WindowSize { rows, columns } => {
                if rows != size.rows || columns != size.columns {
                    // full redraw

                    size.rows = rows;
                    size.columns = columns;
                }
            }
            Event::KeyPress { key: Key::Char('q'), alt: false, ctrl: false, shift: false } => {
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
