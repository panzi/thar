use std::{fs::File, io::BufReader};

use crate::schema::HAR;

pub mod schema;

fn main() -> Result<(), std::io::Error> {
    let file = File::open(std::env::args_os().nth(1).unwrap())?;
    let har: HAR = serde_json::from_reader(BufReader::new(file))?;

    println!("{har:#?}");

    Ok(())
}
