pub mod expression;

use std::collections::HashSet;

use expression::Expression;

use crate::{OptimizationMechanism::MAX, expression::{Constant, Variable}};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstraintType {
    EQUAL,
    GE,
    GT,
    LE,
    LT,
}

pub struct LinearOptimizationConstraint {
    left_hand: Expression,
    right_hand: Expression,
    constraint_type: ConstraintType,
}

impl LinearOptimizationConstraint {
    pub fn new(
        left_hand: Expression,
        right_hand: Expression,
        constraint_type: ConstraintType,
    ) -> Self {
        Self {
            left_hand,
            constraint_type,
            right_hand,
        }
    }

    pub fn get_type(&self) -> ConstraintType {
        self.constraint_type
    }

    pub fn get_left_hand(&self) -> &Expression {
        &self.left_hand
    }

    pub fn get_right_hand(&self) -> &Expression {
        &self.right_hand
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptimizationMechanism {
    MAX,
    MIN,
}

pub struct LinearOptimizationProblem {
    constraints: Vec<LinearOptimizationConstraint>,
    optimizer: Expression,
    mechanism: OptimizationMechanism,
}

impl LinearOptimizationProblem {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            mechanism: MAX,
            optimizer: Expression::new(Box::new(Constant::new(0.0))),
        }
    }

    pub fn add_constraint(&mut self, constraint: LinearOptimizationConstraint) -> &Self {
        self.constraints.push(constraint);
        self
    }

    pub fn set_optimizer(
        &mut self,
        expression: Expression,
        mechanism: OptimizationMechanism,
    ) -> &Self {
        self.mechanism = mechanism;
        self.optimizer = expression;
        self
    }

    pub fn get_variables(self) -> HashSet<Variable> {
        let mut vec = HashSet::new();

        for constraint in self.constraints {
            let set = constraint.left_hand.get_root_term().get_variables();

            for value in set {
                vec.insert(value);
            }
        }

        vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression::{Sum, Variable, VariableTerm};

    #[test]
    fn test_constraint_creation_and_accessors() {
        let lhs = Expression::new(Box::new(Constant::new(10.0)));
        let rhs = Expression::new(Box::new(Constant::new(20.0)));
        let constraint = LinearOptimizationConstraint::new(lhs, rhs, ConstraintType::LE);

        assert_eq!(constraint.get_type(), ConstraintType::LE);
        assert!(constraint.get_left_hand() as *const _ != std::ptr::null());
        assert!(constraint.get_right_hand() as *const _ != std::ptr::null());
    }

    #[test]
    fn test_problem_initialization() {
        let problem = LinearOptimizationProblem::new();
        assert_eq!(problem.get_variables(), HashSet::new());
    }

    #[test]
    fn test_problem_add_constraints_and_extract_variables() {
        let mut problem = LinearOptimizationProblem::new();

        let x = Variable::new("x");
        let y = Variable::new("y");
        let z = Variable::new("z");

        // Constraint 1: x + y <= 10
        let c1_lhs = Expression::new(Box::new(Sum::new(vec![
            Box::new(VariableTerm::new(x.clone())),
            Box::new(VariableTerm::new(y.clone())),
        ])));
        let c1_rhs = Expression::new(Box::new(Constant::new(10.0)));
        let c1 = LinearOptimizationConstraint::new(c1_lhs, c1_rhs, ConstraintType::LE);

        // Constraint 2: y + z == 5 (y is shared, testing deduplication in HashSet)
        let c2_lhs = Expression::new(Box::new(Sum::new(vec![
            Box::new(VariableTerm::new(y.clone())),
            Box::new(VariableTerm::new(z.clone())),
        ])));
        let c2_rhs = Expression::new(Box::new(Constant::new(5.0)));
        let c2 = LinearOptimizationConstraint::new(c2_lhs, c2_rhs, ConstraintType::EQUAL);

        problem.add_constraint(c1);
        problem.add_constraint(c2);

        let vars = problem.get_variables();
        assert_eq!(vars.len(), 3);
        assert_eq!(vars, HashSet::from([x, y, z]));
    }

    #[test]
    fn test_problem_set_optimizer() {
        let mut problem = LinearOptimizationProblem::new();

        let x = Variable::new("x");
        let objective = Expression::new(Box::new(VariableTerm::new(x.clone())));

        problem.set_optimizer(objective, OptimizationMechanism::MIN);

        // Verify problem accepts the optimizer and constraints can still be added
        let c_lhs = Expression::new(Box::new(VariableTerm::new(x.clone())));
        let c_rhs = Expression::new(Box::new(Constant::new(0.0)));
        problem.add_constraint(LinearOptimizationConstraint::new(
            c_lhs,
            c_rhs,
            ConstraintType::GE,
        ));

        assert_eq!(problem.get_variables(), HashSet::from([x]));
    }
}
