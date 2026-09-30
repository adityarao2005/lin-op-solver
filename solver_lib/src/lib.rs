pub mod expression;

use expression::Expression;

pub enum ConstraintType {
    EQUAL,
    GE,
    GT,
    LE,
    LT,
}


pub trait LinearOptimizationConstraint {
    fn get_type(self) -> ConstraintType;
    fn get_left_hand(self) -> Expression;
    fn get_right_hand(self) -> Expression;
}

pub enum OptimizationMechanism {
    MAX,
    MIN,
}

pub trait LinearOptimizationProblem {
    fn add_constraint(constraint: impl LinearOptimizationConstraint);
    fn set_optimizer(expression: Expression, mechanism: OptimizationMechanism);
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
