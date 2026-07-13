use crate::{rich_text::RichText, termio::TermIO};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Table {
    rows: Vec<Row>,
}

impl Table {
    pub fn redraw(&self, termio: &mut TermIO, row: u32, column: u32, row_index: usize) -> std::io::Result<()> {
        // TODO
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Row {
    cells: Vec<RichText>,
    cached: RichText,
}

impl Row {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, cell: RichText) {
        self.cached.right_pad(1);
        self.cached.vertical_append(&cell);
        self.cells.push(cell);
    }
}
