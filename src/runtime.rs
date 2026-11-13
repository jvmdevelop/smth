use crate::{Expr, InequalityOperator};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct VariableRegistry {
    pub variables: BTreeMap<String, f64>,
    pub parameters: BTreeMap<String, f64>,
}

impl VariableRegistry {
    pub fn set_variable(&mut self, name: impl Into<String>, value: f64) {
        self.variables.insert(name.into(), value);
    }
    pub fn set_parameter(&mut self, name: impl Into<String>, value: f64) {
        self.parameters.insert(name.into(), value);
    }
    pub fn variable(&self, name: &str) -> Option<f64> {
        self.variables.get(name).copied()
    }
    pub fn parameter(&self, name: &str) -> Option<f64> {
        self.parameters.get(name).copied()
    }
    fn resolve(&self, name: &str) -> Option<f64> {
        self.variables
            .get(name)
            .or_else(|| self.parameters.get(name))
            .copied()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConditionValidator {
    pub tolerance: f64,
}

impl Default for ConditionValidator {
    fn default() -> Self {
        Self { tolerance: 1e-9 }
    }
}

impl ConditionValidator {
    pub fn new(tolerance: f64) -> Self {
        Self { tolerance }
    }

    pub fn validate(&self, condition: &Expr, variables: &VariableRegistry) -> Result<bool, String> {
        validate_tolerance(self.tolerance)?;
        match condition {
            Expr::Condition {
                expression,
                expected_value,
            } => Ok(
                eval_bool(expression, variables, None, None, self.tolerance)? == *expected_value,
            ),
            other => Err(format!("expected condition expression, got {other:?}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlotSettings {
    pub x_range: (f64, f64),
    pub y_range: (f64, f64),
    pub resolution: usize,
}

impl Default for PlotSettings {
    fn default() -> Self {
        Self {
            x_range: (-10.0, 10.0),
            y_range: (-10.0, 10.0),
            resolution: 160,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    pub variables: VariableRegistry,
    pub condition_validator: ConditionValidator,
    pub plot: PlotSettings,
    pub basis_dimension: usize,
    pub basis: Option<Basis>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Basis {
    pub name: String,
    pub dimension: usize,
    pub coordinates: Vec<f64>,
}

impl Basis {
    pub fn transform(&self, x: f64, y: f64, z: f64) -> [f64; 3] {
        let c = &self.coordinates;
        if self.dimension == 2 {
            [c[0] * x + c[2] * y, c[1] * x + c[3] * y, z]
        } else {
            [
                c[0] * x + c[3] * y + c[6] * z,
                c[1] * x + c[4] * y + c[7] * z,
                c[2] * x + c[5] * y + c[8] * z,
            ]
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotKind {
    Function,
    ImplicitCurve,
    Region,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Plot {
    pub kind: PlotKind,
    pub expression: Expr,
    pub variables: VariableRegistry,
    pub dimension: usize,
    pub basis: Option<Basis>,
    pub resolution: usize,
    pub x_range: (f64, f64),
    pub y_range: (f64, f64),
}

impl Plot {
    pub fn residual(&self, x: f64, y: f64, z: f64) -> Result<f64, String> {
        if let Expr::Equals(left, right) | Expr::NotEquals(left, right) = &self.expression {
            if left.len() == 1 && right.len() == 1 {
                return Ok(
                    eval_number(&left[0], &self.variables, None, Some((x, y, z)))?
                        - eval_number(&right[0], &self.variables, None, Some((x, y, z)))?,
                );
            }
        }
        self.evaluate(x, y, z)
    }

    pub fn evaluate(&self, x: f64, y: f64, z: f64) -> Result<f64, String> {
        let expr = match &self.expression {
            Expr::Equals(left, right)
                if left.len() == 1
                    && right.len() == 1
                    && matches!(&left[0], Expr::Variable(name) if name == if self.dimension == 3 { "z" } else { "y" }) =>
            {
                &right[0]
            }
            other => other,
        };
        eval_number(expr, &self.variables, None, Some((x, y, z)))
    }
    pub fn satisfies(&self, x: f64, y: f64, z: f64, tolerance: f64) -> Result<bool, String> {
        eval_bool(
            &self.expression,
            &self.variables,
            Some((x, y, z)),
            None,
            tolerance,
        )
    }
    pub fn transform(&self, x: f64, y: f64, z: f64) -> [f64; 3] {
        self.basis
            .as_ref()
            .map_or([x, y, z], |basis| basis.transform(x, y, z))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SystemExecution {
    pub system_index: usize,
    pub kind: String,
    pub valid: Option<bool>,
    pub value: Option<f64>,
    pub plot: Option<Plot>,
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub valid: bool,
    pub systems: Vec<SystemExecution>,
    pub plots: Vec<Plot>,
    pub generated_variables: BTreeMap<String, f64>,
}

impl Default for ExecutionResult {
    fn default() -> Self {
        Self {
            valid: true,
            systems: Vec::new(),
            plots: Vec::new(),
            generated_variables: BTreeMap::new(),
        }
    }
}

pub(crate) fn validate_plot_settings(settings: &PlotSettings) -> Result<(), String> {
    for (name, (min, max)) in [("x", settings.x_range), ("y", settings.y_range)] {
        if !min.is_finite() || !max.is_finite() || min >= max {
            return Err(format!("plot {name} range must be finite and increasing"));
        }
    }
    if !(2..=1024).contains(&settings.resolution) {
        return Err("plot resolution must be between 2 and 1024".into());
    }
    Ok(())
}

pub(crate) fn validate_tolerance(tolerance: f64) -> Result<(), String> {
    if !tolerance.is_finite() || tolerance < 0.0 {
        return Err("condition tolerance must be finite and non-negative".into());
    }
    Ok(())
}

pub(crate) fn compare(lhs: f64, operator: InequalityOperator, rhs: f64, tolerance: f64) -> bool {
    match operator {
        InequalityOperator::Less => lhs < rhs - tolerance,
        InequalityOperator::LessEqual => lhs <= rhs + tolerance,
        InequalityOperator::Greater => lhs > rhs + tolerance,
        InequalityOperator::GreaterEqual => lhs >= rhs - tolerance,
        InequalityOperator::NotEqual => (lhs - rhs).abs() > tolerance,
    }
}

pub(crate) fn eval_number(
    expr: &Expr,
    variables: &VariableRegistry,
    parameter_override: Option<(&str, f64)>,
    point: Option<(f64, f64, f64)>,
) -> Result<f64, String> {
    let value = match expr {
        Expr::Number(number) => number
            .parse::<f64>()
            .map_err(|_| format!("invalid number {number:?}"))?,
        Expr::Euler => std::f64::consts::E,
        Expr::Pi => std::f64::consts::PI,
        Expr::Variable(name) => {
            if let Some((x, y, z)) = point {
                match name.as_str() {
                    "x" => x,
                    "y" => y,
                    "z" => z,
                    _ if parameter_override.is_some_and(|(p, _)| p == name) => {
                        parameter_override.unwrap().1
                    }
                    _ => variables
                        .resolve(name)
                        .ok_or_else(|| format!("variable {name:?} has no registered value"))?,
                }
            } else if parameter_override.is_some_and(|(p, _)| p == name) {
                parameter_override.unwrap().1
            } else {
                variables
                    .resolve(name)
                    .ok_or_else(|| format!("variable {name:?} has no registered value"))?
            }
        }
        Expr::Plus(a, b) => {
            eval_number(a, variables, parameter_override, point)?
                + eval_number(b, variables, parameter_override, point)?
        }
        Expr::Minus(a, b) => {
            eval_number(a, variables, parameter_override, point)?
                - eval_number(b, variables, parameter_override, point)?
        }
        Expr::Multiply(a, b) => {
            eval_number(a, variables, parameter_override, point)?
                * eval_number(b, variables, parameter_override, point)?
        }
        Expr::Divide(a, b) => {
            let denominator = eval_number(b, variables, parameter_override, point)?;
            if denominator == 0.0 {
                return Err("division by zero".into());
            }
            eval_number(a, variables, parameter_override, point)? / denominator
        }
        Expr::Degree(base, exponent) => {
            eval_number(base, variables, parameter_override, point)?.powi(*exponent as i32)
        }
        Expr::AnyRoot(parts) if parts.len() == 2 => {
            let degree = eval_number(&parts[0], variables, parameter_override, point)? as usize;
            if degree == 0 {
                return Err("zero-degree root is undefined".into());
            }
            let radicand = eval_number(&parts[1], variables, parameter_override, point)?;
            if radicand < 0.0 && degree % 2 == 0 {
                return Err("even root of a negative number is not real".into());
            }
            radicand.signum() * radicand.abs().powf(1.0 / degree as f64)
        }
        Expr::Factorial(parts) if parts.len() == 1 => {
            let n = eval_number(&parts[0], variables, parameter_override, point)?;
            if n < 0.0 || n.fract() != 0.0 || n > 170.0 {
                return Err("factorial requires an integer from 0 to 170".into());
            }
            (1..=n as u64).fold(1.0, |acc, value| acc * value as f64)
        }
        Expr::Index(base, index) => match base.as_ref() {
            Expr::Vector(_, values) => eval_number(
                values
                    .get(*index)
                    .ok_or_else(|| format!("vector index {index} is out of bounds"))?,
                variables,
                parameter_override,
                point,
            )?,
            _ => return Err("indexing is supported for vectors only".into()),
        },
        _ => return Err(format!("expression {expr:?} does not produce a number")),
    };
    if !value.is_finite() {
        return Err("expression produced a non-finite number".into());
    }
    Ok(value)
}

pub(crate) fn eval_bool(
    expr: &Expr,
    variables: &VariableRegistry,
    point: Option<(f64, f64, f64)>,
    parameter_override: Option<(&str, f64)>,
    tolerance: f64,
) -> Result<bool, String> {
    let eval_pair = |left: &Expr, right: &Expr| -> Result<(f64, f64), String> {
        Ok((
            eval_number(left, variables, parameter_override, point)?,
            eval_number(right, variables, parameter_override, point)?,
        ))
    };
    match expr {
        Expr::Equals(left, right) if left.len() == 1 && right.len() == 1 => {
            let (a, b) = eval_pair(&left[0], &right[0])?;
            Ok((a - b).abs() <= tolerance)
        }
        Expr::NotEquals(left, right) if left.len() == 1 && right.len() == 1 => {
            let (a, b) = eval_pair(&left[0], &right[0])?;
            Ok((a - b).abs() > tolerance)
        }
        Expr::Less(a, b) => {
            let (a, b) = eval_pair(a, b)?;
            Ok(compare(a, InequalityOperator::Less, b, tolerance))
        }
        Expr::LessEqual(a, b) => {
            let (a, b) = eval_pair(a, b)?;
            Ok(compare(a, InequalityOperator::LessEqual, b, tolerance))
        }
        Expr::Greater(a, b) => {
            let (a, b) = eval_pair(a, b)?;
            Ok(compare(a, InequalityOperator::Greater, b, tolerance))
        }
        Expr::GreaterEqual(a, b) => {
            let (a, b) = eval_pair(a, b)?;
            Ok(compare(a, InequalityOperator::GreaterEqual, b, tolerance))
        }
        Expr::LogicalAnd(a, b) => {
            Ok(
                eval_bool(a, variables, point, parameter_override, tolerance)?
                    && eval_bool(b, variables, point, parameter_override, tolerance)?,
            )
        }
        Expr::LogicalOr(a, b) => Ok(
            eval_bool(a, variables, point, parameter_override, tolerance)?
                || eval_bool(b, variables, point, parameter_override, tolerance)?,
        ),
        Expr::LogicalNot(inner) => Ok(!eval_bool(
            inner,
            variables,
            point,
            parameter_override,
            tolerance,
        )?),
        Expr::Condition {
            expression,
            expected_value,
        } => Ok(
            eval_bool(expression, variables, point, parameter_override, tolerance)?
                == *expected_value,
        ),
        Expr::SystemOfEq(left, right) => {
            for item in left.iter().chain(right.iter()) {
                if !eval_bool(item, variables, point, parameter_override, tolerance)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Expr::PlotRequirement(items) if items.len() == 1 => {
            eval_bool(&items[0], variables, point, parameter_override, tolerance)
        }
        _ => Ok(eval_number(expr, variables, parameter_override, point)?.abs() > tolerance),
    }
}

fn contains_variable(expr: &Expr, name: &str) -> bool {
    match expr {
        Expr::Variable(variable) => variable == name,
        Expr::Degree(a, _) | Expr::Index(a, _) | Expr::LogicalNot(a) => contains_variable(a, name),
        Expr::Multiply(a, b)
        | Expr::Divide(a, b)
        | Expr::Plus(a, b)
        | Expr::Minus(a, b)
        | Expr::Less(a, b)
        | Expr::LessEqual(a, b)
        | Expr::Greater(a, b)
        | Expr::GreaterEqual(a, b)
        | Expr::LogicalOr(a, b)
        | Expr::LogicalAnd(a, b) => contains_variable(a, name) || contains_variable(b, name),
        Expr::AnyRoot(items)
        | Expr::Factorial(items)
        | Expr::Equals(items, _)
        | Expr::NotEquals(items, _)
        | Expr::PlotRequirement(items) => items.iter().any(|item| contains_variable(item, name)),
        Expr::SystemOfEq(left, right) => left
            .iter()
            .chain(right.iter())
            .any(|item| contains_variable(item, name)),
        Expr::Condition { expression, .. } => contains_variable(expression, name),
        Expr::Vector(_, values) | Expr::Tensor(_, _, values) => {
            values.iter().any(|item| contains_variable(item, name))
        }
        Expr::Parametrization { equations, .. } => {
            equations.iter().any(|item| contains_variable(item, name))
        }
        Expr::Number(_) | Expr::Euler | Expr::Pi | Expr::Basis { .. } => false,
    }
}

fn plot_kind(expr: &Expr) -> PlotKind {
    match expr {
        Expr::Condition { expression, .. } => plot_kind(expression),
        Expr::Equals(left, right)
            if left.len() == 1
                && right.len() == 1
                && (matches!(&left[0], Expr::Variable(name) if name == "y" && !contains_variable(&right[0], "y") && !contains_variable(&right[0], "z") )
                    || matches!(&left[0], Expr::Variable(name) if name == "z" && !contains_variable(&right[0], "z"))) =>
        {
            PlotKind::Function
        }
        Expr::Equals(_, _) | Expr::NotEquals(_, _) => PlotKind::ImplicitCurve,
        Expr::Less(_, _)
        | Expr::LessEqual(_, _)
        | Expr::Greater(_, _)
        | Expr::GreaterEqual(_, _) => PlotKind::Region,
        Expr::LogicalAnd(_, _) | Expr::LogicalOr(_, _) | Expr::LogicalNot(_) => PlotKind::Region,
        _ if contains_variable(expr, "y") => PlotKind::ImplicitCurve,
        _ => PlotKind::Function,
    }
}

pub(crate) fn build_plot(
    expr: &Expr,
    variables: &VariableRegistry,
    settings: &PlotSettings,
    dimension: usize,
    basis: Option<Basis>,
) -> Result<Plot, String> {
    let kind = plot_kind(expr);
    Ok(Plot {
        kind,
        expression: expr.clone(),
        variables: variables.clone(),
        dimension: if dimension == 3 { 3 } else { 2 },
        basis,
        resolution: settings.resolution,
        x_range: settings.x_range,
        y_range: settings.y_range,
    })
}
