use crate::{color::{Color, Color16}, event::{Event, Key}, rich_text::RichText, style::{FontWeight, ScopedTermIOState}, termio::TermIO, widget::{Rect, Widget}};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Table {
    columns: Vec<Column>,
    header: Vec<RichText>,
    rows: Vec<Vec<RichText>>,
    formatted_header: RichText,
    formatted_rows: Vec<RichText>,
    width: usize,
    header_height: usize,
    rows_height: usize,
    scroll_row: u32,
    scroll_column: u32,
    selected_row_index: usize,
    draw_rect: Rect,
}

fn gather_widths(columns: &mut Vec<Column>, row: &[RichText]) {
    for (index, cell) in row.iter().enumerate() {
        let column = if let Some(column) = columns.get_mut(index) {
            column
        } else {
            let mut len = columns.len();
            loop {
                let column = columns.push_mut(Column::default());
                len += 1;
                if len > index {
                    break column;
                }
            }
        };

        if column.width < cell.width() {
            column.width = cell.width();
        }
    }
}

fn format_row(columns: &[Column], row: &[RichText], formatted: &mut RichText) -> usize {
    let mut height = 0;

    for cell in row {
        if cell.height() > height {
            height = cell.height();
        }
    }

    formatted.append_lines(height);

    if columns.is_empty() {
        return 0;
    }

    let last_index = columns.len() - 1;
    let mut width = 0;
    for (index, (column, cell)) in columns.iter().zip(row.iter()).enumerate() {
        if column.align.is_left() {
            width += column.width() + (index != last_index) as usize;
            formatted.vertical_append(cell);
            formatted.right_pad(width);
        } else {
            let mut cell = cell.clone();
            cell.left_pad(column.width());

            formatted.vertical_append(&cell);

            if index != last_index {
                width += column.width() + 1;
                formatted.right_pad(width);
            }
        }
    }

    height
}

impl Table {
    #[inline]
    pub fn new(header: impl Into<Vec<RichText>>, rows: impl Into<Vec<Vec<RichText>>>, align: &[Align]) -> Self {
        let mut table = Self {
            columns: align.iter().cloned().map(Column::new).collect(),
            header: header.into(),
            rows: rows.into(),
            formatted_header: RichText::new(),
            formatted_rows: Vec::new(),
            width: 0,
            header_height: 0,
            rows_height: 0,
            scroll_row: 0,
            scroll_column: 0,
            selected_row_index: 0,
            draw_rect: Rect::default(),
        };

        table.update();

        table
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub fn header_height(&self) -> usize {
        self.header_height
    }

    #[inline]
    pub fn rows_height(&self) -> usize {
        self.rows_height
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.header_height + self.rows_height
    }

    #[inline]
    pub fn scroll_row(&self) -> u32 {
        self.scroll_row
    }

    #[inline]
    pub fn scroll_column(&self) -> u32 {
        self.scroll_column
    }

    #[inline]
    pub fn selected_row_index(&self) -> usize {
        self.selected_row_index
    }

    #[inline]
    pub fn set_scroll_row(&mut self, scroll_row: u32) {
        self.scroll_row = scroll_row;
    }

    #[inline]
    pub fn set_scroll_column(&mut self, scroll_column: u32) {
        self.scroll_column = scroll_column;
    }

    #[inline]
    pub fn set_selected_row_index(&mut self, selected_row_index: usize) {
        self.selected_row_index = selected_row_index;
    }

    pub fn set_columns(&mut self, column_defs: impl IntoIterator<Item = ColumnDef>) {
        self.columns.clear();
        self.header.clear();

        for column_def in column_defs {
            self.columns.push(Column::new(column_def.align));
            self.header.push(column_def.header);
        }
    }

    pub fn update(&mut self) {
        self.formatted_header.clear();
        self.formatted_rows.clear();

        gather_widths(&mut self.columns, &self.header);

        for row in &self.rows {
            gather_widths(&mut self.columns, row);
        }

        self.formatted_rows.reserve(1 + self.rows.len());
        self.header_height = format_row(&self.columns, &self.header, &mut self.formatted_header);

        self.rows_height = 0;
        for row in &self.rows {
            self.rows_height += format_row(&self.columns, row, self.formatted_rows.push_mut(RichText::new()));
        }

        self.width = self.columns.iter().map(Column::width).sum();
    }

    #[inline]
    pub fn header(&self) -> &[RichText] {
        &self.header
    }

    #[inline]
    pub fn header_mut(&mut self) -> &mut Vec<RichText> {
        &mut self.header
    }

    #[inline]
    pub fn rows(&self) -> &[Vec<RichText>] {
        &self.rows
    }

    #[inline]
    pub fn rows_mut(&mut self) -> &mut Vec<Vec<RichText>> {
        &mut self.rows
    }

    #[inline]
    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    #[inline]
    pub fn columns_mut(&mut self) -> &mut Vec<Column> {
        &mut self.columns
    }

    pub fn clamp_scroll(&mut self) {
        let height = self.height();
        if self.draw_rect.height as usize > height {
            self.scroll_row = 0;
        } else {
            let max_overflow = (height - self.draw_rect.height as usize) as u32;

            if self.scroll_row > max_overflow {
                self.scroll_row = max_overflow;
            }
        }

        if self.draw_rect.width as usize > self.width {
            self.scroll_column = 0;
        } else {
            let max_overflow = (self.width - self.draw_rect.width as usize) as u32;

            if self.scroll_column > max_overflow {
                self.scroll_column = max_overflow;
            }
        }
    }
}

impl Widget for Table {
    #[inline]
    fn draw_rect(&self) -> Rect {
        self.draw_rect
    }

