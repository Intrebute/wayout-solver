use std::fmt::Display;

use crate::bit::Bit;

/// Encodes a matrix as a vector of rows.
///
/// We do not use Grid as the backing structure as we have many matrix operations that are most effectively
/// expressed in terms of row operations, which are easier to perform, on a vector of rows, rather than a monolithic row-major vector.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<Vec<Bit>>,
}

/// Encodes a position within a matrix.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BoundedPosition {
    width: usize,
    height: usize,
    row: usize,
    col: usize,
}

impl BoundedPosition {
    /// Steps to the right, unless at the edge. Returns true if the operation was successful, false if it would lead to an out of bounds position.
    pub fn step_right(&mut self) -> bool {
        if self.col + 1 == self.width {
            return false;
        } else {
            self.col += 1;
            return true;
        }
    }

    /// Steps downwards, unless at the edge. Returns true if the operation was successful, false if it would lead to an out of bounds position.
    pub fn step_down(&mut self) -> bool {
        if self.row + 1 == self.height {
            return false;
        } else {
            self.row += 1;
            return true;
        }
    }
}

/// Finds the next active bit in `row`, starting at index `since`.
pub fn first_active_column_since(row: &[Bit], since: usize) -> Option<usize> {
    for i in since..row.len() {
        if row[i] == Bit::On {
            return Some(i);
        }
    }
    return None;
}

/// Finds the first active bit in `row`.
pub fn get_leading_column(row: &[Bit]) -> Option<usize> {
    first_active_column_since(row, 0)
}

