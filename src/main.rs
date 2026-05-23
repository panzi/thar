//use std::{fs::File, io::BufReader};
//
//use crate::schema::HAR;

use crate::escape::{ESCAPE, InputSequence};

pub mod schema;
pub mod termio;
pub mod escape;
pub mod epoll;

fn main() -> Result<(), std::io::Error> {
//    let file = File::open(std::env::args_os().nth(1).unwrap())?;
//    let har: HAR = serde_json::from_reader(BufReader::new(file))?;
//
//    println!("{har:#?}");

    let mut app = termio::TermIO::from_stdio()?;

    while let Some(seq) = app.wait()? {
        println!("{seq:?}");

        if seq == InputSequence::Char(ESCAPE.into()) {
            break;
        }
    }

    Ok(())
}
