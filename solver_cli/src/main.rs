use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use solver_lib::expression::{Constant, Expression, Product, Sum, Term, Variable, VariableTerm};
use solver_lib::optimizer::{ConstraintType, LinearOptimizationConstraint, LinearOptimizationProblem, OptimizationMechanism};
use solver_lib::solver::solve;

fn parse_term(raw: &str) -> Result<Box<dyn Term>, String> {
    let s = raw.trim();
    if s.is_empty() {
        return Err("Empty term".to_string());
    }

    // Case 1: Form "coeff * var", e.g. "3*x1", "2.99*x2"
    if let Some((coeff_str, var_str)) = s.split_once('*') {
        let coeff: f64 = coeff_str
            .trim()
            .parse()
            .map_err(|_| format!("Invalid coefficient in term '{}'", s))?;
        let var_name = var_str.trim();
        if var_name.is_empty() {
            return Err(format!("Missing variable name in term '{}'", s));
        }
        return Ok(Box::new(Product::new(vec![
            Box::new(Constant::new(coeff)),
            Box::new(VariableTerm::new(Variable::new(var_name))),
        ])));
    }

    // Case 2: Pure number (Constant)
    if let Ok(val) = s.parse::<f64>() {
        return Ok(Box::new(Constant::new(val)));
    }

    // Case 3: Variable with sign, e.g. "+x1", "-x2", "x1"
    let (sign, rest) = if let Some(stripped) = s.strip_prefix('+') {
        (1.0, stripped.trim())
    } else if let Some(stripped) = s.strip_prefix('-') {
        (-1.0, stripped.trim())
    } else {
        (1.0, s)
    };

    if rest.is_empty() {
        return Err(format!("Invalid term '{}'", s));
    }

    if let Ok(val) = rest.parse::<f64>() {
        return Ok(Box::new(Constant::new(sign * val)));
    }

    let var = Variable::new(rest);
    if sign == 1.0 {
        Ok(Box::new(VariableTerm::new(var)))
    } else {
        Ok(Box::new(Product::new(vec![
            Box::new(Constant::new(-1.0)),
            Box::new(VariableTerm::new(var)),
        ])))
    }
}

pub fn parse_expression(input: &str) -> Result<Expression, String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("Expression cannot be empty".to_string());
    }

    // Normalize '-' so that binary subtraction turns into '+ -'
    let mut normalized = String::new();
    let chars: Vec<char> = s.chars().collect();
    for i in 0..chars.len() {
        let ch = chars[i];
        if ch == '-' {
            if i > 0 && chars[i - 1] != '+' && chars[i - 1] != '*' && chars[i - 1] != '/' && chars[i - 1] != ' ' {
                normalized.push_str(" + -");
            } else {
                normalized.push('-');
            }
        } else {
            normalized.push(ch);
        }
    }

    let parts: Vec<&str> = normalized
        .split('+')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();

    if parts.is_empty() {
        return Err("No terms found in expression".to_string());
    }

    let mut terms: Vec<Box<dyn Term>> = Vec::new();
    for part in parts {
        terms.push(parse_term(part)?);
    }

    if terms.len() == 1 {
        Ok(Expression::new(terms.remove(0)))
    } else {
        Ok(Expression::new(Box::new(Sum::new(terms))))
    }
}

pub fn parse_constraint(input: &str) -> Result<LinearOptimizationConstraint, String> {
    let s = input.trim();
    let (lhs_str, rhs_str, op) = if let Some((l, r)) = s.split_once("<=") {
        (l, r, ConstraintType::LE)
    } else if let Some((l, r)) = s.split_once(">=") {
        (l, r, ConstraintType::GE)
    } else if let Some((l, r)) = s.split_once("==") {
        (l, r, ConstraintType::EQUAL)
    } else if let Some((l, r)) = s.split_once('=') {
        (l, r, ConstraintType::EQUAL)
    } else if let Some((l, r)) = s.split_once('<') {
        (l, r, ConstraintType::LT)
    } else if let Some((l, r)) = s.split_once('>') {
        (l, r, ConstraintType::GT)
    } else {
        return Err(format!(
            "No comparison operator (<=, >=, ==, =, <, >) found in: '{}'",
            s
        ));
    };

    let lhs = parse_expression(lhs_str)?;
    let rhs = parse_expression(rhs_str)?;

    Ok(LinearOptimizationConstraint::new(lhs, rhs, op))
}

