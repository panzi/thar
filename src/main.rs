use std::{ffi::OsString, fs::File, io::BufReader};

use crate::{color::{Color, Color16}, event::{Event, Key}, fields::{ContentField, EntryField, Field, RequestField, ResponseField}, rich_text::{RichText, RichTextStyle}, schema::HAR, table::Table, termio::TermIO, widget::{Rect, Widget}};

use clap::Parser;

pub mod schema;
pub mod termio;
pub mod event;
pub mod epoll;
pub mod borrowed_fd;
pub mod color;
pub mod char_width;
pub mod style;
pub mod rich_text;
pub mod fields;
pub mod table;
pub mod widget;

#[derive(Parser)]
struct Args {
    path: Option<OsString>,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    EntryList,
    PageList,
    Entry(usize),
    Page(usize),
}

impl Default for ActiveView {
    #[inline]
    fn default() -> Self {
        Self::EntryList
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ViewState {
    active_view: ActiveView,
    requests_label: RichText,
    pages_label: RichText,
    entries_table: Table,
    pages_table: Table,
}

fn main() -> Result<(), std::io::Error> {
//  println!("{har:#?}");
    if 1 == 2 {
        for arg in std::env::args().skip(1) {
            match RichText::parse(&arg) {
                Ok(rich_text) => {
                    println!("{:#?}", rich_text);
                }
                Err(error) => {
                    eprintln!("{:#?}\n{error}", error.location());
                    error.print_line(&arg, &mut std::io::stderr())?;
                }
            }
        }
        return Ok(());
    }

    if 1 == 2 {
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

        return Ok(());
    }

    if 1 == 2 {
        // debug events
        let mut termio = termio::TermIO::from_tty()?;

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

        return Ok(());
    }

    let args = Args::parse();

    let har: HAR = if let Some(path) = args.path {
        let file = File::open(path)?;
        serde_json::from_reader(BufReader::new(file))?
    } else {
        serde_json::from_reader(std::io::stdin())?
    };

    let mut termio = termio::TermIO::from_tty()?;
    let mut view_state = ViewState::default();

    view_state.requests_label.append_rich_text("[bg=black][color=white][u]R[/u]equests[/color][/bg]").unwrap();
    view_state.pages_label.append_rich_text("[bg=black][color=white][u]P[/u]ages[/color][/bg]").unwrap();

    let entry_columns = [
        EntryField::Request(RequestField::Method),
        EntryField::Request(RequestField::Url),
        EntryField::Response(ResponseField::Status),
        EntryField::Response(ResponseField::StatusText),
        EntryField::Response(ResponseField::Content(ContentField::MimeType)),
        EntryField::Response(ResponseField::HeadersSize),
        EntryField::Response(ResponseField::BodySize),
        EntryField::StartedDateTime,
        EntryField::Time,
    ];

    view_state.entries_table.set_columns(entry_columns.map(Into::into));

    let mut buf = String::new();
    for entry in &har.log.entries {
        let mut row = Vec::new();
        for column in &entry_columns {
            buf.clear();

            let mut cell = RichText::new();
            column.write_rich_text(entry, &mut cell, &mut buf).unwrap();

            row.push(cell);
        }
        view_state.entries_table.rows_mut().push(row);
    }

    {
        let window_size = termio.window_size();
        view_state.entries_table.set_draw_rect(&Rect {
            row: 1,
            column: 0,
            width: window_size.columns,
            height: if window_size.rows > 0 { window_size.rows - 1 } else { 0 },
        });
    }

    view_state.entries_table.update();

    full_redraw(&mut termio, &view_state)?;

    while let Some(event) = termio.wait()? {
        match event {
            Event::WindowSize { columns, rows } => {
                view_state.entries_table.set_draw_rect(&Rect {
                    row: 1,
                    column: 0,
                    width: columns,
                    height: if rows > 0 { rows - 1 } else { 0 },
                });
            }
            Event::KeyPress { key: Key::Char('r'), alt: true, ctrl: false, shift: false } => {
                view_state.active_view = ActiveView::EntryList;
            }
            Event::KeyPress { key: Key::Char('p'), alt: true, ctrl: false, shift: false } => {
                view_state.active_view = ActiveView::PageList;
            }
            Event::KeyPress { key: Key::Char('q'), alt: false, ctrl: false, shift: false } => {
                break;
            }
            _ => {
                match view_state.active_view {
                    ActiveView::EntryList => {
                        view_state.entries_table.handle_event(&event);
                    }
                    _ => {}
                }
            }
        }

        full_redraw(&mut termio, &view_state)?;
    }

    //drop(termio);
    //println!("{}", view_state.entries_table.columns().iter().map(|row| row.width()).sum::<usize>() + view_state.entries_table.columns().len() - 1);
    //println!("{:#?}", view_state.entries_table.formatted_rows[0].width());
    //println!("{:?}", view_state.entries_table.columns().iter().map(Column::width).collect::<Vec<usize>>());
    //for (index, row) in view_state.entries_table.formatted_rows.iter().enumerate() {
    //    println!("{index} {}", row.width());
    //}

    Ok(())
}

fn full_redraw(termio: &mut TermIO, view_state: &ViewState) -> std::io::Result<()> {
    termio.clear_style()?;
    termio.clear_screen()?;

    termio.set_inverted(matches!(view_state.active_view, ActiveView::EntryList | ActiveView::Entry(_)));

    view_state.requests_label.draw(termio, 0, 0)?;

    termio.set_inverted(matches!(view_state.active_view, ActiveView::PageList | ActiveView::Page(_)));

    view_state.pages_label.draw(termio, 0, view_state.requests_label.width().min(i32::MAX as usize - 1) as i32 + 1)?;

    termio.set_inverted(false);

    let window_size = *termio.window_size();

    if window_size.rows > 0 {
        match view_state.active_view {
            ActiveView::EntryList => {
                //view_state.entries_table.formatted_rows[0].draw(termio, 1, 0)?;
                view_state.entries_table.draw(termio)?;
            }
            ActiveView::PageList => {

            }
            ActiveView::Entry(index) => {

            }
            ActiveView::Page(index) => {

            }
        }
    }

    termio.set_default_fg(Color::Default);
    termio.set_default_bg(Color::Default);

    termio.flush()?;

    Ok(())
}
