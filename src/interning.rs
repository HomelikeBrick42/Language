use derive_more::derive::{Debug, Display};
use lasso::{Spur, ThreadedRodeo};
use rustc_hash::FxBuildHasher;
use std::sync::OnceLock;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash)]
#[display("{}", self.to_str())]
#[debug("{:?}", self.to_str())]
pub struct InternedStr(Spur);

impl InternedStr {
    pub fn intern(s: &str) -> Self {
        InternedStr(
            INTERNER
                .get_or_init(|| ThreadedRodeo::with_hasher(FxBuildHasher))
                .get_or_intern(s),
        )
    }

    pub fn to_str(self) -> &'static str {
        // SAFETY: if this type is constructed, then `INTERNER` has already been initialized by `Self::intern`
        let interner = unsafe { INTERNER.get().unwrap_unchecked() };

        // SAFETY: if this type has been constructed, then `self.0` was retrieved from `INTERNER` so it will be resolved
        unsafe { interner.try_resolve(&self.0).unwrap_unchecked() }
    }
}

impl From<&str> for InternedStr {
    fn from(s: &str) -> Self {
        Self::intern(s)
    }
}

static INTERNER: OnceLock<ThreadedRodeo<Spur, FxBuildHasher>> = OnceLock::new();
