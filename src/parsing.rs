use crate::{
    ast::{
        Ast, AstExpression, AstExpressionKind, AstKind, AstPattern, AstPatternKind, BinaryOperator,
    },
    interning::InternedStr,
    lexer::{Lexer, LexerError, LexerErrorKind, Location, Token, TokenKind},
};
use std::num::NonZero;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseErrorKind {
    #[error("{0}")]
    LexerError(LexerErrorKind),
    #[error("Unexpected token '{0}'")]
    UnexpectedToken(TokenKind),
    #[error("Expected global item but got '{0}'")]
    ExpectedGlobalItem(TokenKind),
    #[error("Expected expression but got '{0}'")]
    ExpectedExpression(TokenKind),
    #[error("Expected pattern but got '{0}'")]
    ExpectedPattern(TokenKind),
}

#[derive(Debug, Error)]
#[error("{location}: {kind}")]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub location: Location,
}

impl From<LexerError> for ParseError {
    fn from(error: LexerError) -> Self {
        Self {
            kind: ParseErrorKind::LexerError(error.kind),
            location: error.location,
        }
    }
}

pub fn parse(filepath: InternedStr, source: &str) -> Result<Vec<Ast>, ParseError> {
    let lexer = &mut Lexer::new(filepath, source);
    let mut statements = vec![];
    while !matches!(lexer.peek_token()?.kind, TokenKind::EOF) {
        statements.push(parse_global(lexer)?);
    }
    Ok(statements)
}

macro_rules! expect_token {
    ($lexer:expr, $pattern:pat) => {
        match $lexer.next_token() {
            Ok(token @ Token { kind: $pattern, .. }) => Ok(token),
            Ok(token) => Err(ParseError {
                kind: ParseErrorKind::UnexpectedToken(token.kind),
                location: token.location,
            }),
            Err(error) => Err(error.into()),
        }
    };
}

pub fn parse_global(lexer: &mut Lexer<'_>) -> Result<Ast, ParseError> {
    Ok(match lexer.next_token()? {
        Token {
            kind: TokenKind::Fn,
            location,
        } => parse_fn(lexer, location)?,

        Token { kind, location } => {
            return Err(ParseError {
                kind: ParseErrorKind::ExpectedGlobalItem(kind),
                location,
            });
        }
    })
}

pub fn parse_statement(lexer: &mut Lexer<'_>) -> Result<Ast, ParseError> {
    let start_location = lexer.location();
    Ok(match lexer.peek_token()?.kind {
        TokenKind::Fn => {
            let fn_location = expect_token!(lexer, TokenKind::Fn)?.location;
            parse_fn(lexer, fn_location)?
        }

        TokenKind::Let => {
            let pattern = parse_pattern(lexer, true)?;
            let equals = expect_token!(lexer, TokenKind::Equals)?.location;
            let value = Box::new(parse_expression(lexer)?);
            expect_token!(lexer, TokenKind::Semicolon)?;
            Ast {
                kind: AstKind::Let {
                    pattern,
                    equals,
                    value,
                },
                location: start_location,
            }
        }

        TokenKind::Return => {
            expect_token!(lexer, TokenKind::Return)?;
            let expression = parse_expression(lexer)?;
            expect_token!(lexer, TokenKind::Semicolon)?;
            Ast {
                kind: AstKind::Return { expression },
                location: start_location,
            }
        }

        _ => {
            let expression = parse_expression(lexer)?;
            expect_token!(lexer, TokenKind::Semicolon)?;
            Ast {
                kind: AstKind::Expression(expression),
                location: start_location,
            }
        }
    })
}

pub fn parse_fn(lexer: &mut Lexer<'_>, fn_location: Location) -> Result<Ast, ParseError> {
    let name = expect_token!(lexer, TokenKind::Name(_))?;

    expect_token!(lexer, TokenKind::OpenParenthesis)?;
    let mut arguments = vec![];
    while !matches!(lexer.peek_token()?.kind, TokenKind::CloseParenthesis) {
        arguments.push(parse_pattern(lexer, false)?);
        if !matches!(lexer.peek_token()?.kind, TokenKind::CloseParenthesis) {
            expect_token!(lexer, TokenKind::Comma)?;
        }
    }
    expect_token!(lexer, TokenKind::CloseParenthesis)?;

    let return_type = if let TokenKind::RightArrow = lexer.peek_token()?.kind {
        expect_token!(lexer, TokenKind::RightArrow)?;
        Some(Box::new(parse_expression(lexer)?))
    } else {
        None
    };

    let body = parse_block(lexer, None)?;

    Ok(Ast {
        kind: AstKind::Function {
            name,
            arguments,
            return_type,
            body,
        },
        location: fn_location,
    })
}

