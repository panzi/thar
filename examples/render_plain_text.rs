use thar::{event::{Event, Key}, plain_text::PlainText, termio};

fn main() -> std::io::Result<()> {
    let mut args = std::env::args();

    let plain_text = if args.len() > 1 {
        args.next();

        let mut buf = String::new();
        for arg in args {
            if !buf.is_empty() {
                buf.push('\n');
            }
            buf.push_str(&arg);
        }

        buf
    } else {
        "\
Hello World!
Control characters:
\x00 \x01 \x02 \x03 \x04 \x05 \x06 \x07 \x08 \x09   \x0B \x0C \x0D \x0E \x0F
\x10 \x11 \x12 \x13 \x14 \x15 \x16 \x17 \x18 \x19 \x1A \x1B \x1C \x1D \x1E \x1F
\x7F
This is a long line demonstrating how things are truncated.
A second long line to verify this works for all lines that is even longer.
A last line.".to_string()
    };

    let mut plain_text = PlainText::parse(&plain_text);

    let mut termio = termio::TermIO::from_stdio()?;

    let mut column = 0;
    let mut row = 0;

    {
        let mut plain_text = plain_text.clone();
        plain_text.append(&format!("\nrow: {row}, column: {column}"));
        plain_text.draw(&mut termio, row, column, &None)?;
        termio.flush()?;
    }

    while let Some(event) = termio.wait()? {
        match event {
            Event::WindowSize { rows: _, columns: _ } => {}
            Event::KeyPress { key: Key::Left, alt: false, ctrl: false, shift: false } => {
                column -= 1;
            }
            Event::KeyPress { key: Key::Right, alt: false, ctrl: false, shift: false } => {
                column += 1;
            }
            Event::KeyPress { key: Key::Up, alt: false, ctrl: false, shift: false } => {
                row -= 1;
            }
            Event::KeyPress { key: Key::Down, alt: false, ctrl: false, shift: false } => {
                row += 1;
            }
            Event::KeyPress { key: Key::Char('q'), alt: false, ctrl: false, shift: false } => {
                break;
            }
            Event::KeyPress { key: Key::Char('i'), alt: false, ctrl: false, shift: false } => {
                termio.invert();
            }
            _ => {
                plain_text.append(format!("\n{:?}", event).trim_end());
            }
        }

        termio.clear_screen()?;
        termio.flush()?;
        let mut plain_text = plain_text.clone();
        plain_text.append(&format!("\nrow: {row}, column: {column}"));
        plain_text.draw(&mut termio, row, column, &None)?;
        termio.flush()?;
    }

    drop(termio);
    println!("{:#?}", plain_text);

    Ok(())
}
