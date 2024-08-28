use crate::{
    ast::{Ast, AstExpression, AstExpressionKind, AstKind, AstPattern, AstPatternKind},
    lexer::TokenKind,
    INTERNER,
};
use std::io::{Result, Write};

pub fn pretty_print_ast(
    ast: &Ast,
    indent: usize,
    writer: &mut (impl Write + ?Sized),
) -> Result<()> {
    print_indent(indent, writer)?;
    match ast.kind {
        AstKind::Expression(ref expression) => {
            pretty_print_ast_expression(expression, indent, writer)?;
            writeln!(writer, ";")?;
        }
        AstKind::Let {
            ref pattern,
            ref value,
            ..
        } => {
            write!(writer, "let ")?;
            pretty_print_ast_pattern(pattern, indent, writer)?;
            write!(writer, " = ")?;
            pretty_print_ast_expression(value, indent, writer)?;
            writeln!(writer, ";")?;
        }
        AstKind::Function {
            ref name,
            ref arguments,
            ref return_type,
            ref body,
        } => {
            write!(writer, "fn {}(", name.kind)?;
            for (i, argument) in arguments.iter().enumerate() {
                if i > 0 {
                    write!(writer, ", ")?;
                }
                pretty_print_ast_pattern(argument, indent, writer)?;
            }
            write!(writer, ")")?;
            if let Some(return_type) = return_type {
                write!(writer, " -> ")?;
                pretty_print_ast_expression(return_type, indent, writer)?;
            }
            write!(writer, " ")?;
            pretty_print_ast_expression(body, indent, writer)?;
            writeln!(writer)?;
        }
        AstKind::Return { ref expression } => {
            write!(writer, "return ")?;
            pretty_print_ast_expression(expression, indent, writer)?;
            writeln!(writer, ";")?;
        }
    }
    Ok(())
}

pub fn pretty_print_ast_expression(
    expression: &AstExpression,
    indent: usize,
    writer: &mut (impl Write + ?Sized),
) -> Result<()> {
    match expression.kind {
        AstExpressionKind::Name(name) => write!(writer, "{}", &INTERNER[name])?,
        AstExpressionKind::Integer(value) => write!(writer, "{value}")?,
        AstExpressionKind::Binary {
            ref left,
            ref operator,
            ref right,
        } => {
            write!(writer, "(")?;
            pretty_print_ast_expression(left, indent, writer)?;
            write!(writer, " {operator} ")?;
            pretty_print_ast_expression(right, indent, writer)?;
            write!(writer, ")")?;
        }
        AstExpressionKind::Block {
            ref statements,
            close_brace: _,
        } => {
            writeln!(writer, "{{")?;
            for statement in statements {
                pretty_print_ast(statement, indent + 1, writer)?;
            }
            print_indent(indent, writer)?;
            write!(writer, "}}")?;
        }
        AstExpressionKind::Call {
            ref operand,
            ref arguments,
            close_parenthesis: _,
        } => {
            pretty_print_ast_expression(operand, indent, writer)?;
            write!(writer, "(")?;
            for (i, argument) in arguments.iter().enumerate() {
                if i > 0 {
                    write!(writer, ", ")?;
                }
                pretty_print_ast_expression(argument, indent, writer)?;
            }
            write!(writer, ")")?;
        }
    }
    Ok(())
}

pub fn pretty_print_ast_pattern(
    pattern: &AstPattern,
    indent: usize,
    writer: &mut (impl Write + ?Sized),
) -> Result<()> {
    match pattern.kind {
        AstPatternKind::Let {
            ref name_token,
            ref typ,
        } => {
            let TokenKind::Name(name) = name_token.kind else {
                unreachable!();
            };
            write!(writer, "{}", &INTERNER[name])?;
            if let Some(typ) = typ {
                write!(writer, ": ")?;
                pretty_print_ast_expression(typ, indent, writer)?;
            }
        }
    }
    Ok(())
}

fn print_indent(indent: usize, writer: &mut (impl Write + ?Sized)) -> Result<()> {
    for _ in 0..indent {
        write!(writer, "    ")?;
    }
    Ok(())
}
