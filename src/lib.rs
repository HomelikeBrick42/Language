#![deny(rust_2018_idioms, rust_2024_compatibility)]

pub mod ast;
pub mod lexer;
pub mod parsing;
pub mod pretty_printing;

use lasso::ThreadedRodeo;
use std::sync::LazyLock;

pub static INTERNER: LazyLock<ThreadedRodeo> = LazyLock::new(ThreadedRodeo::new);
