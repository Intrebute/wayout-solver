use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

/// Describes a generic rectangular grid of `V`s.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grid<V> {
    data: Vec<V>,
    width: usize,
    height: usize,
}

/// Encodes a position in a grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

/// Encodes the four orthogonal directions.
#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Computes the opposite direction of `self`.
    pub fn op(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

impl Position {
    pub fn new(row: usize, col: usize) -> Self {
        Position { row, col }
    }

    /// Iterates through a the positions of a grid of size `width * height` in row-major order.
    pub fn iter_row_major(width: usize, height: usize) -> impl Iterator<Item = Position> {
        (0..height).flat_map(move |row| (0..width).map(move |col| Position { row, col }))
    }

    /// Checks whether `self` is inside the bounds of `g`.
    pub fn is_in_bounds_of<V>(self, g: &Grid<V>) -> bool {
        self.col < g.width && self.row < g.height
    }

    /// Computes the position of an adjacent position to `self` in direction `dir`. If the new position would be out of
    /// bounds of `g`, returns None. Otherwise, returns `Some(new_position)`.
    pub fn step_in_bounds<V>(self, g: &Grid<V>, dir: Direction) -> Option<Self> {
        match dir {
            Direction::Up => {
                if self.row == 0 {
                    None
                } else {
                    Some(Position {
                        row: self.row - 1,
                        ..self
                    })
                }
            }
            Direction::Down => {
                if self.row == g.height - 1 {
                    None
                } else {
                    Some(Position {
                        row: self.row + 1,
                        ..self
                    })
                }
            }
            Direction::Left => {
                if self.col == 0 {
                    None
                } else {
                    Some(Position {
                        col: self.col - 1,
                        ..self
                    })
                }
            }
            Direction::Right => {
                if self.col == g.width - 1 {
                    None
                } else {
                    Some(Position {
                        col: self.col + 1,
                        ..self
                    })
                }
            }
        }
    }
}

impl<V> Index<Position> for Grid<V> {
    type Output = V;

    fn index(&self, index: Position) -> &Self::Output {
        &self.data[index.row * self.width + index.col]
    }
}

impl<V> IndexMut<Position> for Grid<V> {
    fn index_mut(&mut self, index: Position) -> &mut Self::Output {
        &mut self.data[index.row * self.width + index.col]
    }
}

impl<V: Display> Display for Grid<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.rows_iter() {
            for col in self.cols_iter() {
                write!(f, "{}", self[Position { row, col }])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl<V> Grid<V> {
    /// Computes a new grid based on a partial representation of its rows. The width of the resulting grid will be the maximum length of elements in `lines`.
    /// Its height will be the length of `lines`. Any line which does not have this maximum length will be filled with clones of `empty` to match its size.
    pub fn new_partial_lines(lines: Vec<Vec<V>>, empty: V) -> Option<Self>
    where
        V: Clone,
    {
        let mut lines = lines;
        let height = lines.len();
        if height == 0 {
            return None;
        }

        let width = {
            let mut width = lines[0].len();
            for line in &lines {
                if width < line.len() {
                    width = line.len();
                }
            }
            width
        };

        if width == 0 {
            return None;
        }

        // Fill short lines
        for line in &mut lines {
            for _ in line.len()..width {
                line.push(empty.clone());
            }
        }

        let data: Vec<V> = lines.into_iter().flat_map(|l| l.into_iter()).collect();

        Some(Grid {
            data,
            width,
            height,
        })
    }

    /// Produces a new board based on a list of rows. `lines` must have `height` elements, and each element of `lines` must have `width` elements. Otherwise, returns None.
    ///
    /// Both `width` and `height` must be non-zero, otherwise returns None.
    pub fn new_full_lines(lines: Vec<Vec<V>>, width: usize, height: usize) -> Option<Self> {
        if width == 0 || height == 0 {
            return None;
        }
        if lines.len() == height {
            if lines.iter().all(move |l| l.len() == width) {
                return Some(Grid {
                    data: lines.into_iter().flat_map(Vec::into_iter).collect(),
                    width,
                    height,
                });
            }
        }
        None
    }

    /// Counts the total number of elements in `self` that match the predicate `p`.
    pub fn count(&self, p: impl Fn(&V) -> bool) -> usize {
        self.data.iter().filter(|v| p(*v)).count()
    }

    /// Iterates through all indices of rows within bounds of `self`.
    pub fn rows_iter(&self) -> impl Iterator<Item = usize> {
        0..self.height
    }

    /// Iterates through all indeces of columns within bounds of `self.`
    pub fn cols_iter(&self) -> impl Iterator<Item = usize> {
        0..self.width
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    pub fn get_rows(&self) -> usize {
        self.height
    }

    pub fn get_cols(&self) -> usize {
        self.width
    }

    /// Produces a new grid, whose elements are those of `self` after applying the function `f`.
    pub fn map<W>(&self, f: impl Fn(&V) -> W) -> Grid<W> {
        Grid {
            data: self.data.iter().map(f).collect(),
            width: self.width,
            height: self.height,
        }
    }
}
