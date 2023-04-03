use super::{Expression, RawExpression};

#[derive(Debug, PartialEq)]
pub struct ArrayLiteralExpression {
    pub literal: Vec<Expression>,
}

impl ArrayLiteralExpression {
    #[inline]
    pub const fn new(literal: Vec<Expression>) -> Self {
        Self { literal }
    }
}

impl From<ArrayLiteralExpression> for RawExpression {
    fn from(array: ArrayLiteralExpression) -> Self {
        Self::Array(array)
    }
}