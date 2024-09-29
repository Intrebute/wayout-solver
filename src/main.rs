use std::io::Error;

use board::BoardAssignment;
use equations::Equations;
use grid::Position;

pub mod bit;
pub mod board;
pub mod equations;
pub mod grid;
pub mod matrix;

fn main() {
    //do_it_all("00000\n00000\n00000\n00000\n00000");
    println!("Welcome to the Lights-Out solver!");
    println!();
    println!("Enter your board line by line, using 0 for an unlit cell, 1 for a lit cell, and space for a cell without a light. When finished, type 'done' on its own line.");

    let board_text = match read_board_text() {
        Ok(board_text) => board_text,
        Err(e) => {
            println!("Error reading line: {}", e);
            return;
        }
    };

    let board = {
        let mut board = match board::parse_board(&board_text) {
            Ok(board) => board.1,
            Err(e) => {
                println!(
                    "Error parsing board with text {}.\nError: {}",
                    board_text, e
                );
                return;
            }
        };

        println!("Does board contain modifiers? [yes/no]");

        let has_modifiers = match read_yes_no() {
            Ok(has_modifiers) => has_modifiers,
            Err(e) => {
                println!("Error reading yes/no answer: {}", e);
                return;
            }
        };

        if has_modifiers {
            println!("Enter modifiers as a grid of spaces, \"H\"s, and \"V\"s.");
            for (row, line) in (0..board.height()).zip(std::io::stdin().lines()) {
                match line {
                    Ok(line) => {
                        for (col, ch) in (0..board.width()).zip(line.chars()) {
                            if ch == ' ' {
                                continue;
                            } else if ch == 'H' {
                                match &mut board[Position { row, col }] {
                                    Some(cell) => {
                                        cell.affects_up = false;
                                        cell.affects_down = false;
                                    }
                                    None => {
                                        println!("Modifier applied to empty cell!");
                                        return;
                                    }
                                }
                            } else if ch == 'V' {
                                match &mut board[Position { row, col }] {
                                    Some(cell) => {
                                        cell.affects_left = false;
                                        cell.affects_right = false;
                                    }
                                    None => {
                                        println!("Modifier applied to empty cell!");
                                        return;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("Could not read line: {}", e);
                        return;
                    }
                }
            }
        }

        board
    };

    let (mut matrix, indexed_locations) = board.to_matrix();
    println!("Computed board matrix:\n{}", matrix);
    matrix.eliminate();
    println!("Reduced board matrix:\n{}", matrix);

    let eqns = Equations::new(matrix);
    let results = eqns.enumerate_all_results();
    if results.len() != 0 {
        println!("Board has {} solutions.", results.len());
    } else {
        println!("Board has no solutions.");
        return;
    }
    println!("Show all solutions, or only one with minimum presses? (all/min)");

    let show_all: bool = {
        let mut show_all = None;
        for line in std::io::stdin().lines() {
            match line {
                Ok(line) => {
                    if line == "all" {
                        show_all = Some(true);
                        break;
                    } else if line == "min" {
                        show_all = Some(false);
                        break;
                    } else {
                        println!("Please input either \"all\" or \"min\"");
                        continue;
                    }
                }
                Err(e) => {
                    println!("Error reading line: {}", e);
                    return;
                }
            }
        }
        match show_all {
            Some(show_all) => show_all,
            None => {
                println!("Error reading \"all\" or \"min\" value. Defaulting to \"min\".");
                false
            }
        }
    };

    if show_all {
        for (count, assignment) in results.into_iter().enumerate() {
            println!(
                "Solution #{}:\n{}",
                count + 1,
                board.assign_assignment(assignment, &indexed_locations)
            );
        }
    } else {
        let mut min_moves_board_count: Option<(usize, BoardAssignment, usize)> = None;
        for (count, assignment) in results.into_iter().enumerate() {
            let assigned_board = board.assign_assignment(assignment, &indexed_locations);
            match &min_moves_board_count {
                Some((min_moves, _, _)) => {
                    if assigned_board.count_ones() < *min_moves {
                        min_moves_board_count =
                            Some((assigned_board.count_ones(), assigned_board, count));
                    }
                }
                None => {
                    min_moves_board_count =
                        Some((assigned_board.count_ones(), assigned_board, count));
                }
            }
        }
        match min_moves_board_count {
            Some((moves, board, count)) => {
                println!("Solution #{}, {} button presses:\n{}", count, moves, board);
            }
            None => {
                println!("Could not find a solution despite there being solutions. Please send board configuration to developer for debugging.");
                return;
            }
        }
    }

    //do_it_all(&"1010\n0110\n11\n00 00\n   1 0\n   010");
    /*let base_matrix = Matrix::new_from_bytes(&[
        &[1, 1, 0, 0, 0, 0, 1],
        &[1, 1, 1, 1, 0, 0, 0],
        &[0, 1, 1, 0, 1, 0, 1],
        &[0, 1, 0, 1, 1, 0, 0],
        &[0, 0, 1, 1, 1, 1, 0],
        &[0, 0, 0, 0, 1, 1, 1],
    ])
    .unwrap();
    let mut matrix = base_matrix.clone();
    matrix.report_elimination();
    let eqns = Equations::new(matrix.clone());
    println!("{}", eqns);
    for res in eqns.enumerate_all_results() {
        println!("{}", res.as_bitstring().unwrap());
    }
    let board = board::parse_board(&"0\n10\n110").unwrap().1;
    println!("{}", board);
    let board_matrix = board.to_matrix();
    println!("{}\n{}", board_matrix.0, base_matrix);*/
}

fn read_board_text() -> Result<String, Error> {
    let mut board_text = String::new();

    for line in std::io::stdin().lines() {
        let line = line?;
        if line == "done" {
            break;
        }
        if is_valid_line(&line) {
            board_text.push_str(&line);
            board_text.push('\n');
        } else {
            println!("Invalid line. Please only enter 0's 1's and spaces.");
            continue;
        }
    }

    board_text.pop();

    Ok(board_text)
}

fn read_single_line() -> Result<String, Error> {
    match std::io::stdin().lines().next() {
        Some(line) => {
            return line;
        }
        None => {
            println!("Could not read line.");
            panic!();
        }
    }
}

fn read_yes_no() -> Result<bool, Error> {
    loop {
        let line = read_single_line()?;
        if line == "yes" {
            return Ok(true);
        } else if line == "no" {
            return Ok(false);
        } else {
            println!("Please input either \"yes\" or \"no\". [yes/no]");
        }
    }
}

pub fn is_valid_line(src: &str) -> bool {
    for c in src.chars() {
        if c == '0' || c == '1' || c == ' ' {
            continue;
        } else {
            return false;
        }
    }
    return true;
}

pub fn do_it_all(src: &str) {
    let board = board::parse_board(src)
        .expect("Could not parse board from input.")
        .1;
    println!("Input: \n{}", board);
    let (mut matrix, indexed_locations) = board.to_matrix();
    println!("Indexed Locations:\n{:?}", &indexed_locations);
    println!("{}", &matrix);
    matrix.sort_rows_by_leading_column();
    println!("Sorted rows:\n{}", &matrix);
    matrix.eliminate();
    println!("{}", &matrix);
    let eqns = Equations::new(matrix);
    println!("Equations:\n{}", &eqns);
    let results = eqns.enumerate_all_results();
    println!("Found {} solutions:", results.len());
    for assignment in results {
        println!(
            "Assignment:\n{}\nResult:\n{}",
            assignment.clone(),
            board.assign_assignment(assignment, &indexed_locations)
        );
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eliminate_works() {
        let mut matrix = Matrix::new([
            [1, 1, 0, 0, 0, 0],
            [1, 1, 1, 1, 0, 0],
            [0, 1, 1, 0, 1, 0],
            [0, 1, 0, 1, 1, 0],
            [0, 0, 1, 1, 1, 1],
            [0, 0, 0, 0, 1, 1],
        ]);
        matrix.eliminate();
    }
}
*/
