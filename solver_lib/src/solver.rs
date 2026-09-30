use std::collections::{HashMap, HashSet};

use crate::expression::*;
use crate::matrix::{Dimension, Matrix};
use crate::optimizer::*;

/// Extracts linear coefficients for all variables and the constant term from an expression.
fn extract_linear_coefficients(
    expr: &Expression,
    variables: &[Variable],
) -> (HashMap<Variable, f64>, f64) {
    let mut zero_map = HashMap::new();
    for v in variables {
        zero_map.insert(v.clone(), 0.0);
    }

    let const_val = expr.get_root_term().compute(zero_map.clone()).unwrap_or(0.0);

    let mut coeffs = HashMap::new();
    for v in variables {
        let mut unit_map = zero_map.clone();
        unit_map.insert(v.clone(), 1.0);
        let val_with_unit = expr.get_root_term().compute(unit_map).unwrap_or(const_val);
        coeffs.insert(v.clone(), val_with_unit - const_val);
    }

    (coeffs, const_val)
}

/// Solves a linear optimization problem using the Simplex algorithm.
pub fn solve(problem: LinearOptimizationProblem) -> HashMap<Variable, f64> {
    // 1. Collect all distinct variables from objective and constraints
    let all_vars: Vec<Variable> = {
        let mut set = HashSet::new();
        for v in problem.optimizer.get_root_term().get_variables() {
            set.insert(v);
        }
        for c in &problem.constraints {
            for v in c.get_left_hand().get_root_term().get_variables() {
                set.insert(v);
            }
            for v in c.get_right_hand().get_root_term().get_variables() {
                set.insert(v);
            }
        }
        let mut list: Vec<Variable> = set.into_iter().collect();
        list.sort_by(|a, b| a.clone().get_name().cmp(&b.clone().get_name()));
        list
    };

    let num_vars = all_vars.len();
    let num_constraints = problem.constraints.len();

    if num_vars == 0 {
        return HashMap::new();
    }

    // Matrix Dimension:
    // Columns: num_vars (original) + num_constraints (slack variables) + 1 (RHS column)
    // Rows: 1 (objective row 0) + num_constraints (constraint rows 1..=m)
    let dim = Dimension {
        x: num_vars + num_constraints + 1,
        y: num_constraints + 1,
    };

    let mut tableau = Matrix::new(dim);

    // Fill objective row (Row 0)
    let (obj_coeffs, _) = extract_linear_coefficients(&problem.optimizer, &all_vars);
    for (j, var) in all_vars.iter().enumerate() {
        let coeff = obj_coeffs.get(var).copied().unwrap_or(0.0);
        let val = match problem.mechanism {
            OptimizationMechanism::MAX => coeff,
            OptimizationMechanism::MIN => -coeff,
        };
        tableau.set_cell(j, 0, val);
    }

    // Fill constraint rows (Rows 1..=num_constraints) and add slack variables
    for (i, constraint) in problem.constraints.iter().enumerate() {
        let row_idx = i + 1;
        let (lhs_coeffs, lhs_const) =
            extract_linear_coefficients(constraint.get_left_hand(), &all_vars);
        let (rhs_coeffs, rhs_const) =
            extract_linear_coefficients(constraint.get_right_hand(), &all_vars);

        // Standard form: LHS - RHS <= 0 => sum (lhs_c - rhs_c)*x <= rhs_const - lhs_const
        let mut rhs_val = rhs_const - lhs_const;

        for (j, var) in all_vars.iter().enumerate() {
            let a_lhs = lhs_coeffs.get(var).copied().unwrap_or(0.0);
            let a_rhs = rhs_coeffs.get(var).copied().unwrap_or(0.0);
            tableau.set_cell(j, row_idx, a_lhs - a_rhs);
        }

        // Add slack variable (+1.0 in slack column for this constraint)
        let slack_col = num_vars + i;
        tableau.set_cell(slack_col, row_idx, 1.0);

        // Handle negative RHS by inverting the row
        if rhs_val < 0.0 {
            rhs_val = -rhs_val;
            for col in 0..dim.x {
                if let Some(c) = tableau.get_cell(col, row_idx) {
                    tableau.set_cell(col, row_idx, -c);
                }
            }
        }

        // Set RHS column
        let rhs_col = dim.x - 1;
        tableau.set_cell(rhs_col, row_idx, rhs_val);
    }

    // Check if the constraint columns are in row echelon form / canonical basis
    let _ = tableau.is_row_echolon_form();

    // 3. Simplex Pivoting Loop
    let total_cols = dim.x - 1; // exclude RHS
    let max_iterations = 1000;
    let mut iterations = 0;

    while iterations < max_iterations {
        iterations += 1;

        // Step 3: Pivot on the largest cell value in the first row (row 0)
        let mut max_val = 0.0;
        let mut pivot_col: Option<usize> = None;

        for col in 0..total_cols {
            if let Some(val) = tableau.get_cell(col, 0) {
                if val > max_val && val > 1e-9 {
                    max_val = val;
                    pivot_col = Some(col);
                }
            }
        }

        let pivot_col = match pivot_col {
            Some(col) => col,
            None => break, // Optimal solution reached (no positive coefficients in row 0)
        };

        // Minimum Ratio Test to find the leaving row
        let mut min_ratio = f64::INFINITY;
        let mut pivot_row: Option<usize> = None;
        let rhs_col = dim.x - 1;

        for row in 1..dim.y {
            let a_rc = tableau.get_cell(pivot_col, row).unwrap_or(0.0);
            if a_rc > 1e-9 {
                let rhs = tableau.get_cell(rhs_col, row).unwrap_or(0.0);
                let ratio = rhs / a_rc;
                if ratio < min_ratio {
                    min_ratio = ratio;
                    pivot_row = Some(row);
                }
            }
        }

        let pivot_row = match pivot_row {
            Some(row) => row,
            None => break, // Unbounded solution
        };

        // Pivot on (pivot_col, pivot_row)
        let pivot_element = tableau.get_cell(pivot_col, pivot_row).unwrap();

        // 1. Normalize pivot row
        for col in 0..dim.x {
            let current = tableau.get_cell(col, pivot_row).unwrap();
            tableau.set_cell(col, pivot_row, current / pivot_element);
        }

        // 2. Eliminate pivot column from all other rows
        for row in 0..dim.y {
            if row != pivot_row {
                let factor = tableau.get_cell(pivot_col, row).unwrap();
                if factor.abs() > 1e-12 {
                    for col in 0..dim.x {
                        let pivot_row_val = tableau.get_cell(col, pivot_row).unwrap();
                        let current_val = tableau.get_cell(col, row).unwrap();
                        tableau.set_cell(col, row, current_val - factor * pivot_row_val);
                    }
                }
            }
        }
    }

    // Extract basic solution values for original variables
    let mut solution = HashMap::new();
    let rhs_col = dim.x - 1;

    for (j, var) in all_vars.iter().enumerate() {
        let mut one_row: Option<usize> = None;
        let mut is_basic = true;

        for row in 0..dim.y {
            let val = tableau.get_cell(j, row).unwrap_or(0.0);
            if (val - 1.0).abs() < 1e-7 {
                if one_row.is_none() && row > 0 {
                    one_row = Some(row);
                } else {
                    is_basic = false;
                    break;
                }
            } else if val.abs() > 1e-7 {
                is_basic = false;
                break;
            }
        }

        let var_val = if is_basic {
            if let Some(r) = one_row {
                tableau.get_cell(rhs_col, r).unwrap_or(0.0)
            } else {
                0.0
            }
        } else {
            0.0
        };

        solution.insert(var.clone(), var_val);
    }

    solution
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_maximization() {
        // Maximize: 3*x + 2*y
        // Subject to:
        // x + y <= 4
        // x <= 2
        // Optimal solution: x = 2, y = 2 -> Z = 10
        let mut problem = LinearOptimizationProblem::new();

        let x = Variable::new("x");
        let y = Variable::new("y");

        let obj = Expression::new(Box::new(Sum::new(vec![
            Box::new(Product::new(vec![
                Box::new(Constant::new(3.0)),
                Box::new(VariableTerm::new(x.clone())),
            ])),
            Box::new(Product::new(vec![
                Box::new(Constant::new(2.0)),
                Box::new(VariableTerm::new(y.clone())),
            ])),
        ])));

        problem.set_optimizer(obj, OptimizationMechanism::MAX);

        // Constraint 1: x + y <= 4
        let c1_lhs = Expression::new(Box::new(Sum::new(vec![
            Box::new(VariableTerm::new(x.clone())),
            Box::new(VariableTerm::new(y.clone())),
        ])));
        let c1_rhs = Expression::new(Box::new(Constant::new(4.0)));
        problem.add_constraint(LinearOptimizationConstraint::new(
            c1_lhs,
            c1_rhs,
            ConstraintType::LE,
        ));

        // Constraint 2: x <= 2
        let c2_lhs = Expression::new(Box::new(VariableTerm::new(x.clone())));
        let c2_rhs = Expression::new(Box::new(Constant::new(2.0)));
        problem.add_constraint(LinearOptimizationConstraint::new(
            c2_lhs,
            c2_rhs,
            ConstraintType::LE,
        ));

        let solution = solve(problem);

        let x_val = solution.get(&x).copied().unwrap_or(0.0);
        let y_val = solution.get(&y).copied().unwrap_or(0.0);

        assert!((x_val - 2.0).abs() < 1e-4, "Expected x=2, got {}", x_val);
        assert!((y_val - 2.0).abs() < 1e-4, "Expected y=2, got {}", y_val);
    }

    #[test]
    fn test_single_variable_bound() {
        // Maximize: 5*x
        // Subject to: x <= 10
        let mut problem = LinearOptimizationProblem::new();

        let x = Variable::new("x");
        let obj = Expression::new(Box::new(Product::new(vec![
            Box::new(Constant::new(5.0)),
            Box::new(VariableTerm::new(x.clone())),
        ])));
        problem.set_optimizer(obj, OptimizationMechanism::MAX);

        let c_lhs = Expression::new(Box::new(VariableTerm::new(x.clone())));
        let c_rhs = Expression::new(Box::new(Constant::new(10.0)));
        problem.add_constraint(LinearOptimizationConstraint::new(
            c_lhs,
            c_rhs,
            ConstraintType::LE,
        ));

        let solution = solve(problem);
        let x_val = solution.get(&x).copied().unwrap_or(0.0);

        assert!((x_val - 10.0).abs() < 1e-4, "Expected x=10, got {}", x_val);
    }
}
