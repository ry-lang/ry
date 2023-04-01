//! Defines `Expression` AST Node, as defined by the [spec].
use super::Statement;
use crate::expression::Expression;

/// The `Expression` AST Node.
///
/// See the [module level documentation][self].
#[derive(Debug, PartialEq)]
pub struct ExpressionStatement {
    has_semicolon: bool,
    expression: Expression,
}

impl ExpressionStatement {
    #[inline]
    pub const fn new(has_semicolon: bool, expression: Expression) -> Self {
        Self {
            has_semicolon,
            expression,
        }
    }

    #[inline]
    pub const fn has_semicolon(&self) -> bool {
        self.has_semicolon
    }

    #[inline]
    pub const fn expression(&self) -> &Expression {
        &self.expression
    }
}

impl From<ExpressionStatement> for Statement {
    fn from(expression: ExpressionStatement) -> Self {
        Self::Expression(expression)
    }
}
