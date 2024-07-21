use std::num::NonZeroUsize;

use flux_diagnostics::ice;
use flux_span::Word;

use crate::{r#type::TypeId, FnSignature};

#[derive(Debug, Clone, Default)]
pub struct Trait {
    signatures: Vec<FnSignature>,
}

impl Trait {
    pub fn new(signatures: Vec<FnSignature>) -> Self {
        Self { signatures }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TraitId(NonZeroUsize);

impl TraitId {
    pub fn new(id: usize) -> Self {
        Self(NonZeroUsize::new(id).unwrap_or_else(|| ice("cannot create `TraitId` with value 0")))
    }

    pub fn raw(&self) -> usize {
        self.0.into()
    }
}

#[derive(Debug, Clone)]
pub struct Application {
    pub tid: TypeId,
    pub assoc_types: Vec<(Word, TypeId)>,
    pub signatures: Vec<FnSignature>,
}

impl Application {
    pub fn new(
        tid: TypeId,
        assoc_types: Vec<(Word, TypeId)>,
        signatures: Vec<FnSignature>,
    ) -> Self {
        Self {
            tid,
            assoc_types,
            signatures,
        }
    }
}

// ApplicationId is wrapped in `Option` in `ThisCtx`, so making it `NonZero` is advantageous. Waste one space in the vector in `TEnv`, but save multiple bytes in each instance of `ThisCtx`
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ApplicationId(NonZeroUsize);

impl ApplicationId {
    // SAFETY: it's obviously all good chill out
    // const UNSET: Self = Self(unsafe { NonZeroUsize::new_unchecked(usize::MAX) });

    pub fn new(id: usize) -> Self {
        Self(
            NonZeroUsize::new(id)
                .unwrap_or_else(|| ice("cannot create `ApplicationId` with value 0")),
        )
    }

    pub fn raw(&self) -> usize {
        self.0.into()
    }
}

#[derive(Debug, Clone)]
pub struct TraitRestriction {
    pub absolute_path: Vec<Word>,
    pub args: Vec<TypeId>,
}

impl TraitRestriction {
    pub fn new(absolute_path: Vec<Word>, args: Vec<TypeId>) -> Self {
        Self {
            absolute_path,
            args,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct ThisCtx {
    pub trait_id: Option<TraitId>,
    pub application_id: Option<ApplicationId>,
}

impl ThisCtx {
    pub fn new(trait_id: Option<TraitId>, application_id: Option<ApplicationId>) -> Self {
        Self {
            trait_id,
            application_id,
        }
    }

    pub const fn unset() -> Self {
        Self {
            trait_id: None,
            application_id: None,
        }
    }
}
