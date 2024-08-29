use derive_more::derive::{Debug, Display};
use lasso::{Spur, ThreadedRodeo};
use rustc_hash::FxBuildHasher;
use std::{ops::Deref, sync::OnceLock};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash)]
#[display("{}", self as &str)]
#[debug("{:?}", self as &str)]
pub struct InternedStr(Spur);

impl From<&str> for InternedStr {
    fn from(s: &str) -> Self {
        InternedStr(
            INTERNER
                .get_or_init(|| ThreadedRodeo::with_hasher(FxBuildHasher))
                .get_or_intern(s),
        )
    }
}

impl Deref for InternedStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        // SAFETY: if this type is constructed, then `INTERNER` has already been initialized by `Self::from`
        let interner = unsafe { INTERNER.get().unwrap_unchecked() };

        // SAFETY: if this type has been constructed, then `self.0` was retrieved from `INTERNER` so it will be resolved
        unsafe { interner.try_resolve(&self.0).unwrap_unchecked() }
    }
}

static INTERNER: OnceLock<ThreadedRodeo<Spur, FxBuildHasher>> = OnceLock::new();
