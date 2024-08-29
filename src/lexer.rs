use crate::INTERNER;
use derive_more::derive::Display;
use lasso::Spur;
use std::{iter::Peekable, num::NonZero, str::CharIndices};
use thiserror::Error;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
#[display("{}:{line}:{column}", &INTERNER[*filepath])]
pub struct Location {
    pub filepath: Spur,
    pub position: usize,
    pub line: NonZero<usize>, // TODO: replace this with some sort of span map
    pub column: NonZero<usize>,
}

#[derive(Debug, Display, Clone, PartialEq, Eq)]
pub enum TokenKind {
    #[display("{{end of file}}")]
    EOF,
    #[display("{}", &INTERNER[*_0])]
    Name(Spur),
    #[display("{_0}")]
    Integer(u64),
    #[display("let")]
    Let,
    #[display("fn")]
    Fn,
    #[display("return")]
    Return,
    #[display("(")]
    OpenParenthesis,
    #[display(")")]
    CloseParenthesis,
    #[display("{{")]
    OpenBrace,
    #[display("}}")]
    CloseBrace,
    #[display(",")]
    Comma,
    #[display(":")]
    Colon,
    #[display(";")]
    Semicolon,
    #[display("=")]
    Equals,
    #[display("+")]
    Plus,
    #[display("-")]
    Minus,
    #[display("*")]
    Asterisk,
    #[display("/")]
    Slash,
    #[display("->")]
    RightArrow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub location: Location,
}

#[derive(Debug, Error)]
pub enum LexerErrorKind {
    #[error("Unexpected character '{0}'")]
    UnexpectedChar(char),
    #[error("Integer literal is too large")]
    IntegerTooLarge,
    #[error("Digit of base {base} integer is too large")]
    DigitTooLarge { base: u8 },
}

#[derive(Debug, Error)]
#[error("{location}: {kind}")]
pub struct LexerError {
    pub kind: LexerErrorKind,
    pub location: Location,
}

#[derive(Debug, Clone)]
pub struct Lexer<'source> {
    location: Location,
    source: &'source str,
    chars: Peekable<CharIndices<'source>>,
}

impl<'source> Lexer<'source> {
    pub fn new(filepath: Spur, source: &'source str) -> Self {
        Self {
            location: Location {
                filepath,
                position: 0,
                line: NonZero::<usize>::MIN,
                column: NonZero::<usize>::MIN,
            },
            source,
            chars: source.char_indices().peekable(),
        }
    }

    pub fn location(&self) -> Location {
        self.location
    }

    pub fn peek_char(&self) -> Option<char> {
        self.source[self.location.position..].chars().next()
    }

    pub fn next_char(&mut self) -> Option<char> {
        let (pos, c) = self.chars.next()?;
        debug_assert_eq!(pos, self.location.position);

        self.location.position = self
            .chars
            .peek()
            .map(|&(pos, _)| pos)
            .unwrap_or(self.source.len());

        self.location.column = self.location.column.saturating_add(1);
        if c == '\n' {
            self.location.line = self.location.line.saturating_add(1);
            self.location.column = NonZero::<usize>::MIN;
        }

        Some(c)
    }

    pub fn peek_token(&self) -> Result<Token, LexerError> {
        self.clone().next_token()
    }

    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        loop {
            let start_location = self.location;
            break Ok(Token {
                location: start_location,
                kind: match self.next_char() {
                    None => TokenKind::EOF,

                    Some('(') => TokenKind::OpenParenthesis,
                    Some(')') => TokenKind::CloseParenthesis,
                    Some('{') => TokenKind::OpenBrace,
                    Some('}') => TokenKind::CloseBrace,
                    Some(',') => TokenKind::Comma,
                    Some(':') => TokenKind::Colon,
                    Some(';') => TokenKind::Semicolon,
                    Some('=') => TokenKind::Equals,
                    Some('+') => TokenKind::Plus,
                    Some('-') => {
                        if let Some('>') = self.peek_char() {
                            self.next_char();
                            TokenKind::RightArrow
                        } else {
                            TokenKind::Minus
                        }
                    }
                    Some('*') => TokenKind::Asterisk,
                    Some('/') => TokenKind::Slash,

                    Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                        while self
                            .peek_char()
                            .map_or(false, |c| c.is_ascii_alphanumeric() || c == '_')
                        {
                            self.next_char();
                        }

                        match &self.source[start_location.position..self.location.position] {
                            "let" => TokenKind::Let,
                            "fn" => TokenKind::Fn,
                            "return" => TokenKind::Return,
                            name => TokenKind::Name(INTERNER.get_or_intern(name)),
                        }
                    }

                    Some(c) if c.is_ascii_digit() => {
                        let mut value = c.to_digit(10).unwrap() as u64;
                        let base = if c == '0' {
                            match self.peek_char() {
                                Some('x') => {
                                    self.next_char();
                                    16
                                }
                                Some('d') => {
                                    self.next_char();
                                    10
                                }
                                Some('o') => {
                                    self.next_char();
                                    8
                                }
                                Some('b') => {
                                    self.next_char();
                                    2
                                }
                                _ => 10,
                            }
                        } else {
                            10
                        };

                        while let Some(c) = self.peek_char().filter(|c| c.is_ascii_alphanumeric()) {
                            let digit = c.to_digit(base as _).ok_or(LexerError {
                                kind: LexerErrorKind::DigitTooLarge { base },
                                location: self.location,
                            })?;

                            self.next_char();

                            value = value
                                .checked_mul(base as _)
                                .and_then(|value| value.checked_add(digit as _))
                                .ok_or(LexerError {
                                    kind: LexerErrorKind::IntegerTooLarge,
                                    location: start_location,
                                })?;
                        }

                        TokenKind::Integer(value)
                    }

                    Some(c) if c.is_whitespace() => continue,
                    Some(c) => {
                        return Err(LexerError {
                            kind: LexerErrorKind::UnexpectedChar(c),
                            location: start_location,
                        });
                    }
                },
            });
        }
    }
}
