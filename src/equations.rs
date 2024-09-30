use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use crate::{
    bit::Bit,
    matrix::{get_leading_column, Matrix},
};

type Var = usize;

/// Encodes a system of equations as a set of free variables, along with a set of equations encoding each non-free variable as a sum of
/// free variables and a constant term.
#[derive(Clone, Debug)]
pub struct Equations {
    free_vars: HashSet<Var>,
    eqns: HashMap<Var, (HashSet<Var>, Bit)>,
}

/// Encodes an assignment of values to variables, as a map from variable indices to concrete bits.
#[derive(Clone, Debug)]
pub struct Assignment(pub HashMap<Var, Bit>);

impl Display for Assignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sorted = {
            let mut sorted = self
                .0
                .iter()
                .map(|(&v, &b)| (v, b))
                .collect::<Vec<(Var, Bit)>>();
            sorted.sort_by_key(|(v, _)| *v);
            sorted
        };

        for (var, bit) in sorted.into_iter() {
            write!(f, "{}:{}, ", var, bit)?;
        }
        Ok(())
    }
}

impl Assignment {
    /// Renders an assignment as a string of `1`s and `0`s, where the `i`th character corresponds to the variable at index `i`.
    pub fn as_bitstring(&self) -> Option<String> {
        let mut res = String::new();
        for i in 0..self.0.len() {
            if let Some(&b) = self.0.get(&i) {
                res.push(match b {
                    Bit::Off => '0',
                    Bit::On => '1',
                });
            } else {
                return None;
            }
        }
        Some(res)
    }
}

impl Display for Equations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Free vars: ")?;
        let sorted_free_vars = {
            let mut sorted_free_vars = self.free_vars.iter().map(|&v| v).collect::<Vec<usize>>();
            sorted_free_vars.sort();
            sorted_free_vars
        };
        for free_var in sorted_free_vars.into_iter() {
            write!(f, "x_{}, ", free_var)?;
        }
        let sorted_equations = {
            let mut sorted_equations = self
                .eqns
                .iter()
                .map(|(&v, p)| (v, &p.0, p.1))
                .collect::<Vec<_>>();
            sorted_equations.sort_by_key(|(v, _, _)| *v);
            sorted_equations
        };
        writeln!(f, "Equations:")?;
        for (var, terms, constant_term) in sorted_equations.into_iter() {
            write!(f, "x_{} = ", var)?;
            let sorted_terms = {
                let mut sorted_terms = terms.iter().map(|&b| b).collect::<Vec<usize>>();
                sorted_terms.sort();
                sorted_terms
            };
            write!(f, "{}", constant_term)?;
            for term in sorted_terms.into_iter() {
                write!(f, " + x_{}", term)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl Equations {
    /// Computes a system of equations from `matrix`` in reduced row-echelon form. Does not check if `matrix`` is in reduced row-echelon form.
    /// Will produce a system of equations of questionable quality otherwise.
    pub fn new(matrix: Matrix) -> Self {
        let free_vars = {
            let mut free_vars: HashSet<Var> = matrix.non_leading_columns().into_iter().collect();
            free_vars.remove(&(matrix.cols - 1));
            free_vars
        };
        let mut eqns = HashMap::new();
        for row in 0..matrix.rows {
            if matrix.is_row_zero(row) {
                break;
            }
            let params = free_vars
                .iter()
                .filter_map(|&col| {
                    if matrix.data[row][col] == Bit::On {
                        Some(col)
                    } else {
                        None
                    }
                })
                .collect::<HashSet<Var>>();
            let constant_term = matrix.data[row][matrix.cols - 1];
            let leading_col = get_leading_column(&matrix.data[row]).unwrap();
            if leading_col != matrix.cols - 1 {
                eqns.insert(
                    get_leading_column(&matrix.data[row]).unwrap(), // Safe to unwrap as we bailed before if row is zero
                    (params, constant_term),
                );
            }
        }

        Equations { free_vars, eqns }
    }

    /// Uses a partial `assignment` of only free variables in `self`, and the equations of `self`, to compute a full assignment of all variables in `self`.
    /// Does not check that `assignment` does in fact only assign values to free variables in `self`.
    pub fn backfeed(&self, assignment: Assignment) -> Assignment {
        let Assignment(valuation) = assignment;
        let mut results = HashMap::new();

        for (var, terms) in self.eqns.iter() {
            let mut value = Bit::Off;
            for term in terms.0.iter() {
                value += *valuation.get(term).unwrap();
            }
            value += terms.1;
            results.insert(*var, value);
        }

        for (var, value) in valuation {
            results.insert(var, value);
        }

        Assignment(results)
    }

    /// Enumerates the full assignment of all possible partial assignments in the free variables in `self`.
    pub fn enumerate_all_results(&self) -> Vec<Assignment> {
        let assignments = enumerate_all_assignments(&self.free_vars);
        if assignments.len() > 0 {
            let assignments = assignments.into_iter().map(|a| self.backfeed(a)).collect();
            assignments
        } else {
            let assignments = vec![self.backfeed(Assignment(HashMap::new()))];
            assignments
        }
    }
}

/// Produces all possible assignments of values for the variables in `vars`.
pub fn enumerate_all_assignments(vars: &HashSet<Var>) -> Vec<Assignment> {
    let mut assignments = Vec::new();
    println!("Vars: {:?}", vars);
    let sorted_vars = {
        let mut sorted_vars = vars.iter().cloned().collect::<Vec<usize>>();
        sorted_vars.sort();
        sorted_vars
    };
    let remaining_vars = {
        let mut remaining_vars = sorted_vars.clone();
        if let Some(&arbitrary) = remaining_vars.first() {
            assignments.push(Assignment(HashMap::from([(arbitrary, Bit::Off)])));
            assignments.push(Assignment(HashMap::from([(arbitrary, Bit::On)])));
            remaining_vars.remove(0);
        } else {
            return assignments;
        }
        let mut remaining_vars = remaining_vars.into_iter().collect::<Vec<usize>>();
        remaining_vars.sort();
        remaining_vars
    };
    for var in remaining_vars {
        assignments = assignments
            .into_iter()
            .flat_map(|Assignment(a)| {
                let mut a1 = a.clone();
                let mut a2 = a;
                a1.insert(var, Bit::Off);
                a2.insert(var, Bit::On);
                [Assignment(a1), Assignment(a2)].into_iter()
            })
            .collect();
    }
    assignments
}
