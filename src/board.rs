use std::{collections::HashMap, fmt::Display};

use nom::{
    branch::alt,
    character::complete::{newline, one_of},
    combinator::{eof, map},
    multi::{many1, separated_list1},
    sequence::tuple,
    IResult,
};

use crate::{bit::Bit, equations::Assignment, matrix::Matrix};

#[derive(Clone, Debug)]
pub struct Board {
    data: Vec<Vec<Option<Bit>>>,
    width: usize,
    height: usize,
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "W: {} H: {} C: {}",
            self.width,
            self.height,
            self.count_ones()
        )?;
        for row in &self.data {
            for cell in row {
                match cell {
                    Some(b) => write!(f, "{}", b)?,
                    None => write!(f, " ")?,
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Board {
    pub fn to_matrix(&self) -> (Matrix, HashMap<usize, (usize, usize)>) {
        let mut indexed_locations: HashMap<usize, (usize, usize)> = HashMap::new();
        let mut indexed_values: HashMap<usize, Bit> = HashMap::new();
        let mut index_of: HashMap<(usize, usize), usize> = HashMap::new();
        let mut count: usize = 0;
        for row in 0..self.height {
            for col in 0..self.width {
                if let Some(b) = &self.data[row][col] {
                    indexed_locations.insert(count, (row, col));
                    indexed_values.insert(count, *b);
                    index_of.insert((row, col), count);
                    count += 1;
                }
            }
        }

        let mut matrix_data: Vec<Vec<Bit>> = vec![vec![Bit::Off; count]; count];

        for var in 0..count {
            matrix_data[var][var] = Bit::On;
            let this_pos = indexed_locations[&var];
            if var == 3 {
                println!("Problematic case!");
                println!("this_pos: {this_pos:?}");
            }

            for dir in [
                Direction::Up,
                Direction::Down,
                Direction::Left,
                Direction::Right,
            ] {
                if let Some(adjacent_pos) = self.step(this_pos, dir) {
                    if let Some(adjacent_index) = index_of.get(&adjacent_pos) {
                        matrix_data[var][*adjacent_index] = Bit::On;
                    }
                }
            }
        }

        let mut almost_matrix =
            Matrix::new(matrix_data, count, count).expect("Could not form matrix");

        let constant_row = (0..count)
            .map(|i| indexed_values[&i])
            .map(|b| b + Bit::On)
            // We add On to everything as the constant row is equal to the current state,
            // plus the target state, which is all Ons.
            .collect::<Vec<Bit>>();

        almost_matrix.augment_column(&constant_row);

        (almost_matrix, indexed_locations)
    }

    pub fn count_ones(&self) -> usize {
        let mut count = 0usize;
        for row in &self.data {
            for mb in row {
                if *mb == Some(Bit::On) {
                    count += 1;
                }
            }
        }
        count
    }

    pub fn assign_assignment(
        &self,
        assignment: Assignment,
        indexed_locations: &HashMap<usize, (usize, usize)>,
    ) -> Self {
        let mut new_board = self.clone();
        for (i, loc) in indexed_locations {
            new_board.data[loc.0][loc.1] = Some(assignment.0[i]);
        }
        new_board
    }

    pub fn step(&self, (row, col): (usize, usize), dir: Direction) -> Option<(usize, usize)> {
        match dir {
            Direction::Up => {
                if row == 0 {
                    None
                } else {
                    Some((row - 1, col))
                }
            }
            Direction::Down => {
                if row == self.height - 1 {
                    None
                } else {
                    Some((row + 1, col))
                }
            }
            Direction::Left => {
                if col == 0 {
                    None
                } else {
                    Some((row, col - 1))
                }
            }
            Direction::Right => {
                if col == self.width - 1 {
                    None
                } else {
                    Some((row, col + 1))
                }
            }
        }
    }
}

pub fn parse_cell(input: &str) -> IResult<&str, Option<Bit>> {
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

pub fn parse_line(input: &str) -> IResult<&str, Vec<Option<Bit>>> {
    let (input, bits) = many1(parse_cell)(input)?;
    Ok((input, bits))
}

pub fn parse_board(input: &str) -> IResult<&str, Board> {
    let (input, (mut lines, _)) = tuple((
        separated_list1(newline, parse_line),
        alt((map(eof, |_| ()), map(tuple((newline, eof)), |_| ()))),
    ))(input)?;
    let width = {
        let mut max_width = 0;
        for line in &lines {
            if max_width < line.len() {
                max_width = line.len();
            }
        }
        max_width
    };
    let height = lines.len();
    for line in &mut lines {
        if line.len() < width {
            for _ in line.len()..width {
                line.push(None);
            }
        }
    }
    Ok((
        input,
        Board {
            data: lines,
            width,
            height,
        },
    ))
}
