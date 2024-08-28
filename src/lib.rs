#![deny(rust_2018_idioms, rust_2024_compatibility)]

pub mod ast;
pub mod lexer;
pub mod parsing;
pub mod pretty_printing;

use lasso::{Spur, ThreadedRodeo};
use rustc_hash::FxBuildHasher;
use std::sync::LazyLock;

pub static INTERNER: LazyLock<ThreadedRodeo<Spur, FxBuildHasher>> =
    LazyLock::new(|| ThreadedRodeo::with_hasher(FxBuildHasher));
