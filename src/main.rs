//use std::{fs::File, io::BufReader};
//
//use crate::schema::HAR;

use crate::event::{Event, Key};

pub mod schema;
pub mod termio;
pub mod event;
pub mod epoll;
pub mod borrowedfd;
pub mod color;

fn main() -> Result<(), std::io::Error> {
//    let file = File::open(std::env::args_os().nth(1).unwrap())?;
//    let har: HAR = serde_json::from_reader(BufReader::new(file))?;
//
//    println!("{har:#?}");

    let mut app = termio::TermIO::from_stdio()?;

    while let Some(event) = app.wait()? {
        println!("{event}");

        if matches!(event, Event::KeyPress { key: Key::Escape, alt: false, ctrl: false, shift: false }) {
            break;
        }
    }

    Ok(())
}
