use std::{collections::{HashMap, HashSet}, vec};

use crate::expression::TermComputationError::VariablesNotSpecified;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Variable {
    name: String,
}

impl Variable {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn get_name(self) -> String {
        self.name
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TermComputationError {
    VariablesNotSpecified(Vec<Variable>),
    ArithmeticError(String),
}

pub trait Term {
    fn get_variables(&self) -> HashSet<Variable>;
    fn compute(&self, value_map: HashMap<Variable, f64>) -> Result<f64, TermComputationError>;
    fn get_child_terms(&self) -> &[Box<dyn Term>];
}

pub struct Sum {
    terms: Vec<Box<dyn Term>>,
}

impl Sum {
    pub fn new(terms: Vec<Box<dyn Term>>) -> Self {
        Self { terms }
    }
}

impl Term for Sum {
    fn get_child_terms(&self) -> &[Box<dyn Term>] {
        &self.terms
    }

    fn get_variables(&self) -> HashSet<Variable> {
        let mut variables = HashSet::new();

        for term in self.terms.iter() {
            let term_vars = term.get_variables();

            for term_var in term_vars {
                variables.insert(term_var);
            }
        }

        return variables;
    }

    fn compute(&self, value_map: HashMap<Variable, f64>) -> Result<f64, TermComputationError> {
        let mut sum: f64 = 0.0;

        for term in self.terms.iter() {
            let result = term.compute(value_map.clone());

            match result {
                Ok(value) => {
                    sum += value;
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        return Ok(sum);
    }
}

pub struct Product {
    terms: Vec<Box<dyn Term>>,
}

impl Product {
    pub fn new(terms: Vec<Box<dyn Term>>) -> Self {
        Self { terms }
    }
}

impl Term for Product {
    fn get_child_terms(&self) -> &[Box<dyn Term>] {
        &self.terms
    }

    fn get_variables(&self) -> HashSet<Variable> {
        let mut variables = HashSet::new();

        for term in self.terms.iter() {
            let term_vars = term.get_variables();

            for term_var in term_vars {
                variables.insert(term_var);
            }
        }

        return variables;
    }

    fn compute(&self, value_map: HashMap<Variable, f64>) -> Result<f64, TermComputationError> {
        let mut product: f64 = 1.0;

        for term in self.terms.iter() {
            let result = term.compute(value_map.clone());

            match result {
                Ok(value) => {
                    product *= value;
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        return Ok(product);
    }
}

pub struct Constant {
    value: f64,
}

impl Constant {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

impl Term for Constant {
    fn compute(&self, _: HashMap<Variable, f64>) -> Result<f64, TermComputationError> {
        Ok(self.value)
    }

    fn get_child_terms(&self) -> &[Box<dyn Term>] {
        &[]
    }

    fn get_variables(&self) -> HashSet<Variable> {
        HashSet::new()
    }
}

pub struct VariableTerm {
    var: Variable,
}

impl VariableTerm {
    pub fn new(var: Variable) -> Self {
        Self { var }
    }
}

impl Term for VariableTerm {
    fn compute(&self, lookup: HashMap<Variable, f64>) -> Result<f64, TermComputationError> {
        let value = lookup.get(&self.var);

        match value {
            Some(value) => {
                return Ok(value.clone());
            }
            None => Err(VariablesNotSpecified(vec![self.var.clone()])),
        }
    }

    fn get_child_terms(&self) -> &[Box<dyn Term>] {
        &[]
    }

    fn get_variables(&self) -> HashSet<Variable> {
        HashSet::from([self.var.clone()])
    }
}

pub struct Expression {
    root_term: Box<dyn Term>,
}

impl Expression {
    pub fn new(root_term: Box<dyn Term>) -> Self {
        return Self {
            root_term: root_term,
        };
    }

    pub fn get_root_term(self) -> Box<dyn Term> {
        return self.root_term;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_creation_and_equality() {
        let v1 = Variable::new("x");
        let v2 = Variable::new("x");
        let v3 = Variable::new("y");

        assert_eq!(v1, v2);
        assert_ne!(v1, v3);
        assert_eq!(v1.clone().get_name(), "x");
    }

    #[test]
    fn test_constant_term() {
        let constant = Constant::new(42.0);
        let empty_map = HashMap::new();

        assert_eq!(constant.compute(empty_map).unwrap(), 42.0);
        assert!(constant.get_variables().is_empty());
        assert!(constant.get_child_terms().is_empty());
    }

    #[test]
    fn test_variable_term_success() {
        let var = Variable::new("x");
        let var_term = VariableTerm::new(var.clone());

        let mut map = HashMap::new();
        map.insert(var.clone(), 3.5);

        assert_eq!(var_term.compute(map).unwrap(), 3.5);
        assert_eq!(var_term.get_variables(), HashSet::from([var]));
        assert!(var_term.get_child_terms().is_empty());
    }

    #[test]
    fn test_variable_term_missing_variable() {
        let var = Variable::new("x");
        let var_term = VariableTerm::new(var.clone());
        let empty_map = HashMap::new();

        let result = var_term.compute(empty_map);
        assert_eq!(
            result,
            Err(TermComputationError::VariablesNotSpecified(vec![var]))
        );
    }

    #[test]
    fn test_sum_compute() {
        let x = Variable::new("x");
        let sum = Sum::new(vec![
            Box::new(Constant::new(5.0)),
            Box::new(VariableTerm::new(x.clone())),
            Box::new(Constant::new(2.5)),
        ]);

        let mut map = HashMap::new();
        map.insert(x.clone(), 10.0);

        assert_eq!(sum.compute(map).unwrap(), 17.5);
        assert_eq!(sum.get_variables(), HashSet::from([x]));
        assert_eq!(sum.get_child_terms().len(), 3);
    }

    #[test]
    fn test_sum_missing_variable_error() {
        let x = Variable::new("x");
        let sum = Sum::new(vec![
            Box::new(Constant::new(5.0)),
            Box::new(VariableTerm::new(x.clone())),
        ]);

        let result = sum.compute(HashMap::new());
        assert_eq!(
            result,
            Err(TermComputationError::VariablesNotSpecified(vec![x]))
        );
    }

    #[test]
    fn test_product_compute() {
        let x = Variable::new("x");
        let y = Variable::new("y");
        let product = Product::new(vec![
            Box::new(Constant::new(2.0)),
            Box::new(VariableTerm::new(x.clone())),
            Box::new(VariableTerm::new(y.clone())),
        ]);

        let mut map = HashMap::new();
        map.insert(x.clone(), 3.0);
        map.insert(y.clone(), 4.0);

        assert_eq!(product.compute(map).unwrap(), 24.0);
        assert_eq!(product.get_variables(), HashSet::from([x, y]));
        assert_eq!(product.get_child_terms().len(), 3);
    }

    #[test]
    fn test_product_missing_variable_error() {
        let x = Variable::new("x");
        let product = Product::new(vec![
            Box::new(Constant::new(2.0)),
            Box::new(VariableTerm::new(x.clone())),
        ]);

        let result = product.compute(HashMap::new());
        assert_eq!(
            result,
            Err(TermComputationError::VariablesNotSpecified(vec![x]))
        );
    }

    #[test]
    fn test_nested_expression_composite() {
        // Compute: (2 * x) + (3 * y) + 1
        let x = Variable::new("x");
        let y = Variable::new("y");

        let term_2x = Product::new(vec![
            Box::new(Constant::new(2.0)),
            Box::new(VariableTerm::new(x.clone())),
        ]);

        let term_3y = Product::new(vec![
            Box::new(Constant::new(3.0)),
            Box::new(VariableTerm::new(y.clone())),
        ]);

        let root_sum = Sum::new(vec![
            Box::new(term_2x),
            Box::new(term_3y),
            Box::new(Constant::new(1.0)),
        ]);

        let expr = Expression::new(Box::new(root_sum));
        let root = expr.get_root_term();

        let mut map = HashMap::new();
        map.insert(x.clone(), 4.0);
        map.insert(y.clone(), 5.0);

        // 2*4 + 3*5 + 1 = 8 + 15 + 1 = 24
        assert_eq!(root.compute(map).unwrap(), 24.0);
        assert_eq!(root.get_variables(), HashSet::from([x, y]));
    }
}
