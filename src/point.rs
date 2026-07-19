use std::ops::{Add, AddAssign, Sub, SubAssign};

use crate::rect::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Point {
    pub row: i32,
    pub column: i32,
}

impl Point {
    #[inline]
    pub fn nest_into(&self, rect: &Rect) -> Option<Point> {
        nest_into(self.row, self.column, rect)
    }
}

pub fn nest_into(row: i32, column: i32, rect: &Rect) -> Option<Point> {
    if row < rect.row || row >= rect.bottom() || column < rect.column || column >= rect.right() {
        return None;
    }

    Some(Point {
        row:    row    - rect.row,
        column: column - rect.column,
    })
}

impl Sub for &Point {
    type Output = Point;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Point {
            row: self.row - rhs.row,
            column: self.column - rhs.column,
        }
    }
}

impl SubAssign<&Point> for Point {
    #[inline]
    fn sub_assign(&mut self, rhs: &Point) {
        self.row -= rhs.row;
        self.column -= rhs.column;
    }
}


impl Add for &Point {
    type Output = Point;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Point {
            row: self.row + rhs.row,
            column: self.column + rhs.column,
        }
    }
}

impl AddAssign<&Point> for Point {
    #[inline]
    fn add_assign(&mut self, rhs: &Point) {
        self.row += rhs.row;
        self.column += rhs.column;
    }
}
