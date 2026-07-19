use std::{ffi::OsString, fs::File, io::BufReader};

use crate::{color::{Color, Color16}, event::{Event, Key}, fields::{ContentField, EntryField, Field, PageField, RequestField, ResponseField}, rect::Rect, rich_text::{RichText, RichTextStyle}, schema::HAR, table::Table, tabs::{Tab, Tabs}, termio::TermIO, widget::Widget};

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
pub mod tabs;
pub mod rect;
pub mod point;

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

#[derive(Debug, Default)]
pub struct ViewState {
    active_view: ActiveView,
    tabs: Tabs,
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

    let mut har: HAR = if let Some(path) = args.path {
        let file = File::open(path)?;
        serde_json::from_reader(BufReader::new(file))?
    } else {
        serde_json::from_reader(std::io::stdin())?
    };

    {
        // DEBUG: make list big
        let entry_len = har.log.entries.len();
        for _ in 0..10 {
            har.log.entries.extend_from_within(0..entry_len);
        }
    }

    let mut termio = termio::TermIO::from_tty()?;
    let mut view_state = ViewState::default();

    {
        let mut entries_table = Table::default();
        let mut pages_table = Table::default();

        // format entries table
        let entry_columns = [
            EntryField::Index,
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

        entries_table.set_columns(entry_columns.map(Into::into));

        let mut buf = String::new();
        for (index, entry) in har.log.entries.iter().enumerate() {
            let mut row = Vec::new();
            for column in &entry_columns {
                buf.clear();

                let mut cell = RichText::new();
                column.write_rich_text(index, entry, &mut cell, &mut buf).unwrap();

                row.push(cell);
            }
            entries_table.rows_mut().push(row);
        }


        entries_table.update();

        // format pages table
        let page_columns = [
            PageField::Index,
            PageField::Id,
            PageField::StartedDateTime,
            PageField::Title,
        ];

        pages_table.set_columns(page_columns.map(Into::into));

        for (index, page) in har.log.pages.iter().enumerate() {
            let mut row = Vec::new();
            for column in &page_columns {
                buf.clear();

                let mut cell = RichText::new();
                column.write_rich_text(index, page, &mut cell, &mut buf).unwrap();

                row.push(cell);
            }
            pages_table.rows_mut().push(row);
        }

        pages_table.update();

        view_state.tabs = Tabs::new([
            Tab {
                title: "Requests".to_string(),
                mnemonic: 'R',
                content: Box::new(entries_table),
            },
            Tab {
                title: "Pages".to_string(),
                mnemonic: 'P',
                content: Box::new(pages_table),
            },
        ]);
    }

    {
        let window_size = termio.window_size();
        view_state.tabs.set_draw_rect(&Rect {
            row: 0,
            column: 0,
            width: window_size.columns,
            height: window_size.rows,
        });
    }


    full_redraw(&mut termio, &view_state)?;

    while let Some(event) = termio.wait()? {
        match event {
            Event::WindowSize { columns, rows } => {
                view_state.tabs.set_draw_rect(&Rect {
                    row: 0,
                    column: 0,
                    width: columns,
                    height: rows,
                });
            }
            Event::KeyPress { key: Key::Char('q'), alt: false, ctrl: false, shift: false } => {
                break;
            }
            _ => {}
        }

        event.send_to(&mut view_state.tabs);

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

    let res = view_state.tabs.draw(termio, 0, 0);

    termio.set_default_fg(Color::Default);
    termio.set_default_bg(Color::Default);

    termio.flush()?;

    res?;

    Ok(())
}
