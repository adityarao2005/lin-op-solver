use solver_lib::expression::{Constant, Expression, Sum, Variable, VariableTerm};
use std::collections::HashMap;

fn main() {
    let x = Variable::new("x");
    let expr = Expression::new(Box::new(Sum::new(vec![
        Box::new(VariableTerm::new(x.clone())),
        Box::new(Constant::new(10.0)),
    ])));

    let mut values = HashMap::new();
    values.insert(x, 5.0);

    let result = expr.get_root_term().compute(values);
    println!("Evaluation result: {:?}", result);
}