    #[inline]
    fn set_draw_rect(&mut self, rect: &Rect) {
        self.draw_rect = *rect;
        self.clamp_scroll();
    }

    fn draw(&self, termio: &mut TermIO) -> std::io::Result<()> {
        let Rect { row, column, width, height } = self.draw_rect;
        let &Table { scroll_row, scroll_column, selected_row_index, .. } = self;

        let even_bg = Color::from_u32(0x555555);
        let odd_bg = Color::Color16(Color16::Black);
        let fg = Color::Color16(Color16::White);

        let mut scoped_state = ScopedTermIOState::default_bg(termio, odd_bg);
        let mut scoped_state = ScopedTermIOState::default_fg(scoped_state.termio_mut(), fg);

        {
            scoped_state.termio_mut().font_weight(FontWeight::Bold)?;

            let res = self.formatted_header.draw_cropped(
                scoped_state.termio_mut(),
                row,
                column,
                0,
                scroll_column,
                width,
                height,
            );

            scoped_state.termio_mut().font_weight(FontWeight::Normal)?;

            res?;
        }

        let header_height = self.formatted_header.height();

        if (height as usize) < header_height {
            return Ok(());
        }

        let mut body_height = 0;
        let mut current_row_index = 0;

        while current_row_index < self.formatted_rows.len() {
            let row_height = self.formatted_rows[current_row_index].height();
            if body_height as i32 + row_height as i32 - scroll_row as i32 >= header_height as i32 {
                break;
            }

            body_height += row_height;
            current_row_index += 1;
        }

        let mut avail_height = height - header_height as u32;

        while current_row_index < self.formatted_rows.len() {
            let table_row = &self.formatted_rows[current_row_index];

            let mut scoped_state = ScopedTermIOState::default_bg(
                scoped_state.termio_mut(),
                if ((current_row_index + scroll_row as usize) & 1) == 0 { even_bg } else { odd_bg }
            );

            let mut scoped_state = if current_row_index == selected_row_index {
                ScopedTermIOState::inverted(scoped_state.termio_mut(), true)
            } else {
                ScopedTermIOState::none(scoped_state.termio_mut())
            };

            table_row.draw_cropped(
                scoped_state.termio_mut(),
                row + header_height as i32 + (body_height as i32 - scroll_row as i32).max(0),
                column,
                0,
                scroll_column,
                width,
                avail_height,
            )?;

            current_row_index += 1;

            let row_height = table_row.height();
            body_height += row_height;
            avail_height = if (avail_height as usize) > row_height {
                avail_height - row_height as u32
            } else {
                break;
            };
        }

        Ok(())
    }

    fn handle_event(&mut self, event: &Event) {
        match event {
            Event::KeyPress { key: Key::Down, alt: false, ctrl: false, shift: false } => {
                if self.scroll_row > 0 {
                    self.scroll_row -= 1;
                    self.clamp_scroll();
                }
            }
            Event::KeyPress { key: Key::Up, alt: false, ctrl: false, shift: false } => {
                if self.scroll_row < u32::MAX {
                    self.scroll_row += 1;
                    self.clamp_scroll();
                }
            }
            Event::KeyPress { key: Key::Right, alt: false, ctrl: false, shift: false } => {
                if self.scroll_column > 0 {
                    self.scroll_column -= 1;
                    self.clamp_scroll();
                }
            }
            Event::KeyPress { key: Key::Left, alt: false, ctrl: false, shift: false } => {
                if self.scroll_column < u32::MAX {
                    self.scroll_column += 1;
                    self.clamp_scroll();
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Column {
    width: usize,
    align: Align,
}

impl Column {
    #[inline]
    pub fn new(align: Align) -> Self {
        Self { width: 0, align }
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub fn align(&self) -> Align {
        self.align
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Align {
    Left,
    Right,
}

impl Align {
    #[inline]
    pub fn is_left(&self) -> bool {
        matches!(self, Self::Left)
    }

    #[inline]
    pub fn is_right(&self) -> bool {
        matches!(self, Self::Right)
    }
}

impl Default for Align {
    #[inline]
    fn default() -> Self {
        Self::Left
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnDef {
    pub header: RichText,
    pub align: Align,
}

impl ColumnDef {
    #[inline]
    pub fn new(header: RichText, align: Align) -> Self {
        Self { header, align }
    }
}
