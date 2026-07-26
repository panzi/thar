use crate::{color::Color, event::{Event, Key}, message::MessageBroker, rect::Rect, rich_text::{DEFAULT_STYLE, RichText, RichTextStyle, line_width, right_pad_line_with}, style::{FontWeight, ScopedTermIOState}, termio::TermIO, widget::{ActionFlags, Widget, WidgetId, next_widget_id}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    columns: Vec<Column>,
    header: Vec<RichText>,
    rows: Vec<Vec<RichText>>,
    formatted_header: RichText,
    formatted_rows: Vec<RichText>,
    draw_rect: Rect,
    widget_id: WidgetId,
    width: usize,
    rows_height: usize,
    selected_row_index: usize,
    scroll_row: u32,
    scroll_column: u32,
}

fn gather_widths(columns: &mut Vec<Column>, row: &[RichText]) {
    if row.len() > columns.len() {
        columns.reserve(row.len() - columns.len());
    }

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

fn format_row(columns: &[Column], row: &[RichText], formatted: &mut RichText, other_styles: &mut Vec<RichTextStyle>) -> usize {
    if columns.is_empty() {
        formatted.bottom_pad(1);
        return formatted.height();
    }

    let max_lines = row.iter()
        .map(RichText::height)
        .max()
        .unwrap_or(0)
        .max(1);

    formatted.bottom_pad(max_lines);

    let other_styles_len = other_styles.len();
    other_styles[..columns.len().min(other_styles_len)].fill(DEFAULT_STYLE);
    other_styles.resize(columns.len(), DEFAULT_STYLE);

    let mut self_style = DEFAULT_STYLE;

    for line_index in 0..max_lines {
        let self_line = if let Some(self_line) = formatted.lines.get_mut(line_index) {
            self_style.apply_changes(self_line);
            self_line
        } else {
            formatted.lines.push_mut(Vec::new())
        };

        self_line.reserve(
            row.len() +
            row.iter()
            .map(|other|
                other.lines.get(line_index)
                .map_or(0, Vec::len)
            ).sum::<usize>()
        );

        let mut actual_line_width = line_width(self_line);
        let mut wanted_line_width = actual_line_width;
        let mut prev_style = &self_style;
        let mut first = true;

        for ((other, other_style), column) in row.iter().zip(&mut other_styles[..]).zip(columns) {
            prev_style.diff(&other_style, self_line);

            if first {
                first = false;
            } else {
                wanted_line_width += 1;
            }

            let column_width = column.width();
            if let Some(other_line) = other.lines.get(line_index) {
                let mut pad_width = wanted_line_width;
                let other_width = line_width(other_line);
                if column.align().is_right() {
                    if other_width < column_width {
                        pad_width += column_width - other_width;
                    }
                }
                right_pad_line_with(self_line, actual_line_width, pad_width);
                actual_line_width = pad_width;
                self_line.extend_from_slice(&other_line);
                actual_line_width += other_width;
                other_style.apply_changes(other_line);
            }
            wanted_line_width += column_width;

            prev_style = other_style;
        }

        right_pad_line_with(self_line, actual_line_width, wanted_line_width);
    }

    formatted.width = columns.iter().map(Column::width).sum::<usize>();
    if !columns.is_empty() {
        formatted.width += columns.len() - 1;
    }
    formatted.height()
}

impl Default for Table {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Table {
    #[inline]
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            header: Vec::new(),
            rows: Vec::new(),
            formatted_header: RichText::new(),
            formatted_rows: Vec::new(),
            draw_rect: Rect::default(),
            widget_id: next_widget_id(),
            width: 0,
            rows_height: 0,
            selected_row_index: 0,
            scroll_row: 0,
            scroll_column: 0,
        }
    }

    #[inline]
    pub fn with_data(header: impl Into<Vec<RichText>>, rows: impl Into<Vec<Vec<RichText>>>, align: &[Align]) -> Self {
        let mut table = Self {
            columns: align.iter().cloned().map(Column::new).collect(),
            header: header.into(),
            rows: rows.into(),
            formatted_header: RichText::new(),
            formatted_rows: Vec::new(),
            width: 0,
            rows_height: 0,
            scroll_row: 0,
            scroll_column: 0,
            selected_row_index: 0,
            widget_id: next_widget_id(),
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
        self.formatted_header.height()
    }

    #[inline]
    pub fn rows_height(&self) -> usize {
        self.rows_height
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.formatted_header.height() + self.rows_height
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

        let mut style_buf = Vec::new();
        self.formatted_rows.reserve(1 + self.rows.len());
        format_row(&self.columns, &self.header, &mut self.formatted_header, &mut style_buf);

        self.rows_height = 0;
        for row in &self.rows {
            self.rows_height += format_row(&self.columns, row, self.formatted_rows.push_mut(RichText::new()), &mut style_buf);
        }

        self.width = self.columns.iter().map(Column::width).sum();

        if self.columns.len() > 0 {
            self.width += self.columns.len() - 1;
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

    pub fn clamp_scroll_row(&mut self) {
        let height = self.height();
        if self.draw_rect.height as usize > height {
            self.scroll_row = 0;
        } else {
            let max_overflow = (height - self.draw_rect.height as usize) as u32;

            if self.scroll_row > max_overflow {
                self.scroll_row = max_overflow;
            }
        }
    }

    pub fn clamp_scroll_column(&mut self) {
        if self.draw_rect.width as usize > self.width {
            self.scroll_column = 0;
        } else {
            let max_overflow = (self.width - self.draw_rect.width as usize) as u32;

            if self.scroll_column > max_overflow {
                self.scroll_column = max_overflow;
            }
        }
    }

    fn after_selection_up(&mut self) {
        if self.header_height() >= self.draw_rect.height as usize {
            self.scroll_row = 0;
        } else {
            let avail_height = self.draw_rect.height as usize - self.header_height();
            let mut body_height = 0;
            let mut selected_top = 0;
            let mut selected_height = 0;

            for (index, row) in self.formatted_rows.iter().enumerate() {
                if index == self.selected_row_index {
                    selected_top = body_height;
                    selected_height = row.height();
                    break;
                }
                body_height += row.height();
            }

            if selected_top < self.scroll_row as usize {
                self.scroll_row = selected_top as u32;
            } else if selected_top + selected_height > self.scroll_row as usize + avail_height {
                if selected_height > avail_height {
                    self.scroll_row = selected_top as u32;
                } else {
                    self.scroll_row = (selected_top + selected_height - avail_height) as u32;
                }
            }
        }
    }

    fn after_selection_down(&mut self) {
        if self.header_height() >= self.draw_rect.height as usize {
            self.scroll_row = 0;
        } else {
            let avail_height = self.draw_rect.height as usize - self.header_height();
            let mut body_height = 0;
            let mut selected_top = 0;
            let mut selected_height = 0;

            for (index, row) in self.formatted_rows.iter().enumerate() {
                if index == self.selected_row_index {
                    selected_top = body_height;
                    selected_height = row.height();
                    break;
                } else {
                    body_height += row.height();
                }
            }

            if selected_top + selected_height > self.scroll_row as usize + avail_height {
                if selected_height > avail_height {
                    self.scroll_row = selected_top as u32;
                } else {
                    self.scroll_row = (selected_top + selected_height - avail_height) as u32;
                }
            } else if selected_top < self.scroll_row as usize {
                self.scroll_row = selected_top as u32;
            }
        }
    }
}

const EVEN_BACKGROUND:          Color = Color::from_u32(0x111111);
const ODD_BACKGROUND:           Color = Color::from_u32(0x000000);
const SELECTED_EVEN_BACKGROUND: Color = Color::from_u32(0x333333);
const SELECTED_ODD_BACKGROUND:  Color = Color::from_u32(0x222222);
const FOREGROUND:               Color = Color::from_u32(0xFFFFFF);

impl Widget for Table {
    #[inline]
    fn widget_id(&self) -> WidgetId {
        self.widget_id
    }

    #[inline]
    fn draw_rect(&self) -> &Rect {
        &self.draw_rect
    }

    #[inline]
    fn set_draw_rect(&mut self, rect: &Rect) {
        self.draw_rect = *rect;
        self.clamp_scroll_row();
        self.clamp_scroll_column();
    }

    fn draw(&self, termio: &mut TermIO, parent_row: i32, parent_column: i32) -> std::io::Result<()> {
        let Rect { row, column, width, height } = self.draw_rect;
        let &Table { scroll_row, scroll_column, selected_row_index, .. } = self;
        let row    = row    + parent_row;
        let column = column + parent_column;

        let mut scoped_state = ScopedTermIOState::default_bg(termio, ODD_BACKGROUND);
        let mut scoped_state = ScopedTermIOState::default_fg(scoped_state.termio_mut(), FOREGROUND);

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

        let mut avail_height = height - header_height as u32 + scroll_row;

        while current_row_index < self.formatted_rows.len() {
            let table_row = &self.formatted_rows[current_row_index];

            let mut scoped_state = ScopedTermIOState::default_bg(
                scoped_state.termio_mut(),
                if ((current_row_index + scroll_row as usize) & 1) == 0 {
                    if current_row_index == selected_row_index { SELECTED_EVEN_BACKGROUND } else { EVEN_BACKGROUND }
                } else {
                    if current_row_index == selected_row_index { SELECTED_ODD_BACKGROUND } else { ODD_BACKGROUND }
                }
            );

            let offset_body_height = body_height as i32 - scroll_row as i32;

            table_row.draw_cropped(
                scoped_state.termio_mut(),
                row + header_height as i32 + offset_body_height.max(0),
                column,
                -offset_body_height.min(0) as u32,
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

    fn handle_event(&mut self, event: &Event, broker: &mut MessageBroker) -> ActionFlags {
        match event {
            Event::KeyPress { key: Key::Enter, alt: false, ctrl: false, shift: false } => {
                if self.selected_row_index < self.rows.len() {
                    broker.dispatch(SelectTableRow {
                        widget_id: self.widget_id,
                        row_index: self.selected_row_index,
                    });
                }
            }
            &Event::KeyPress { key: Key::Up, alt, ctrl: false, shift: false } => {
                if alt {
                    if self.scroll_row > 0 {
                        self.scroll_row -= 1;
                        return ActionFlags::Redraw;
                    }
                } else if self.selected_row_index > 0 {
                    self.selected_row_index -= 1;

                    self.after_selection_up();
                    return ActionFlags::Redraw;
                }
            }
            Event::KeyPress { key: Key::Home, alt: false, ctrl: false, shift: false } => {
                if self.selected_row_index > 0 {
                    self.selected_row_index = 0;

                    self.after_selection_up();
                    return ActionFlags::Redraw;
                }
            }
            &Event::KeyPress { key: Key::Down, alt, ctrl: false, shift: false } => {
                if alt {
                    if self.scroll_row < u32::MAX {
                        let scroll_row = self.scroll_row;
                        self.scroll_row += 1;
                        self.clamp_scroll_row();
                        if self.scroll_row != scroll_row {
                            return ActionFlags::Redraw;
                        }
                    }
                } else if self.formatted_rows.is_empty() {
                    if self.selected_row_index != 0 || self.scroll_row != 0 {
                        self.selected_row_index = 0;
                        self.scroll_row = 0;
                        return ActionFlags::Redraw;
                    }
                } else if self.selected_row_index < self.formatted_rows.len() - 1 {
                    self.selected_row_index += 1;
                    self.after_selection_down();
                    return ActionFlags::Redraw;
                }
            }
            Event::KeyPress { key: Key::End, alt: false, ctrl: false, shift: false } => {
                if self.formatted_rows.is_empty() {
                    if self.selected_row_index != 0 || self.scroll_row != 0 {
                        self.selected_row_index = 0;
                        self.scroll_row = 0;
                        return ActionFlags::Redraw;
                    }
                } else if self.selected_row_index < self.formatted_rows.len() - 1 {
                    self.selected_row_index = self.formatted_rows.len() - 1;
                    self.after_selection_down();
                    return ActionFlags::Redraw;
                }
            }
            Event::KeyPress { key: Key::Left, alt: false, ctrl: false, shift: false } => {
                if self.scroll_column > 0 {
                    self.scroll_column -= 1;
                    return ActionFlags::Redraw;
                }
            }
            Event::KeyPress { key: Key::Right, alt: false, ctrl: false, shift: false } => {
                if self.scroll_column < u32::MAX {
                    let scroll_column = self.scroll_column;
                    self.scroll_column += 1;
                    self.clamp_scroll_column();
                    if self.scroll_column != scroll_column {
                        return ActionFlags::Redraw;
                    }
                }
            }
            Event::KeyPress { key: Key::Home, alt: false, ctrl: true, shift: false } => {
                if self.scroll_column > 0 {
                    self.scroll_column = 0;
                    return ActionFlags::Redraw;
                }
            }
            Event::KeyPress { key: Key::End, alt: false, ctrl: true, shift: false } => {
                if self.scroll_column < u32::MAX {
                    if self.draw_rect.width as usize > self.width {
                        if self.scroll_column != 0 {
                            self.scroll_column = 0;
                            return ActionFlags::Redraw;
                        }
                    } else {
                        let max_overflow = (self.width - self.draw_rect.width as usize) as u32;
                        if self.scroll_column != max_overflow {
                            self.scroll_column = max_overflow;
                            return ActionFlags::Redraw;
                        }
                    }
                }
            }
            Event::KeyPress { key: Key::PageUp, alt: false, ctrl: false, shift: false } => {
                if self.header_height() >= self.draw_rect.height as usize {
                    if self.scroll_row != 0 {
                        self.scroll_row = 0;
                        return ActionFlags::Redraw;
                    }
                } else if self.selected_row_index > 0 {
                    let avail_height = self.draw_rect.height as usize - self.header_height();
                    let mut body_height = 0;
                    let mut selected_top = 0;

                    for (index, row) in self.formatted_rows.iter().enumerate() {
                        if index == self.selected_row_index {
                            selected_top = body_height;
                            break;
                        }
                        body_height += row.height();
                    }

                    let mut page_height = 0;
                    let old_selected_top = selected_top;

                    while page_height < avail_height && self.selected_row_index > 0 {
                        self.selected_row_index -= 1;

                        let row = &self.formatted_rows[self.selected_row_index];
                        let selected_height = row.height();
                        selected_top -= selected_height;
                        page_height += selected_height;
                    }

                    let scroll_diff = old_selected_top - selected_top;
                    if scroll_diff > self.scroll_row as usize {
                        // can't happen
                        if self.scroll_row != 0 {
                            self.scroll_row = 0;
                            return ActionFlags::Redraw;
                        }
                    } else {
                        self.scroll_row -= scroll_diff as u32;
                        return ActionFlags::Redraw;
                    }
                }
            }
            Event::KeyPress { key: Key::PageDown, alt: false, ctrl: false, shift: false } => {
                if self.header_height() >= self.draw_rect.height as usize {
                    if self.scroll_row != 0 {
                        self.scroll_row = 0;
                        return ActionFlags::Redraw;
                    }
                } else if self.selected_row_index < self.formatted_rows.len() {
                    let avail_height = self.draw_rect.height as usize - self.header_height();
                    let mut body_height = 0;
                    let mut selected_top = 0;

                    for (index, row) in self.formatted_rows.iter().enumerate() {
                        if index == self.selected_row_index {
                            selected_top = body_height;
                            break;
                        }
                        body_height += row.height();
                    }

                    let mut page_height = 0;
                    let old_selected_top = selected_top;

                    while page_height < avail_height && self.selected_row_index + 1 < self.formatted_rows.len() {
                        self.selected_row_index += 1;

                        let row = &self.formatted_rows[self.selected_row_index];
                        let selected_height = row.height();
                        selected_top += selected_height;
                        page_height += selected_height;
                    }

                    let scroll_diff = selected_top - old_selected_top;
                    if scroll_diff > 0 {
                        self.scroll_row += scroll_diff as u32;
                        self.clamp_scroll_row();
                        return ActionFlags::Redraw;
                    }
                }
            }
            _ => {}
        }

        ActionFlags::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectTableRow {
    pub widget_id: WidgetId,
    pub row_index: usize,
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
