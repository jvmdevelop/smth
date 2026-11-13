mod ast;
mod input;
mod parser;
mod runtime;
mod systems;

pub use ast::Expr;
pub use input::{MathInput, msystem, msystem_file, msystem_text};
pub use runtime::{
    Basis, ConditionValidator, ExecutionContext, ExecutionResult, Plot, PlotKind, PlotSettings,
    SystemExecution, VariableRegistry,
};
pub use systems::{
    Equality, Equation, Expression, Inequality, InequalityOperator, MathSystem, MathSystemKind,
    MathSystems, PlotRequirement,
};