fn main() {
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    println!("=== Linear Optimization Solver CLI ===");
    println!("Enter objective function (e.g. 'max 3*x1 + 2.99*x2' or '3*x1 + 2.99*x2'):");
    print!("> ");
    let _ = io::stdout().flush();

    let mut obj_line = String::new();
    if reader.read_line(&mut obj_line).unwrap_or(0) == 0 {
        return;
    }

    let obj_trimmed = obj_line.trim();
    if obj_trimmed.is_empty() {
        println!("Error: Objective function cannot be empty.");
        return;
    }

    let (mechanism, obj_expr_str) = if let Some(stripped) = obj_trimmed.strip_prefix("max ") {
        (OptimizationMechanism::MAX, stripped.trim())
    } else if let Some(stripped) = obj_trimmed.strip_prefix("min ") {
        (OptimizationMechanism::MIN, stripped.trim())
    } else if let Some(stripped) = obj_trimmed.strip_prefix("MAX ") {
        (OptimizationMechanism::MAX, stripped.trim())
    } else if let Some(stripped) = obj_trimmed.strip_prefix("MIN ") {
        (OptimizationMechanism::MIN, stripped.trim())
    } else {
        // Default to MAX if not explicitly prefixed
        (OptimizationMechanism::MAX, obj_trimmed)
    };

    let obj_expr = match parse_expression(obj_expr_str) {
        Ok(expr) => expr,
        Err(e) => {
            eprintln!("Error parsing objective: {}", e);
            return;
        }
    };

    let mut problem = LinearOptimizationProblem::new();
    problem.set_optimizer(obj_expr, mechanism);

    println!("\nEnter constraints one per line (e.g. '3*x1 + 2.99*x2 <= 5').");
    println!("Press Enter on an empty line or type 'solve' when finished:");

    let mut count = 0;
    loop {
        print!("constraint #{}> ", count + 1);
        let _ = io::stdout().flush();

        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("solve") || trimmed.eq_ignore_ascii_case("done") {
            break;
        }

        match parse_constraint(trimmed) {
            Ok(constraint) => {
                problem.add_constraint(constraint);
                count += 1;
            }
            Err(err) => {
                eprintln!("  [!] Parse error: {}", err);
            }
        }
    }

    if count == 0 {
        println!("No constraints provided.");
        return;
    }

    println!("\nSolving problem with {} constraints...", count);
    let solution: HashMap<Variable, f64> = solve(problem);

    println!("\n=== Optimization Results ===");
    let mut vars: Vec<(Variable, f64)> = solution.into_iter().collect();
    vars.sort_by(|a, b| a.0.clone().get_name().cmp(&b.0.clone().get_name()));

    for (var, val) in &vars {
        println!("  {} = {:.4}", var.clone().get_name(), val);
    }
    println!("============================");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_expression() {
        let expr = parse_expression("3*x1 + 2.99*x2 + 5").unwrap();
        let mut map = HashMap::new();
        let x1 = Variable::new("x1");
        let x2 = Variable::new("x2");
        map.insert(x1.clone(), 2.0);
        map.insert(x2.clone(), 1.0);

        let val = expr.get_root_term().compute(map).unwrap();
        // 3*2 + 2.99*1 + 5 = 6 + 2.99 + 5 = 13.99
        assert!((val - 13.99).abs() < 1e-4);
    }

    #[test]
    fn test_parse_constraint() {
        let constraint = parse_constraint("3*x1 + 2.99*x2 <= 5").unwrap();
        assert_eq!(constraint.get_type(), ConstraintType::LE);
    }
}


