use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

#[derive(Clone, Debug)]
pub struct Grid<V> {
    data: Vec<V>,
    width: usize,
    height: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Position {
    pub fn new(row: usize, col: usize) -> Self {
        Position { row, col }
    }

    pub fn iter_row_major(width: usize, height: usize) -> impl Iterator<Item = Position> {
        (0..height).flat_map(move |row| (0..width).map(move |col| Position { row, col }))
    }

    pub fn is_in_bounds_of<V>(self, g: &Grid<V>) -> bool {
        self.col < g.width && self.row < g.height
    }

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

    pub fn count(&self, p: impl Fn(&V) -> bool) -> usize {
        self.data.iter().filter(|v| p(*v)).count()
    }

    pub fn rows_iter(&self) -> impl Iterator<Item = usize> {
        0..self.height
    }

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

    pub fn map<W>(&self, f: impl Fn(&V) -> W) -> Grid<W> {
        Grid {
            data: self.data.iter().map(f).collect(),
            width: self.width,
            height: self.height,
        }
    }
}
