use crate::{error::ParserError, macros::*, Parser, ParserResult};
use num_traits::ToPrimitive;
use ry_ast::{precedence::Precedence, token::RawToken::*, *};

impl<'c> Parser<'c> {
    pub(crate) fn parse_expression(&mut self, precedence: i8) -> ParserResult<Expression> {
        let mut left = self.parse_prefix()?;

        while precedence < self.current.value.to_precedence() {
            left = match &self.current.value {
                binop_pattern!() => {
                    let start = left.span.start;

                    let op = self.current.clone();
                    let precedence = self.current.value.to_precedence();
                    self.advance(false)?; // op

                    let right = self.parse_expression(precedence)?;

                    let end = self.current.span.end;

                    Box::new(RawExpression::Binary(left, op, right)).with_span(start..end)
                }
                OpenParent => {
                    let start = left.span.start;

                    self.advance(false)?; // `(`

                    let arguments =
                        parse_list!(self, "call arguments list", CloseParent, false, || self
                            .parse_expression(Precedence::Lowest.to_i8().unwrap()));

                    let end = self.previous.as_ref().unwrap().span.end;

                    Box::new(RawExpression::Call(vec![], left, arguments)).with_span(start..end)
                }
                Dot => {
                    self.advance(false)?; // `.`

                    if self.current.value.is(LessThan) {
                        let start = left.span.start;

                        let generics = self.parse_type_generic_part()?;

                        check_token!(self, OpenParent, "call")?;

                        self.advance(false)?; // `)`

                        let arguments =
                            parse_list!(self, "generics for call", CloseParent, false, || self
                                .parse_expression(Precedence::Lowest.to_i8().unwrap()));

                        let end = self.previous.as_ref().unwrap().span.end;

                        Box::new(RawExpression::Call(
                            if let Some(v) = generics { v } else { vec![] },
                            left,
                            arguments,
                        ))
                        .with_span(start..end)
                    } else {
                        let start = left.span.start;

                        check_token0!(
                            self,
                            "identifier for property name",
                            Identifier(_),
                            "property"
                        )?;

                        let name = self.current_ident_with_span();
                        let end = self.current.span.end;

                        self.advance(false)?; // id

                        Box::new(RawExpression::Property(left, name)).with_span(start..end)
                    }
                }
                OpenBracket => {
                    let start = left.span.start;

                    self.advance(false)?; // `[`

                    let inner_expr = self.parse_expression(Precedence::Lowest.to_i8().unwrap())?;

                    check_token!(self, CloseBracket, "index")?;

                    let end = self.current.span.end;

                    self.advance(false)?; // `]`

                    Box::new(RawExpression::Index(left, inner_expr)).with_span(start..end)
                }
                postfixop_pattern!() => {
                    let right = self.current.clone();
                    let span = left.span.start..self.current.span.end;

                    self.advance(false)?;

                    Box::new(RawExpression::PrefixOrPostfix(right, left)).with_span(span)
                }
                As => {
                    self.advance(false)?; // `as`

                    let r#type = self.parse_type()?;

                    let span = left.span.start..self.previous.as_ref().unwrap().span.end;

                    Box::new(RawExpression::As(left, r#type)).with_span(span)
                }
                _ => break,
            };
        }

        Ok(left)
    }

    pub(crate) fn parse_prefix(&mut self) -> ParserResult<Expression> {
        self.check_scanning_error()?;

        match &self.current.value {
            Int(i) => {
                let value = *i;
                let span = self.current.span;

                self.advance(false)?; // int

                Ok(Box::new(RawExpression::Int(value)).with_span(span))
            }
            Float(f) => {
                let value = *f;
                let span = self.current.span;

                self.advance(false)?; // float

                Ok(Box::new(RawExpression::Float(value)).with_span(span))
            }
            Imag(i) => {
                let value = *i;
                let span = self.current.span;

                self.advance(false)?; // imag

                Ok(Box::new(RawExpression::Imag(value)).with_span(span))
            }
            String(s) => {
                let value = s.to_owned();
                let span = self.current.span;

                self.advance(false)?; // string

                Ok(Box::new(RawExpression::String(value)).with_span(span))
            }
            Char(c) => {
                let value = *c;
                let span = self.current.span;

                self.advance(false)?; // char

                Ok(Box::new(RawExpression::Char(value)).with_span(span))
            }
            Bool(b) => {
                let value = *b;
                let span = self.current.span;

                self.advance(false)?; // bool

                Ok(Box::new(RawExpression::Bool(value)).with_span(span))
            }
            Bang | Not | PlusPlus | MinusMinus | Minus | Plus => {
                let left = self.current.clone();
                let start = left.span.start;
                self.advance(false)?; // left

                let expr = self.parse_expression(Precedence::PrefixOrPostfix.to_i8().unwrap())?;
                let end = expr.span.end;

                Ok(Box::new(RawExpression::PrefixOrPostfix(left, expr)).with_span(start..end))
            }
            OpenParent => {
                self.advance(false)?; // `(`

                let expr = self.parse_expression(Precedence::Lowest.to_i8().unwrap())?;

                check_token!(self, CloseParent, "parenthesized expression")?;

                self.advance(false)?; // `)`

                Ok(expr)
            }
            OpenBracket => {
                let start = self.current.span.start;
                self.advance(false)?; // `[`

                let list = parse_list!(self, "list literal", CloseBracket, false, || {
                    self.parse_expression(Precedence::Lowest.to_i8().unwrap())
                });

                let end = self.previous.as_ref().unwrap().span.end;

                Ok(Box::new(RawExpression::List(list)).with_span(start..end))
            }
            Identifier(_) => {
                let n = self.parse_name()?;

                Ok(Box::new(RawExpression::StaticName(n.value)).with_span(n.span))
            }
            If => {
                let start = self.current.span.start;
                self.advance(false)?;

                let if_condition = self.parse_expression(Precedence::Lowest.to_i8().unwrap())?;
                let if_statements_block = self.parse_statements_block(false)?;

                let mut else_statements_block = None;
                let mut else_if_chains = vec![];

                while self.current.value.is(Else) {
                    self.advance(false)?; // `else`

                    if !self.current.value.is(If) {
                        else_statements_block = Some(self.parse_statements_block(false)?);
                        break;
                    }

                    self.advance(false)?; // `if`

                    let else_if_condition =
                        self.parse_expression(Precedence::Lowest.to_i8().unwrap())?;

                    let else_if_statements_block = self.parse_statements_block(false)?;

                    else_if_chains.push((else_if_condition, else_if_statements_block));
                }

                let end = self.current.span.end;

                Ok(Box::new(RawExpression::If(
                    (if_condition, if_statements_block),
                    else_if_chains,
                    else_statements_block,
                ))
                .with_span(start..end))
            }
            While => {
                let start = self.current.span.start;

                self.advance(false)?; // `while`

                let condition = self.parse_expression(Precedence::Lowest.to_i8().unwrap())?;
                let block = self.parse_statements_block(false)?;

                let end = self.current.span.end;

                Ok(Box::new(RawExpression::While(condition, block)).with_span(start..end))
            }
            _ => Err(ParserError::UnexpectedToken(
                self.current.clone(),
                "expression".to_owned(),
                None,
            )),
        }
    }
}
