use std::{mem::transmute, ops::Deref};

use cstree::interning::TokenKey;

use crate::{span::Spanned, Interner, Span};

pub type Word = TokenKey;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(transparent)]
pub struct FileId(TokenKey);

impl FileId {
    // This should only be used when you know for certain the file id won't end up being used (the diagnostic won't be reported)
    pub unsafe fn poisoned() -> Self {
        Self(transmute(u32::MAX))
    }

    pub fn new(key: TokenKey) -> Self {
        Self(key)
    }

    pub fn key(&self) -> &TokenKey {
        &self.0
    }

    pub fn prelude(interner: &'static Interner) -> Self {
        Self(interner.get_or_intern_static("<~~~prelude~~~>"))
    }

    pub fn as_str(&self, interner: &'static Interner) -> &str {
        interner.resolve(&self.0)
    }
}

#[derive(Debug, Clone)]
pub struct InFile<T> {
    pub inner: T,
    pub file_id: FileId,
}

impl<T> Deref for InFile<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> InFile<T> {
    pub fn new(inner: T, file_id: FileId) -> Self {
        Self { inner, file_id }
    }

    /// Maps the inner value of an [`InFile`]
    ///
    /// `InFile<A>` -> `InFile<B>`
    pub fn map<F, B>(self, f: F) -> InFile<B>
    where
        F: FnOnce(T) -> B,
    {
        InFile::new(f(self.inner), self.file_id)
    }

    /// Maps the inner value of an [`InFile`] passing the values to the closure by reference
    ///
    /// `InFile<A>` -> `InFile<B>`
    pub fn map_ref<F, B>(&self, f: F) -> InFile<B>
    where
        F: FnOnce(&T) -> B,
    {
        InFile::new(f(&self.inner), self.file_id)
    }
}

pub type FileSpanned<T> = InFile<Spanned<T>>;

impl Copy for InFile<Span> {}

impl InFile<Span> {
    pub fn to_file_spanned<T>(&self, inner: T) -> FileSpanned<T> {
        FileSpanned::new(Spanned::new(inner, self.inner), self.file_id)
    }

    pub fn to_file_span(&self) -> FileSpan {
        FileSpan {
            file_id: self.file_id,
            span: self.inner,
        }
    }
}

impl<T> InFile<Spanned<T>> {
    pub fn to_file_span(&self) -> InFile<Span> {
        InFile::new(self.span, self.file_id)
    }

    pub fn map_inner<F, B>(self, f: F) -> InFile<Spanned<B>>
    where
        F: FnOnce(T) -> B,
    {
        InFile::new(self.inner.map(|v| f(v)), self.file_id)
    }

    /// Maps the inner value of an `InFile<Spanned<T>` passing the values to the closure by reference
    ///
    /// `InFile<Spanned<A>>` -> `InFile<Spanned<B>>`
    pub fn map_inner_ref<F, B>(&self, f: F) -> InFile<Spanned<B>>
    where
        F: FnOnce(&T) -> B,
    {
        InFile::new(self.inner.map_ref(|v| f(v)), self.file_id)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FileSpan {
    pub file_id: FileId,
    pub span: Span,
}

impl FileSpan {
    pub fn new(file_id: FileId, span: Span) -> Self {
        Self { file_id, span }
    }
}
