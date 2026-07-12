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
}

fn main() -> Result<(), std::io::Error> {
//  println!("{har:#?}");
    if 1 == 1 {
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
    let args = Args::parse();

    let har: HAR = if let Some(path) = args.path {
        let file = File::open(path)?;
        serde_json::from_reader(BufReader::new(file))?
    } else {
        serde_json::from_reader(std::io::stdin())?
    };

    let mut termio = termio::TermIO::from_stdio()?;
    let mut view_state = ViewState::default();

    full_redraw(&har, &mut termio, &view_state)?;

    while let Some(event) = termio.wait()? {
        match event {
            Event::WindowSize { rows: _, columns: _ } => {
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
    termio.move_cursor(1, 1)?;

    if matches!(view_state.active_view, ActiveView::EntryList | ActiveView::Entry(_)) {
        termio.bg16(Color16::Black)?;
        termio.fg16(Color16::White)?;
    }
    termio.underline()?;
    //termio.text(1, 1, "R")?;
    termio.not_underline()?;
    //termio.text(1, 2, "equests")?;

    // TODO: come up with better system to format text

    if matches!(view_state.active_view, ActiveView::EntryList | ActiveView::Entry(_)) {
        termio.fg_default()?;
        termio.bg_default()?;
    }

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
