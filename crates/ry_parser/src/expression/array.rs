use super::ExpressionParser;
use crate::{error::ParseResult, macros::parse_list, Parser, ParserState};
use ry_ast::{
    expression::{ArrayLiteralExpression, Expression, RawExpression},
    span::{At, Span},
    Token,
};

pub(crate) struct ArrayLiteralExpressionParser;

impl Parser for ArrayLiteralExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let start = state.next.span().start();

        let literal = parse_list!(state, "array literal", Token![']'], || {
            ExpressionParser::default().parse_with(state)
        });

        state.next_token();

        Ok(
            RawExpression::from(ArrayLiteralExpression { literal }).at(Span::new(
                start,
                state.current.span().end(),
                state.file_id,
            )),
        )
    }
}
