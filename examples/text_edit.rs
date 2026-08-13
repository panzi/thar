use std::io::Read;

use thar::{event::{Event, Key}, message::{MessageBroker}, rect::Rect, termio::TermIO, text_editor::TextEditor, widget::{ActionFlags, Widget}};

use clap::Parser;

#[derive(Parser)]
struct Args {
    path: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let mut text = String::new();

    {
        let mut file = std::fs::OpenOptions::new()
            .create(false)
            .read(true)
            .open(&args.path)?;
        file.read_to_string(&mut text)?;
    }

    let mut text_editor = TextEditor::with_text(&text);

    let mut termio = TermIO::from_tty()?;
    let mut broker = MessageBroker::new();

    let mut running = true;

    {
        let window_size = termio.window_size();
        text_editor.set_draw_rect(&Rect {
            row: 0,
            column: 0,
            width: window_size.columns,
            height: window_size.rows,
        });
    }

    termio.clear_screen()?;

    text_editor.draw(&mut termio, 0, 0)?;

    while let Some(event) = termio.wait()? {
        let mut flags = ActionFlags::None;

        match event {
            Event::WindowSize { columns, rows } => {
                text_editor.set_draw_rect(&Rect {
                    row: 0,
                    column: 0,
                    width: columns,
                    height: rows,
                });
                flags |= ActionFlags::Dirty;
            }
            Event::KeyPress { key: Key::Char('q'), ctrl: true, alt: false, shift: false } => {
                running = false;
            }
            Event::KeyPress { key: Key::Char('s'), ctrl: true, alt: false, shift: false } => {
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&args.path)?;

                text_editor.write_to(&mut file)?;
            }
            _ => {}
        }

        flags |= event.send_to(&mut text_editor, &mut broker);
        flags |= broker.deliver(&mut [&mut text_editor]);

        if flags.contains(ActionFlags::Dirty) {
            text_editor.draw(&mut termio, 0, 0)?;
        }

        if !running {
            break;
        }
    }

    Ok(())
}