impl Display for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..self.rows {
            for col in 0..self.cols {
                write!(f, "{}", self.data[row][col])?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl Matrix {
    /// Creates a new matrix from a vec of rows. `data` must have exactly `rows` entries, and each row in `data` must have `cols` elements.
    /// If not, returns None. Furthermore, `rows` and `cols` must both be non-zero, otherwise returns `None`.
    pub fn new(data: Vec<Vec<Bit>>, rows: usize, cols: usize) -> Option<Self> {
        if rows == 0 || cols == 0 {
            return None;
        }
        if data.len() != rows {
            return None;
        }
        for row in &data {
            if row.len() != cols {
                return None;
            }
        }

        Some(Matrix { rows, cols, data })
    }

    /// Attempts to augment self with a new column. `col` must have as many elements as `self` has rows. Returns true if successful,
    /// false otherwise. If false is returned, self remains untouched.
    pub fn augment_column(&mut self, col: &[Bit]) -> bool {
        if col.len() != self.rows {
            return false;
        }
        for row in 0..self.rows {
            self.data[row].push(col[row]);
        }
        self.cols += 1;
        return true;
    }

    /// Returns the value at position `pos`. Panics if out of bounds of `self`.
    pub fn get_at(&self, pos: BoundedPosition) -> Bit {
        self.data[pos.row][pos.col]
    }

    /// Sorts rows based on how deep its leading column is. A row with a leftmost leading column is considered earlier in the ordering.
    /// Further active bits in the column do not affect the order.
    pub fn sort_rows_by_leading_column(&mut self) {
        self.data
            .sort_unstable_by_key(|row| match get_leading_column(row) {
                Some(col) => col,
                None => self.cols,
            });
    }

    /// Swaps two rows at indices `row1` and `row2`. If both indices are the same, nothing happens.
    pub fn swap_rows(&mut self, row1: usize, row2: usize) {
        self.data.swap(row1, row2);
    }

    /// Adds the rows at indices `source_row` and `target_row`, storing the result in `target_row`.
    pub fn elementary_add_row_to(&mut self, source_row: usize, target_row: usize) {
        let row = self.data[source_row].clone();
        for col in 0..self.cols {
            self.data[target_row][col] += row[col];
        }
    }

    /// Adds `source_row` onto every row whose `column` bit is `On`. Does not affect row at `source_row` itself.
    /// Effectively eliminates every `On` bit in the entire column at index `column`.
    pub fn decimate_column_with_row(&mut self, source_row: usize, column: usize) {
        for row in 0..self.rows {
            if row == source_row {
                continue;
            }
            if self.data[row][column] == Bit::On {
                self.elementary_add_row_to(source_row, row);
            }
        }
    }

    /// Performs Gauss-Jordan elimination on `self` over the field of bits. Once complete, `self` will be in reduced row-echelon form.
    pub fn eliminate(&mut self) {
        self.sort_rows_by_leading_column();

        let mut current_cell = BoundedPosition {
            width: self.cols,
            height: self.rows,
            row: 0,
            col: 0,
        };

        loop {
            match self.get_at(current_cell) {
                Bit::Off => {
                    if !current_cell.step_right() {
                        // We've reached the outside of the grid. We quit here.
                        break;
                    } else {
                        // We are still within the grid, pointing at a new cell. We loop again.
                        continue;
                    }
                }
                Bit::On => {
                    //We found a leading column.
                    self.decimate_column_with_row(current_cell.row, current_cell.col);
                    // At this point, every row has a 0 at the current column. Now we sort rows again.
                    self.sort_rows_by_leading_column();
                    // All the rows with proper leading bits should still be above the current row,
                    // which means the row we were decimating with should still be the current row.
                    // Now, we step down.
                    if !current_cell.step_down() {
                        // We've reached the outside of the grid. We quit here.
                        break;
                    } else {
                        // We are still within the grid, pointing at a new cell. We loop again.
                        // Something to consider is to assert the new position necessarily holds
                        // an off bit, as a sanity check.
                        //assert_eq!(self.get_at(current_cell), Bit::Off);
                        continue;
                    }
                }
            }
        }
    }

    /// Returns whether or not the entire row at index `row` is `Off`.
    pub fn is_row_zero(&self, row: usize) -> bool {
        for b in &self.data[row] {
            if *b == Bit::On {
                return false;
            }
        }
        return true;
    }

    /// Counts the number of non-zero rows. This is equal to the rank of the matrix
    /// if the matrix is in reduced row echelon form.
    pub fn non_zero_row_count(&self) -> usize {
        fn is_zero(row: &[Bit]) -> bool {
            for bit in row {
                if *bit == Bit::On {
                    return false;
                }
            }
            return true;
        }
        for row in 0..self.rows {
            if is_zero(&self.data[row]) {
                return row;
            }
        }
        return self.rows;
    }

    /// Collects the column indices which do not contain a leading 1.
    /// These correspond to the free parameters of the system.
    pub fn non_leading_columns(&self) -> Vec<usize> {
        let mut current_cell = BoundedPosition {
            width: self.cols,
            height: self.rows,
            row: 0,
            col: 0,
        };

        let mut free_cols = vec![];

        loop {
            match self.get_at(current_cell) {
                Bit::Off => {
                    free_cols.push(current_cell.col);
                    if !current_cell.step_right() {
                        break;
                    } else {
                        continue;
                    }
                }
                Bit::On => {
                    if !current_cell.step_down() {
                        break;
                    } else {
                        if !current_cell.step_right() {
                            break;
                        } else {
                            continue;
                        }
                    }
                }
            }
        }

        return free_cols;
    }

    /// Debug function. Reports effects of calling `eliminate`.
    pub fn report_elimination(&mut self) {
        println!("Input:\n{}", self);
        self.eliminate();
        println!("Output:\n{}\nRank: {}", self, self.non_zero_row_count());
        let non_l_cols: Vec<usize> = self.non_leading_columns().into_iter().collect();
        println!("Free columns:");
        self.display_selected_columns(&non_l_cols);
    }

    /// Debug function. Prints matrix, but only the columns indexed in `col_nums`. Other entries are displayed as a `.`.
    pub fn display_selected_columns(&self, col_nums: &[usize]) {
        for row in 0..self.rows {
            for col in 0..self.cols {
                if col_nums.contains(&col) {
                    print!("{}", self.data[row][col]);
                } else {
                    print!(".");
                }
            }
            println!();
        }
    }
}
