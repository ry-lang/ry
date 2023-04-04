use super::Item;
use crate::{
    expression::Expression,
    name::Name,
    r#type::{generics::Generics, where_clause::WhereClause, Type},
    statement::StatementsBlock,
    Visibility,
};

#[derive(Debug, PartialEq)]
pub struct FunctionDeclaration {
    pub definition: FunctionDefinition,
    pub body: StatementsBlock,
}

impl From<Function> for Item {
    fn from(function: Function) -> Self {
        Self::Function(function)
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionDefinition {
    pub visibility: Visibility,
    pub name: Name,
    pub generics: Generics,
    pub arguments: Vec<FunctionArgument>,
    pub return_type: Option<Type>,
    pub r#where: WhereClause,
}

#[derive(Debug, PartialEq)]
pub struct FunctionArgument {
    pub name: Name,
    pub r#type: Type,
    pub default_value: Option<Expression>,
}

#[derive(Debug, PartialEq)]
pub enum Function {
    Definition(FunctionDefinition),
    Declaration(FunctionDeclaration),
}

impl From<FunctionDefinition> for Function {
    fn from(definition: FunctionDefinition) -> Self {
        Self::Definition(definition)
    }
}

impl From<FunctionDeclaration> for Function {
    fn from(declaration: FunctionDeclaration) -> Self {
        Self::Declaration(declaration)
    }
}

pub type Method = Function;
