use crate::point::Point;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub row: i32,
    pub column: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    #[inline]
    pub fn from_corners(top: i32, bottom: i32, left: i32, right: i32) -> Self {
        Self {
            row:    top,
            column: left,
            width:  if right  > left { right  - left } else { 0 } as u32,
            height: if bottom > top  { bottom - top  } else { 0 } as u32,
        }
    }

    #[inline]
    pub fn from_points(top_left: &Point, bottom_right: &Point) -> Self {
        Self::from_corners(top_left.row, bottom_right.row, top_left.column, bottom_right.column)
    }

    #[inline]
    pub fn top(&self) -> i32 {
        self.row
    }

    #[inline]
    pub fn bottom(&self) -> i32 {
        self.row + self.height as i32
    }

    #[inline]
    pub fn left(&self) -> i32 {
        self.column
    }

    #[inline]
    pub fn right(&self) -> i32 {
        self.column + self.width as i32
    }

    #[inline]
    pub fn contains(&self, row: i32, column: i32) -> bool {
        row >= self.row && row < self.bottom() && column >= self.column && column < self.right()
    }

    #[inline]
    pub fn contains_point(&self, point: &Point) -> bool {
        self.contains(point.row, point.column)
    }

    #[inline]
    pub fn top_left(&self) -> Point {
        Point { row: self.row, column: self.column }
    }

    #[inline]
    pub fn bottom_right(&self) -> Point {
        Point { row: self.bottom(), column: self.right() }
    }

    #[inline]
    pub fn contains_rect(&self, other: &Rect) -> bool {
        other.row >= self.row && other.bottom() <= self.bottom() &&
        other.column >= self.column && other.right() <= self.right()
    }

    #[inline]
    pub fn overlaps(&self, other: &Rect) -> bool {
        (if other.row    >= self.row    { other.row    < self.bottom() } else { self.row    < other.bottom() }) &&
        (if other.column >= self.column { other.column < self.right()  } else { self.column < other.right()  })
    }

    pub fn overlap(&self, other: &Rect) -> Rect {
        let top    = self.top().max(other.top());
        let bottom = self.bottom().min(other.bottom());
        let left   = self.left().max(other.left());
        let right  = self.right().min(other.right());

        Rect::from_corners(top, bottom, left, right)
    }
}
