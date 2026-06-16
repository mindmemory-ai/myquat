//! Parameter handling for quantum gates
//!
//! This module provides support for both numeric and symbolic parameters,
//! similar to Qiskit's parameter expressions.

use crate::error::{MyQuatError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// A parameter that can be either a concrete value or a symbolic expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Parameter {
    /// A concrete floating-point value
    Float(f64),
    /// A symbolic parameter with a name
    Symbol(String),
    /// A mathematical expression involving parameters
    Expression(Box<ParameterExpression>),
}

/// Mathematical expressions for symbolic parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterExpression {
    /// A single parameter
    Parameter(Parameter),
    /// Addition of two expressions
    Add(Box<ParameterExpression>, Box<ParameterExpression>),
    /// Subtraction of two expressions
    Sub(Box<ParameterExpression>, Box<ParameterExpression>),
    /// Multiplication of two expressions
    Mul(Box<ParameterExpression>, Box<ParameterExpression>),
    /// Division of two expressions
    Div(Box<ParameterExpression>, Box<ParameterExpression>),
    /// Sine function
    Sin(Box<ParameterExpression>),
    /// Cosine function
    Cos(Box<ParameterExpression>),
    /// Exponential function
    Exp(Box<ParameterExpression>),
    /// Natural logarithm
    Ln(Box<ParameterExpression>),
    /// Power function
    Pow(Box<ParameterExpression>, Box<ParameterExpression>),
}

impl Parameter {
    /// Create a new float parameter
    pub fn new_float(value: f64) -> Self {
        Parameter::Float(value)
    }

    /// Create a new symbolic parameter
    pub fn new_symbol(name: impl Into<String>) -> Self {
        Parameter::Symbol(name.into())
    }

    /// Create a new expression parameter
    pub fn new_expression(expr: ParameterExpression) -> Self {
        Parameter::Expression(Box::new(expr))
    }

    /// Evaluate the parameter with given symbol values
    pub fn evaluate(&self, symbols: &HashMap<String, f64>) -> Result<f64> {
        match self {
            Parameter::Float(value) => Ok(*value),
            Parameter::Symbol(name) => symbols.get(name).copied().ok_or_else(|| {
                MyQuatError::invalid_parameter(format!("Undefined symbol: {}", name))
            }),
            Parameter::Expression(expr) => expr.evaluate(symbols),
        }
    }

    /// Check if the parameter is numeric (can be evaluated without symbols)
    pub fn is_numeric(&self) -> bool {
        match self {
            Parameter::Float(_) => true,
            Parameter::Symbol(_) => false,
            Parameter::Expression(expr) => expr.is_numeric(),
        }
    }

    /// Get the numeric value if the parameter is numeric
    pub fn numeric_value(&self) -> Option<f64> {
        match self {
            Parameter::Float(value) => Some(*value),
            Parameter::Symbol(_) => None,
            Parameter::Expression(expr) => expr.numeric_value(),
        }
    }

    /// Get all symbols used in this parameter
    pub fn symbols(&self) -> Vec<String> {
        let mut symbols = Vec::new();
        self.collect_symbols(&mut symbols);
        symbols.sort();
        symbols.dedup();
        symbols
    }

    fn collect_symbols(&self, symbols: &mut Vec<String>) {
        match self {
            Parameter::Float(_) => {}
            Parameter::Symbol(name) => symbols.push(name.clone()),
            Parameter::Expression(expr) => expr.collect_symbols(symbols),
        }
    }
}

impl ParameterExpression {
    /// Evaluate the expression with given symbol values
    pub fn evaluate(&self, symbols: &HashMap<String, f64>) -> Result<f64> {
        match self {
            ParameterExpression::Parameter(param) => param.evaluate(symbols),
            ParameterExpression::Add(left, right) => {
                Ok(left.evaluate(symbols)? + right.evaluate(symbols)?)
            }
            ParameterExpression::Sub(left, right) => {
                Ok(left.evaluate(symbols)? - right.evaluate(symbols)?)
            }
            ParameterExpression::Mul(left, right) => {
                Ok(left.evaluate(symbols)? * right.evaluate(symbols)?)
            }
            ParameterExpression::Div(left, right) => {
                let right_val = right.evaluate(symbols)?;
                if right_val == 0.0 {
                    return Err(MyQuatError::invalid_parameter("Division by zero"));
                }
                Ok(left.evaluate(symbols)? / right_val)
            }
            ParameterExpression::Sin(expr) => Ok(expr.evaluate(symbols)?.sin()),
            ParameterExpression::Cos(expr) => Ok(expr.evaluate(symbols)?.cos()),
            ParameterExpression::Exp(expr) => Ok(expr.evaluate(symbols)?.exp()),
            ParameterExpression::Ln(expr) => {
                let val = expr.evaluate(symbols)?;
                if val <= 0.0 {
                    return Err(MyQuatError::invalid_parameter(
                        "Logarithm of non-positive number",
                    ));
                }
                Ok(val.ln())
            }
            ParameterExpression::Pow(base, exp) => {
                Ok(base.evaluate(symbols)?.powf(exp.evaluate(symbols)?))
            }
        }
    }

