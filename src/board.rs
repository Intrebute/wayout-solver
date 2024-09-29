use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Index, IndexMut},
};

use nom::{
    branch::alt,
    character::complete::{newline, one_of},
    combinator::{eof, map},
    multi::{many1, separated_list1},
    sequence::tuple,
    IResult,
};

use crate::{
    bit::Bit,
    equations::Assignment,
    grid::{Direction, Grid, Position},
    matrix::Matrix,
};

/// Describes the initial state of the board, along with any modifiers its cells might have
#[derive(Clone, Debug)]
pub struct BoardDescription {
    grid: Grid<Option<Cell>>,
}

/// Describes an assignment on a board.
pub struct BoardAssignment {
    grid: Grid<Option<Bit>>,
}

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub affects_up: bool,
    pub affects_down: bool,
    pub affects_left: bool,
    pub affects_right: bool,
    pub starting_value: Bit,
}

impl Display for BoardAssignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "W: {} H: {} C: {}",
            self.grid.get_width(),
            self.grid.get_height(),
            self.count_ones(),
        )?;

        for row in self.grid.rows_iter() {
            for col in self.grid.cols_iter() {
                match self.grid[Position::new(row, col)] {
                    Some(b) => write!(f, "{}", b)?,
                    None => write!(f, " ")?,
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "U:{} D:{} L:{} R:{} V:{}",
            self.affects_up,
            self.affects_down,
            self.affects_left,
            self.affects_right,
            self.starting_value
        )
    }
}

impl Index<Direction> for Cell {
    type Output = bool;

    fn index(&self, index: Direction) -> &Self::Output {
        match index {
            Direction::Up => &self.affects_up,
            Direction::Down => &self.affects_down,
            Direction::Left => &self.affects_left,
            Direction::Right => &self.affects_right,
        }
    }
}

impl Cell {
    pub fn new(
        affects_up: bool,
        affects_down: bool,
        affects_left: bool,
        affects_right: bool,
        starting_value: Bit,
    ) -> Self {
        Cell {
            affects_up,
            affects_down,
            affects_left,
            affects_right,
            starting_value,
        }
    }
    pub fn new_basic(b: Bit) -> Self {
        Self::new(true, true, true, true, b)
    }
}

impl Display for BoardDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "W: {} H: {} C: {}",
            self.grid.get_width(),
            self.grid.get_height(),
            self.count_ones()
        )?;
        for row in self.grid.rows_iter() {
            for col in self.grid.cols_iter() {
                match self.grid[Position::new(row, col)] {
                    Some(b) => write!(f, "{}", b.starting_value)?,
                    None => write!(f, " ")?,
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl BoardAssignment {
    pub fn count_ones(&self) -> usize {
        self.grid.count(|oc| oc.is_some_and(|c| c == Bit::On))
    }
}

impl Index<Position> for BoardDescription {
    type Output = Option<Cell>;

    fn index(&self, index: Position) -> &Self::Output {
        &self.grid[index]
    }
}

impl IndexMut<Position> for BoardDescription {
    fn index_mut(&mut self, index: Position) -> &mut Self::Output {
        &mut self.grid[index]
    }
}

impl BoardDescription {
    pub fn to_matrix(&self) -> (Matrix, HashMap<usize, Position>) {
        let mut indexed_locations: HashMap<usize, Position> = HashMap::new();
        let mut indexed_values: HashMap<usize, Cell> = HashMap::new();
        let mut index_of: HashMap<Position, usize> = HashMap::new();
        let mut count: usize = 0;
        for row in self.grid.rows_iter() {
            for col in self.grid.cols_iter() {
                if let Some(b) = &self.grid[Position::new(row, col)] {
                    indexed_locations.insert(count, Position::new(row, col));
                    indexed_values.insert(count, *b);
                    index_of.insert(Position::new(row, col), count);
                    count += 1;
                }
            }
        }

        let mut matrix_data: Vec<Vec<Bit>> = vec![vec![Bit::Off; count]; count];

        for var in 0..count {
            matrix_data[var][var] = Bit::On;
            let this_pos = indexed_locations[&var];

            for dir in [
                Direction::Up,
                Direction::Down,
                Direction::Left,
                Direction::Right,
            ] {
                if let Some(adjacent_pos) = this_pos.step_in_bounds(&self.grid, dir) {
                    if let Some(adjacent_index) = index_of.get(&adjacent_pos) {
                        if let Some(this_cell) = indexed_values.get(&var) {
                            if this_cell[dir] {
                                matrix_data[var][*adjacent_index] = Bit::On;
                            }
                        }
                        //matrix_data[var][*adjacent_index] = Bit::On;
                    }
                }
            }
        }

        let mut almost_matrix =
            Matrix::new(matrix_data, count, count).expect("Could not form matrix");

        let constant_row = (0..count)
            .map(|i| indexed_values[&i])
            .map(|b| b.starting_value + Bit::On)
            // We add On to everything as the constant row is equal to the current state,
            // plus the target state, which is all Ons.
            .collect::<Vec<Bit>>();

        almost_matrix.augment_column(&constant_row);

        (almost_matrix, indexed_locations)
    }

    pub fn count_ones(&self) -> usize {
        self.grid
            .count(|oc| oc.is_some_and(|c| c.starting_value == Bit::On))
    }

    pub fn assign_assignment(
        &self,
        assignment: Assignment,
        indexed_locations: &HashMap<usize, Position>,
    ) -> BoardAssignment {
        let mut new_grid = self.grid.map(|oc| oc.map(|c| c.starting_value));
        for (i, loc) in indexed_locations {
            new_grid[*loc] = Some(assignment.0[i]);
        }
        BoardAssignment { grid: new_grid }
    }

    pub fn height(&self) -> usize {
        self.grid.get_height()
    }

    pub fn width(&self) -> usize {
        self.grid.get_width()
    }
}

pub fn parse_bit(input: &str) -> IResult<&str, Option<Bit>> {
    let (input, next) = one_of("01 ")(input)?;
    Ok((
        input,
        match next {
            '0' => Some(Bit::Off),
            '1' => Some(Bit::On),
            ' ' => None,
            _ => panic!("Supposedly unreachable!"),
        },
    ))
}

pub fn parse_basic_cell(input: &str) -> IResult<&str, Option<Cell>> {
    let (input, ob) = parse_bit(input)?;
    Ok((input, ob.map(|b| Cell::new_basic(b))))
}

pub fn parse_bit_line(input: &str) -> IResult<&str, Vec<Option<Bit>>> {
    let (input, bits) = many1(parse_bit)(input)?;
    Ok((input, bits))
}

pub fn parse_basic_cell_line(input: &str) -> IResult<&str, Vec<Option<Cell>>> {
    let (input, cells) = many1(parse_basic_cell)(input)?;
    Ok((input, cells))
}

pub fn parse_board(input: &str) -> IResult<&str, BoardDescription> {
    let (input, (lines, _)) = tuple((
        separated_list1(newline, parse_basic_cell_line),
        alt((map(eof, |_| ()), map(tuple((newline, eof)), |_| ()))),
    ))(input)?;
    let grid = Grid::new_partial_lines(lines, None).unwrap(); // unwrap is safe due to the parsers' guarantee of nonemptiness.
    Ok((input, BoardDescription { grid }))
}
