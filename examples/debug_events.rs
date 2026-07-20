use thar::{event::{Event, Key}, termio::TermIO};

fn main() -> std::io::Result<()> {
    let mut termio = TermIO::from_tty()?;

    termio.move_cursor(0, 0)?;
    termio.enable_mouse()?;
    termio.flush()?;

    while let Some(event) = termio.wait()? {
        println!("{event}");
        match event {
            Event::KeyPress { key: Key::Char('q'), alt: false, ctrl: false, shift: false } => {
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
