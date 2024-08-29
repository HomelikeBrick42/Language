use derive_more::derive::{Debug, Display};
use lasso::{Key, ThreadedRodeo};
use rustc_hash::FxBuildHasher;
use std::{ops::Deref, sync::OnceLock};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[display("{}", INTERNER[*self])]
#[debug("{:?}", INTERNER[*self])]
pub struct InternedStr(usize);

impl From<&str> for InternedStr {
    fn from(s: &str) -> Self {
        INTERNER.get_or_intern(s)
    }
}

impl Deref for InternedStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &INTERNER[*self]
    }
}

unsafe impl Key for InternedStr {
    fn into_usize(self) -> usize {
        self.0
    }

    fn try_from_usize(int: usize) -> Option<Self> {
        Some(InternedStr(int))
    }
}

struct Interner(());

type InternerType = ThreadedRodeo<InternedStr, FxBuildHasher>;
impl Deref for Interner {
    type Target = InternerType;

    fn deref(&self) -> &Self::Target {
        static INTERNER: OnceLock<InternerType> = OnceLock::new();
        INTERNER.get_or_init(|| ThreadedRodeo::with_hasher(FxBuildHasher))
    }
}

static INTERNER: Interner = Interner(());
