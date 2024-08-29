#![deny(rust_2018_idioms, rust_2024_compatibility)]

pub mod ast;
pub mod lexer;
pub mod parsing;
pub mod pretty_printing;

use lasso::{Spur, ThreadedRodeo};
use rustc_hash::FxBuildHasher;
use std::{ops::Deref, sync::OnceLock};

pub struct Interner(());

type InternerType = ThreadedRodeo<Spur, FxBuildHasher>;
impl Deref for Interner {
    type Target = InternerType;

    fn deref(&self) -> &Self::Target {
        static INTERNER: OnceLock<InternerType> = OnceLock::new();
        INTERNER.get_or_init(|| ThreadedRodeo::with_hasher(FxBuildHasher))
    }
}

pub static INTERNER: Interner = Interner(());
