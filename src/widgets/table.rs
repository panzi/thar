use crate::{rich_text::RichText, termio::TermIO};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Table {
    columns: Vec<Column>,
    header: Vec<RichText>,
    rows: Vec<Vec<RichText>>,
    formatted: Vec<RichText>,
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

fn format_row(columns: &[Column], row: &[RichText]) -> RichText {
    let mut formatted = RichText::new();
    let mut height = 0;

    for cell in row {
        if cell.height() > height {
            height = cell.height();
        }
    }

    formatted.append_lines(height);

    if columns.is_empty() {
        return formatted;
    }

    let last_index = columns.len() - 1;
    let mut width = 0;
    for (index, (column, cell)) in columns.iter().zip(row.iter()).enumerate() {
        width += column.width() + (index != last_index) as usize;
        formatted.vertical_append(cell);
        if column.align.is_left() {
            formatted.right_pad(width);
        } else {
            formatted.left_pad(width);
        }
    }

    formatted
}

impl Table {
    #[inline]
    pub fn new(header: impl Into<Vec<RichText>>, rows: impl Into<Vec<Vec<RichText>>>, align: &[Align]) -> Self {
        let mut table = Self {
            columns: align.iter().cloned().map(Column::new).collect(),
            header: header.into(),
            rows: rows.into(),
            formatted: Vec::new(),
        };

        table.format();

        table
    }

    pub fn format(&mut self) {
        self.formatted.clear();

        gather_widths(&mut self.columns, &self.header);

        for row in &self.rows {
            gather_widths(&mut self.columns, row);
        }

        self.formatted.reserve(1 + self.rows.len());
        self.formatted.push(format_row(&self.columns, &self.header));

        for row in &self.rows {
            self.formatted.push(format_row(&self.columns, row));
        }
    }

    #[inline]
    pub fn rows(&self) -> &[Vec<RichText>] {
        &self.rows
    }

    #[inline]
    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    #[inline]
    pub fn rows_mut(&mut self) -> &mut Vec<Vec<RichText>> {
        &mut self.rows
    }

    pub fn redraw(&self, termio: &mut TermIO, row: u32, column: u32, width: u32, height: u32, scroll_row: u32, scroll_column: u32, row_index: usize) -> std::io::Result<()> {
        // TODO

        let mut height = 0;

        let mut row_iter = self.formatted.iter();

        while let Some(row) = row_iter.next() {
            if height + row.height() >= scroll_row as usize {
                // TODO
                break;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
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