    /// Check if the expression is numeric
    pub fn is_numeric(&self) -> bool {
        match self {
            ParameterExpression::Parameter(param) => param.is_numeric(),
            ParameterExpression::Add(left, right)
            | ParameterExpression::Sub(left, right)
            | ParameterExpression::Mul(left, right)
            | ParameterExpression::Div(left, right)
            | ParameterExpression::Pow(left, right) => left.is_numeric() && right.is_numeric(),
            ParameterExpression::Sin(expr)
            | ParameterExpression::Cos(expr)
            | ParameterExpression::Exp(expr)
            | ParameterExpression::Ln(expr) => expr.is_numeric(),
        }
    }

    /// Get the numeric value if the expression is numeric
    pub fn numeric_value(&self) -> Option<f64> {
        if self.is_numeric() {
            self.evaluate(&HashMap::new()).ok()
        } else {
            None
        }
    }

    fn collect_symbols(&self, symbols: &mut Vec<String>) {
        match self {
            ParameterExpression::Parameter(param) => param.collect_symbols(symbols),
            ParameterExpression::Add(left, right)
            | ParameterExpression::Sub(left, right)
            | ParameterExpression::Mul(left, right)
            | ParameterExpression::Div(left, right)
            | ParameterExpression::Pow(left, right) => {
                left.collect_symbols(symbols);
                right.collect_symbols(symbols);
            }
            ParameterExpression::Sin(expr)
            | ParameterExpression::Cos(expr)
            | ParameterExpression::Exp(expr)
            | ParameterExpression::Ln(expr) => expr.collect_symbols(symbols),
        }
    }
}

impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Parameter::Float(value) => write!(f, "{}", value),
            Parameter::Symbol(name) => write!(f, "{}", name),
            Parameter::Expression(expr) => write!(f, "{}", expr),
        }
    }
}

impl fmt::Display for ParameterExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParameterExpression::Parameter(param) => write!(f, "{}", param),
            ParameterExpression::Add(left, right) => write!(f, "({} + {})", left, right),
            ParameterExpression::Sub(left, right) => write!(f, "({} - {})", left, right),
            ParameterExpression::Mul(left, right) => write!(f, "({} * {})", left, right),
            ParameterExpression::Div(left, right) => write!(f, "({} / {})", left, right),
            ParameterExpression::Sin(expr) => write!(f, "sin({})", expr),
            ParameterExpression::Cos(expr) => write!(f, "cos({})", expr),
            ParameterExpression::Exp(expr) => write!(f, "exp({})", expr),
            ParameterExpression::Ln(expr) => write!(f, "ln({})", expr),
            ParameterExpression::Pow(base, exp) => write!(f, "({} ^ {})", base, exp),
        }
    }
}

impl From<f64> for Parameter {
    fn from(value: f64) -> Self {
        Parameter::Float(value)
    }
}

impl From<&str> for Parameter {
    fn from(name: &str) -> Self {
        Parameter::Symbol(name.to_string())
    }
}

impl From<String> for Parameter {
    fn from(name: String) -> Self {
        Parameter::Symbol(name)
    }
}

// Implement Neg trait for Parameter to support unary minus
impl std::ops::Neg for Parameter {
    type Output = Parameter;

    fn neg(self) -> Self::Output {
        match self {
            Parameter::Float(value) => Parameter::Float(-value),
            Parameter::Symbol(_) | Parameter::Expression(_) => {
                // For symbolic parameters, create a multiplication by -1
                let minus_one = Parameter::Float(-1.0);
                let expr = ParameterExpression::Mul(
                    Box::new(ParameterExpression::Parameter(minus_one)),
                    Box::new(ParameterExpression::Parameter(self)),
                );
                Parameter::Expression(Box::new(expr))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float_parameter() {
        let param = Parameter::new_float(3.14);
        assert!(param.is_numeric());
        assert_eq!(param.numeric_value(), Some(3.14));
        assert_eq!(param.evaluate(&HashMap::new()).unwrap(), 3.14);
    }

    #[test]
    fn test_symbol_parameter() {
        let param = Parameter::new_symbol("theta");
        assert!(!param.is_numeric());
        assert_eq!(param.numeric_value(), None);

        let mut symbols = HashMap::new();
        symbols.insert("theta".to_string(), 1.57);
        assert_eq!(param.evaluate(&symbols).unwrap(), 1.57);
    }

    #[test]
    fn test_expression_parameter() {
        let theta = Parameter::new_symbol("theta");
        let pi = Parameter::new_float(std::f64::consts::PI);

        let expr = ParameterExpression::Add(
            Box::new(ParameterExpression::Parameter(theta)),
            Box::new(ParameterExpression::Parameter(pi)),
        );

        let param = Parameter::new_expression(expr);
        assert!(!param.is_numeric());

        let mut symbols = HashMap::new();
        symbols.insert("theta".to_string(), 1.57);
        let result = param.evaluate(&symbols).unwrap();
        assert!((result - (1.57 + std::f64::consts::PI)).abs() < 1e-10);
    }
}