pub fn parse_primary_expression(lexer: &mut Lexer<'_>) -> Result<AstExpression, ParseError> {
    Ok(match lexer.next_token()? {
        Token {
            kind: TokenKind::Integer(value),
            location,
        } => AstExpression {
            kind: AstExpressionKind::Integer(value),
            location,
        },

        Token {
            kind: TokenKind::Name(name),
            location,
        } => AstExpression {
            kind: AstExpressionKind::Name(name),
            location,
        },

        Token {
            kind: TokenKind::OpenParenthesis,
            ..
        } => {
            let expression = parse_expression(lexer)?;
            expect_token!(lexer, TokenKind::CloseParenthesis)?;
            expression
        }

        Token {
            kind: TokenKind::OpenBrace,
            location,
        } => parse_block(lexer, Some(location))?,

        Token { kind, location } => {
            return Err(ParseError {
                kind: ParseErrorKind::ExpectedExpression(kind),
                location,
            });
        }
    })
}

pub fn parse_binary_expression(
    lexer: &mut Lexer<'_>,
    parent_precedence: Option<NonZero<u8>>,
) -> Result<AstExpression, ParseError> {
    let mut left = parse_primary_expression(lexer)?;

    loop {
        left = if let Some(operator) = BinaryOperator::from_token_kind(lexer.peek_token()?.kind) {
            let precedence = operator.precedence();
            if parent_precedence.map_or(false, |parent_precedence| precedence <= parent_precedence)
            {
                break;
            }

            let location = lexer.next_token()?.location;
            let right = Box::new(parse_binary_expression(lexer, Some(precedence))?);
            AstExpression {
                kind: AstExpressionKind::Binary {
                    left: Box::new(left),
                    operator,
                    right,
                },
                location,
            }
        } else if let TokenKind::OpenParenthesis = lexer.peek_token()?.kind {
            let location = expect_token!(lexer, TokenKind::OpenParenthesis)?.location;
            let mut arguments = vec![];
            while !matches!(lexer.peek_token()?.kind, TokenKind::CloseParenthesis) {
                arguments.push(parse_expression(lexer)?);
                if !matches!(lexer.peek_token()?.kind, TokenKind::CloseParenthesis) {
                    expect_token!(lexer, TokenKind::Comma)?;
                }
            }
            let close_parenthesis = expect_token!(lexer, TokenKind::CloseParenthesis)?.location;

            AstExpression {
                kind: AstExpressionKind::Call {
                    operand: Box::new(left),
                    arguments,
                    close_parenthesis,
                },
                location,
            }
        } else {
            break;
        };
    }

    Ok(left)
}

pub fn parse_expression(lexer: &mut Lexer<'_>) -> Result<AstExpression, ParseError> {
    parse_binary_expression(lexer, None)
}

pub fn parse_block(
    lexer: &mut Lexer<'_>,
    open_brace_location: Option<Location>,
) -> Result<AstExpression, ParseError> {
    let location = if let Some(location) = open_brace_location {
        location
    } else {
        expect_token!(lexer, TokenKind::OpenBrace)?.location
    };
    let mut statements = vec![];
    while !matches!(lexer.peek_token()?.kind, TokenKind::CloseBrace) {
        statements.push(parse_statement(lexer)?);
    }
    let close_brace = expect_token!(lexer, TokenKind::CloseBrace)?.location;
    Ok(AstExpression {
        kind: AstExpressionKind::Block {
            statements,
            close_brace,
        },
        location,
    })
}

pub fn parse_pattern(lexer: &mut Lexer<'_>, requires_let: bool) -> Result<AstPattern, ParseError> {
    Ok(match lexer.next_token()? {
        Token {
            kind: TokenKind::Let,
            location,
        } => AstPattern {
            location,
            kind: AstPatternKind::Let {
                name_token: expect_token!(lexer, TokenKind::Name(_))?,
                typ: if let TokenKind::Colon = lexer.peek_token()?.kind {
                    lexer.next_token()?;
                    Some(parse_expression(lexer)?)
                } else {
                    None
                },
            },
        },

        name_token @ Token {
            kind: TokenKind::Name(_),
            location,
        } if !requires_let => AstPattern {
            location,
            kind: AstPatternKind::Let {
                name_token,
                typ: if let TokenKind::Colon = lexer.peek_token()?.kind {
                    lexer.next_token()?;
                    Some(parse_expression(lexer)?)
                } else {
                    None
                },
            },
        },

        Token { kind, location } => {
            return Err(ParseError {
                kind: ParseErrorKind::ExpectedPattern(kind),
                location,
            });
        }
    })
}
