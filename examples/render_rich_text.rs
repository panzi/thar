use thar::{color::Color16, event::{Event, Key}, rich_text::{RichText, RichTextStyle}, termio};


fn main() -> std::io::Result<()> {
    let rich_text = "\
[b][color=red]Hello[/color] [i][color=#cccc00]World![/color][/i][/b]\n[bg=magenta]FOO [bg=green]BAR[/bg] BAZ[/bg]
This is a long line demonstrating how things are truncated.
A second long line to verify this works for all lines.
A last line.";
    //let rich_text = "[bg=black][color=white][u]R[/u]equests[/color][/bg]";
    //let rich_text = "[u]R[/u]equests";
    let mut rich_text = match RichText::parse(rich_text) {
        Ok(rich_text) => rich_text,
        Err(error) => {
            eprintln!("{:#?}\n{error}", error.location());
            error.print_line(&rich_text, &mut std::io::stderr())?;
            std::process::exit(1);
        }
    };
    //rich_text.append_plain_text(&format!("\n{:#?}", rich_text));

    let mut termio = termio::TermIO::from_stdio()?;

    let mut column = 0;
    let mut row = 0;

    {
        let mut rich_text = rich_text.clone();
        rich_text.append_plain_text(&format!("\nrow: {row}, column: {column}"));
        rich_text.draw(&mut termio, row, column)?;
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
                rich_text.append_text(
                    &RichTextStyle::build().foreground(Color16::Red.into()).into_inner(),
                    format!("\n{:?}", event).trim_end()
                );
            }
        }

        termio.clear_screen()?;
        termio.flush()?;
        let mut rich_text = rich_text.clone();
        rich_text.append_plain_text(&format!("\nrow: {row}, column: {column}"));
        rich_text.draw(&mut termio, row, column)?;
        termio.flush()?;
    }

    drop(termio);
    println!("{:#?}", rich_text);

    Ok(())
}
