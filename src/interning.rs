use derive_more::derive::{Debug, Display};
use lasso::{Key, ThreadedRodeo};
use rustc_hash::FxBuildHasher;
use std::{num::NonZero, ops::Deref, sync::OnceLock};

type IdType = u32;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash)]
#[display("{}", self as &str)]
#[debug("{:?}", self as &str)]
pub struct InternedStr(NonZero<IdType>);

impl From<&str> for InternedStr {
    fn from(s: &str) -> Self {
        INTERNER
            .get_or_init(|| ThreadedRodeo::with_hasher(FxBuildHasher))
            .get_or_intern(s)
    }
}

impl Deref for InternedStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        // Safety: if this type is constructed then INTERNER should be initialized
        unsafe { &INTERNER.get().unwrap_unchecked()[*self] }
    }
}

unsafe impl Key for InternedStr {
    fn into_usize(self) -> usize {
        // Safety: the only way this type is constructed is through `Self::try_from_usize`, so the value must fit in a usize
        unsafe { (self.0.get() ^ IdType::MAX).try_into().unwrap_unchecked() }
    }

    fn try_from_usize(int: usize) -> Option<Self> {
        Some(InternedStr(NonZero::new(
            IdType::try_from(int).ok()? ^ IdType::MAX,
        )?))
    }
}

static INTERNER: OnceLock<ThreadedRodeo<InternedStr, FxBuildHasher>> = OnceLock::new();
