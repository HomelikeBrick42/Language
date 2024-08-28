use derive_more::derive::Display;

use crate::lexer::{Location, Token, TokenKind};
use std::num::NonZero;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstKind<'filepath, 'source> {
    Expression(AstExpression<'filepath, 'source>),
    Let {
        pattern: AstPattern<'filepath, 'source>,
        equals: Location<'filepath>,
        value: Box<AstExpression<'filepath, 'source>>,
    },
    Function {
        name: Token<'filepath, 'source>,
        arguments: Vec<AstPattern<'filepath, 'source>>,
        return_type: Option<Box<AstExpression<'filepath, 'source>>>,
        body: AstExpression<'filepath, 'source>,
    },
    Return {
        expression: AstExpression<'filepath, 'source>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ast<'filepath, 'source> {
    pub kind: AstKind<'filepath, 'source>,
    pub location: Location<'filepath>,
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
    pub fn from_token_kind(kind: TokenKind<'_>) -> Option<BinaryOperator> {
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
pub enum AstExpressionKind<'filepath, 'source> {
    Name(&'source str),
    Integer(u64),
    Binary {
        left: Box<AstExpression<'filepath, 'source>>,
        operator: BinaryOperator,
        right: Box<AstExpression<'filepath, 'source>>,
    },
    Block {
        statements: Vec<Ast<'filepath, 'source>>,
        close_brace: Location<'filepath>,
    },
    Call {
        operand: Box<AstExpression<'filepath, 'source>>,
        arguments: Vec<AstExpression<'filepath, 'source>>,
        close_parenthesis: Location<'filepath>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstExpression<'filepath, 'source> {
    pub kind: AstExpressionKind<'filepath, 'source>,
    pub location: Location<'filepath>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstPatternKind<'filepath, 'source> {
    Let {
        name_token: Token<'filepath, 'source>,
        typ: Option<AstExpression<'filepath, 'source>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstPattern<'filepath, 'source> {
    pub kind: AstPatternKind<'filepath, 'source>,
    pub location: Location<'filepath>,
}
