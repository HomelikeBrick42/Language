use crate::lexer::{Location, Token, TokenKind};
use derive_more::derive::Display;
use lasso::Spur;
use std::num::NonZero;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstKind {
    Expression(AstExpression),
    Let {
        pattern: AstPattern,
        equals: Location,
        value: Box<AstExpression>,
    },
    Function {
        name: Token,
        arguments: Vec<AstPattern>,
        return_type: Option<Box<AstExpression>>,
        body: AstExpression,
    },
    Return {
        expression: AstExpression,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ast {
    pub kind: AstKind,
    pub location: Location,
}

#[derive(Debug, Display, Clone, PartialEq, Eq)]
pub enum BinaryOperator {
    #[display("+")]
    Add,
    #[display("-")]
    Subtract,
    #[display("*")]
    Multiply,
    #[display("/")]
    Divide,
}

impl BinaryOperator {
    pub fn from_token_kind(kind: TokenKind) -> Option<BinaryOperator> {
        Some(match kind {
            TokenKind::Plus => BinaryOperator::Add,
            TokenKind::Minus => BinaryOperator::Subtract,
            TokenKind::Asterisk => BinaryOperator::Multiply,
            TokenKind::Slash => BinaryOperator::Divide,
            _ => return None,
        })
    }

    pub fn precedence(&self) -> NonZero<u8> {
        macro_rules! l {
            ($l:literal) => {
                const {
                    match NonZero::<u8>::new($l) {
                        Some(l) => l,
                        None => unreachable!(),
                    }
                }
            };
        }

        match *self {
            BinaryOperator::Multiply | BinaryOperator::Divide => l!(2),
            BinaryOperator::Add | BinaryOperator::Subtract => l!(1),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstExpressionKind {
    Name(Spur),
    Integer(u64),
    Binary {
        left: Box<AstExpression>,
        operator: BinaryOperator,
        right: Box<AstExpression>,
    },
    Block {
        statements: Vec<Ast>,
        close_brace: Location,
    },
    Call {
        operand: Box<AstExpression>,
        arguments: Vec<AstExpression>,
        close_parenthesis: Location,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstExpression {
    pub kind: AstExpressionKind,
    pub location: Location,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstPatternKind {
    Let {
        name_token: Token,
        typ: Option<AstExpression>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstPattern {
    pub kind: AstPatternKind,
    pub location: Location,
}
