use std::{ffi::OsString, fs::File, io::BufReader};

use crate::{app::{App, AppConfig, AppQuit}, event::Event, fields::{ContentField, EntryField, PageField, RequestField, ResponseField}, message::{MessageBroker, MessageReceiver}, rect::Rect, schema::HAR, widget::Widget};

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
pub mod message;
pub mod app;
pub mod request_view;
pub mod page_view;
pub mod property_list;
pub mod colorize;

#[derive(Parser)]
struct Args {
    #[clap(long, use_value_delimiter = true, value_delimiter = ',')]
    request_columns: Option<Vec<EntryField>>,

    #[clap(long, use_value_delimiter = true, value_delimiter = ',')]
    page_columns: Option<Vec<PageField>>,

    path: Option<OsString>,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let har: HAR = if let Some(path) = args.path {
        let file = File::open(path)?;
        serde_json::from_reader(BufReader::new(file))?
    } else {
        serde_json::from_reader(std::io::stdin())?
    };

    // {
    //     // DEBUG: make list big
    //     let entry_len = har.log.entries.len();
    //     for _ in 0..10 {
    //         har.log.entries.extend_from_within(0..entry_len);
    //     }
    // }

    let mut broker = MessageBroker::new();
    let mut app = App::new(har, &AppConfig {
        request_columns: args.request_columns.as_ref().map_or(&[
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
        ], |fields| fields),
        page_columns: args.page_columns.as_ref().map_or(&[
            PageField::Index,
            PageField::Id,
            PageField::StartedDateTime,
            PageField::Title,
        ], |fields| fields)
    });

    let mut termio = termio::TermIO::from_tty()?;

    {
        let window_size = termio.window_size();
        app.set_draw_rect(&Rect {
            row: 0,
            column: 0,
            width: window_size.columns,
            height: window_size.rows,
        });
    }

    let mut handler = MessageHandler {
        running: true
    };

    app.draw(&mut termio, 0, 0)?;

    while let Some(event) = termio.wait()? {
        match event {
            Event::WindowSize { columns, rows } => {
                app.set_draw_rect(&Rect {
                    row: 0,
                    column: 0,
                    width: columns,
                    height: rows,
                });
            }
            _ => {}
        }

        event.send_to(&mut app, &mut broker);
        broker.deliver(&mut [&mut handler, &mut app]);

        if !handler.running {
            break;
        }

        app.draw(&mut termio, 0, 0)?;
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

struct MessageHandler {
    running: bool
}

impl MessageReceiver for MessageHandler {
    fn handle_message(&mut self, message: &mut message::Message, _broker: &mut MessageBroker) {
        if let Some(AppQuit) = message.data() {
            self.running = false;
        }
    }
}
