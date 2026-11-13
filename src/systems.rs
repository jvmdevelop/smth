use crate::{
    Basis, ExecutionContext, ExecutionResult, Expr, SystemExecution,
    parser::{Lexer, Parser},
    runtime::{build_plot, compare, eval_number, validate_plot_settings, validate_tolerance},
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathSystems {
    pub systems: Vec<MathSystemKind>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MathSystemKind {
    Expression(Expression),
    Equation(Equation),
    Inequality(Inequality),
    PlotRequirement(PlotRequirement),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression {
    pub expression: Expr,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Equation {
    pub left: Expr,
    pub right: Expr,
}
pub type Equality = Equation;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inequality {
    pub left: Expr,
    pub operator: InequalityOperator,
    pub right: Expr,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InequalityOperator {
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    NotEqual,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlotRequirement {
    pub expression: Expr,
}

pub trait MathSystem {
    fn kind_name(&self) -> &'static str;
    fn execute(
        &self,
        system_index: usize,
        context: &mut ExecutionContext,
    ) -> Result<SystemExecution, String>;
}
impl MathSystem for Expression {
    fn kind_name(&self) -> &'static str {
        "expression"
    }
    fn execute(
        &self,
        system_index: usize,
        context: &mut ExecutionContext,
    ) -> Result<SystemExecution, String> {
        let (kind, valid, value) = match &self.expression {
            condition @ Expr::Condition { .. } => (
                "condition",
                Some(
                    context
                        .condition_validator
                        .validate(condition, &context.variables)?,
                ),
                None,
            ),
            Expr::Parametrization {
                parameter_name,
                variable_name,
                equations,
            } => {
                let parameter_value =
                    context.variables.parameter(parameter_name).ok_or_else(|| {
                        format!("parameter {parameter_name:?} has no registered value")
                    })?;
                if equations.len() != 1 {
                    return Err(format!(
                        "parametrization for {variable_name:?} must contain one expression"
                    ));
                }
                let computed = eval_number(
                    &equations[0],
                    &context.variables,
                    Some((parameter_name, parameter_value)),
                    None,
                )?;
                context
                    .variables
                    .set_variable(variable_name.clone(), computed);
                ("parametrization", Some(true), Some(computed))
            }
            Expr::Basis { name, coordinates } => {
                let n = coordinates.len();
                let dim = (n as f64).sqrt() as usize;
                if dim * dim != n || !(2..=3).contains(&dim) {
                    return Err(format!("basis must be 2D or 3D, got {n} coordinates"));
                }
                let values = coordinates
                    .iter()
                    .map(|value| eval_number(value, &context.variables, None, None))
                    .collect::<Result<Vec<_>, _>>()?;
                let determinant = if dim == 2 {
                    values[0] * values[3] - values[2] * values[1]
                } else {
                    values[0] * (values[4] * values[8] - values[5] * values[7])
                        - values[3] * (values[1] * values[8] - values[2] * values[7])
                        + values[6] * (values[1] * values[5] - values[2] * values[4])
                };
                if determinant.abs() < 1e-12 {
                    return Err("basis vectors must be linearly independent".into());
                }
                context.basis = Some(Basis {
                    name: name.clone(),
                    dimension: dim,
                    coordinates: values,
                });
                context.basis_dimension = dim;
                ("basis", None, None)
            }
            Expr::Vector(_, _) | Expr::Tensor(_, _, _) => ("declaration", None, None),
            other => (
                "expression",
                None,
                Some(eval_number(other, &context.variables, None, None)?),
            ),
        };
        Ok(SystemExecution {
            system_index,
            kind: kind.into(),
            valid,
            value,
            plot: None,
        })
    }
}
impl MathSystem for Equation {
    fn kind_name(&self) -> &'static str {
        "equation"
    }
    fn execute(
        &self,
        system_index: usize,
        context: &mut ExecutionContext,
    ) -> Result<SystemExecution, String> {
        let lhs = eval_number(&self.left, &context.variables, None, None)?;
        let rhs = eval_number(&self.right, &context.variables, None, None)?;
        let valid = (lhs - rhs).abs() <= context.condition_validator.tolerance;
        Ok(SystemExecution {
            system_index,
            kind: self.kind_name().into(),
            valid: Some(valid),
            value: None,
            plot: None,
        })
    }
}
impl MathSystem for Inequality {
    fn kind_name(&self) -> &'static str {
        "inequality"
    }
    fn execute(
        &self,
        system_index: usize,
        context: &mut ExecutionContext,
    ) -> Result<SystemExecution, String> {
        let lhs = eval_number(&self.left, &context.variables, None, None)?;
        let rhs = eval_number(&self.right, &context.variables, None, None)?;
        let valid = compare(
            lhs,
            self.operator,
            rhs,
            context.condition_validator.tolerance,
        );
        Ok(SystemExecution {
            system_index,
            kind: self.kind_name().into(),
            valid: Some(valid),
            value: None,
            plot: None,
        })
    }
}
impl MathSystem for PlotRequirement {
    fn kind_name(&self) -> &'static str {
        "plot requirement"
    }
    fn execute(
        &self,
        system_index: usize,
        context: &mut ExecutionContext,
    ) -> Result<SystemExecution, String> {
        let plot = build_plot(
            &self.expression,
            &context.variables,
            &context.plot,
            context.basis_dimension,
            context.basis.clone(),
        )?;
        Ok(SystemExecution {
            system_index,
            kind: self.kind_name().into(),
            valid: Some(true),
            value: None,
            plot: Some(plot),
        })
    }
}
impl MathSystem for MathSystemKind {
    fn kind_name(&self) -> &'static str {
        match self {
            Self::Expression(value) => value.kind_name(),
            Self::Equation(value) => value.kind_name(),
            Self::Inequality(value) => value.kind_name(),
            Self::PlotRequirement(value) => value.kind_name(),
        }
    }
    fn execute(
        &self,
        system_index: usize,
        context: &mut ExecutionContext,
    ) -> Result<SystemExecution, String> {
        match self {
            Self::Expression(item) => item.execute(system_index, context),
            Self::Equation(item) => item.execute(system_index, context),
            Self::Inequality(item) => item.execute(system_index, context),
            Self::PlotRequirement(item) => item.execute(system_index, context),
        }
    }
}

impl MathSystems {
    pub fn parse(text: &str) -> Result<Self, String> {
        let tokens = Lexer::new(text).lex()?;
        let expr = Parser::new(tokens).parse()?;
        let mut systems = Vec::new();
        Self::collect(expr, &mut systems);
        Ok(Self { systems })
    }

    pub fn execute(&self, context: &ExecutionContext) -> Result<ExecutionResult, String> {
        validate_plot_settings(&context.plot)?;
        validate_tolerance(context.condition_validator.tolerance)?;
        let mut runtime = context.clone();
        let mut result = ExecutionResult::default();

        for (index, system) in self.systems.iter().enumerate() {
            let row = system.execute(index, &mut runtime)?;
            if let Some(passed) = row.valid {
                result.valid &= passed;
            }
            if let Some(plot) = &row.plot {
                result.plots.push(plot.clone());
            }
            result.systems.push(row);
        }

        for (name, value) in runtime.variables.variables {
            if context.variables.variables.get(&name) != Some(&value) {
                result.generated_variables.insert(name, value);
            }
        }
        Ok(result)
    }

    fn collect(expr: Expr, out: &mut Vec<MathSystemKind>) {
        match expr {
            Expr::SystemOfEq(left, right) => {
                for item in left.into_vec().into_iter().chain(right.into_vec()) {
                    Self::collect(item, out);
                }
            }
            Expr::Equals(left, right) if left.len() == 1 && right.len() == 1 => {
                out.push(MathSystemKind::Equation(Equation {
                    left: left[0].clone(),
                    right: right[0].clone(),
                }))
            }
            Expr::NotEquals(left, right) if left.len() == 1 && right.len() == 1 => {
                out.push(MathSystemKind::Inequality(Inequality {
                    left: left[0].clone(),
                    operator: InequalityOperator::NotEqual,
                    right: right[0].clone(),
                }))
            }
            Expr::Less(left, right) => out.push(MathSystemKind::Inequality(Inequality {
                left: *left,
                operator: InequalityOperator::Less,
                right: *right,
            })),
            Expr::LessEqual(left, right) => out.push(MathSystemKind::Inequality(Inequality {
                left: *left,
                operator: InequalityOperator::LessEqual,
                right: *right,
            })),
            Expr::Greater(left, right) => out.push(MathSystemKind::Inequality(Inequality {
                left: *left,
                operator: InequalityOperator::Greater,
                right: *right,
            })),
            Expr::GreaterEqual(left, right) => out.push(MathSystemKind::Inequality(Inequality {
                left: *left,
                operator: InequalityOperator::GreaterEqual,
                right: *right,
            })),
            Expr::PlotRequirement(items) if items.len() == 1 => {
                out.push(MathSystemKind::PlotRequirement(PlotRequirement {
                    expression: items[0].clone(),
                }))
            }
            other => out.push(MathSystemKind::Expression(Expression { expression: other })),
        }
    }
}
