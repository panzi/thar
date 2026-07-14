use std::{ffi::OsString, fs::File, io::BufReader};

use crate::{color::Color16, event::{Event, Key}, rich_text::{RichText, RichTextStyle}, schema::HAR, termio::TermIO};

use clap::Parser;

pub mod schema;
pub mod termio;
pub mod event;
pub mod epoll;
pub mod borrowed_fd;
pub mod color;
pub mod char_width;
pub mod widgets;
pub mod style;
pub mod rich_text;

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
    scroll_x: u32,
    scroll_y: u32,
    active_view: ActiveView,
    entry_index: u32,
    page_index: u32,
    requests_label: RichText,
    pages_label: RichText,
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

    if 1 == 1 {
        let rich_text = "[b][color=red]Hello[/color] [i][color=#cccc00]World![/color][/i][/b]\n[bg=magenta]FOO [bg=green]BAR[/bg] BAZ[/bg]";
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
        rich_text.append_plain_text(&format!("\n{:#?}", rich_text));

        let mut termio = termio::TermIO::from_stdio()?;

        let mut x = 1;
        let mut y = 1;

        rich_text.draw(&mut termio, y, x)?;
        termio.flush()?;

        while let Some(event) = termio.wait()? {
            match event {
                Event::WindowSize { rows: _, columns: _ } => {}
                Event::KeyPress { key: Key::Left, alt: false, ctrl: false, shift: false } => {
                    x -= 1;
                }
                Event::KeyPress { key: Key::Right, alt: false, ctrl: false, shift: false } => {
                    x += 1;
                }
                Event::KeyPress { key: Key::Up, alt: false, ctrl: false, shift: false } => {
                    y -= 1;
                }
                Event::KeyPress { key: Key::Down, alt: false, ctrl: false, shift: false } => {
                    y += 1;
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
            rich_text.draw(&mut termio, y, x)?;
            termio.flush()?;
        }

        drop(termio);
        println!("{:#?}", rich_text);

        return Ok(());
    }

    let args = Args::parse();

    let har: HAR = if let Some(path) = args.path {
        let file = File::open(path)?;
        serde_json::from_reader(BufReader::new(file))?
    } else {
        serde_json::from_reader(std::io::stdin())?
    };

    let mut termio = termio::TermIO::from_stdio()?;
    let mut view_state = ViewState::default();

    view_state.requests_label.append_rich_text("[bg=black][color=white][u]R[/u]equests[/color][/bg]").unwrap();
    view_state.pages_label.append_rich_text("[bg=black][color=white][u]P[/u]ages[/color][/bg]").unwrap();

    full_redraw(&har, &mut termio, &view_state)?;

    while let Some(event) = termio.wait()? {
        match event {
            Event::WindowSize { .. } => {
                full_redraw(&har, &mut termio, &view_state)?;
            }
            Event::KeyPress { key: Key::Char('r'), alt: true, ctrl: false, shift: false } => {
                view_state.active_view = ActiveView::EntryList;
                full_redraw(&har, &mut termio, &view_state)?;
            }
            Event::KeyPress { key: Key::Char('p'), alt: true, ctrl: false, shift: false } => {
                view_state.active_view = ActiveView::PageList;
                full_redraw(&har, &mut termio, &view_state)?;
            }
            Event::KeyPress { key: Key::Char('q'), alt: false, ctrl: false, shift: false } => {
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

fn full_redraw(har: &HAR, termio: &mut TermIO, view_state: &ViewState) -> std::io::Result<()> {
    termio.clear_screen()?;

    termio.set_inverted(matches!(view_state.active_view, ActiveView::EntryList | ActiveView::Entry(_)));

    view_state.requests_label.draw(termio, 1, 1)?;

    termio.set_inverted(matches!(view_state.active_view, ActiveView::PageList | ActiveView::Page(_)));

    view_state.pages_label.draw(termio, 1, view_state.requests_label.width().min(i32::MAX as usize - 2) as i32 + 2)?;

    termio.set_inverted(false);

    termio.flush()?;

    match view_state.active_view {
        ActiveView::EntryList => {

        }
        ActiveView::PageList => {

        }
        ActiveView::Entry(index) => {

        }
        ActiveView::Page(index) => {

        }
    }

    termio.flush()?;

    Ok(())
}
