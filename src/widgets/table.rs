use crate::{color::{Color, Color16}, rich_text::RichText, style::{FontWeight, ScopedTermIOState}, termio::TermIO};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Table {
    columns: Vec<Column>,
    header: Vec<RichText>,
    rows: Vec<Vec<RichText>>,
    formatted_header: RichText,
    formatted_rows: Vec<RichText>,
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

fn format_row(columns: &[Column], row: &[RichText], formatted: &mut RichText) {
    let mut height = 0;

    for cell in row {
        if cell.height() > height {
            height = cell.height();
        }
    }

    formatted.append_lines(height);

    if columns.is_empty() {
        return;
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
        };

        table.format();

        table
    }

    pub fn set_columns(&mut self, column_defs: impl IntoIterator<Item = ColumnDef>) {
        self.columns.clear();
        self.header.clear();

        for column_def in column_defs {
            self.columns.push(Column::new(column_def.align));
            self.header.push(column_def.header);
        }
    }

    pub fn format(&mut self) {
        self.formatted_header.clear();
        self.formatted_rows.clear();

        gather_widths(&mut self.columns, &self.header);

        for row in &self.rows {
            gather_widths(&mut self.columns, row);
        }

        self.formatted_rows.reserve(1 + self.rows.len());
        format_row(&self.columns, &self.header, &mut self.formatted_header);

        for row in &self.rows {
            format_row(&self.columns, row, self.formatted_rows.push_mut(RichText::new()));
        }
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

    pub fn redraw(&self, termio: &mut TermIO, row: i32, column: i32, width: u32, height: u32, scroll_row: u32, scroll_column: u32, selected_row_index: usize) -> std::io::Result<()> {
        let even_bg = Color::from_u32(0x555555);
        let odd_bg = Color::Color16(Color16::Black);
        let fg = Color::Color16(Color16::White);

        let mut scoped_state = ScopedTermIOState::default_bg(termio, odd_bg);
        let mut scoped_state = ScopedTermIOState::default_fg(scoped_state.termio_mut(), fg);

        {
            scoped_state.termio_mut().font_weight(FontWeight::Bold)?;

            let res = self.formatted_header.draw_cropped(
                scoped_state.termio_mut(),
                row + scroll_row as i32,
                column + scroll_column as i32,
                width,
                height,
            );

            scoped_state.termio_mut().font_weight(FontWeight::Normal)?;

            res?;
        }

        let mut acc_height = self.formatted_header.height();

        let mut current_row_index = 0;

        // XXX: lots of bugs
        while current_row_index < self.formatted_rows.len() {
            let row_height = self.formatted_rows[current_row_index].height();
            if acc_height + row_height >= scroll_row as usize {
                break;
            }

            acc_height += row_height;
            current_row_index += 1;
        }

        let end_height = height as usize + scroll_row as usize;

        while current_row_index < self.formatted_rows.len() {
            if acc_height >= end_height {
                break;
            }

            let table_row = &self.formatted_rows[current_row_index];

            let mut scoped_state = ScopedTermIOState::default_bg(
                scoped_state.termio_mut(),
                if (current_row_index & 1) == 0 { even_bg } else { odd_bg }
            );

            let mut scoped_state = if current_row_index == selected_row_index {
                ScopedTermIOState::inverted(scoped_state.termio_mut(), true)
            } else {
                ScopedTermIOState::none(scoped_state.termio_mut())
            };

            let res = table_row.draw_cropped(
                scoped_state.termio_mut(),
                row + scroll_row as i32 + acc_height as i32,
                column + scroll_column as i32,
                width,
                height - acc_height as u32,
            );

            res?;

            acc_height += table_row.height();
            current_row_index += 1;
        }

        Ok(())
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
