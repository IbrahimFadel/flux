use std::ops::Deref;

use crate::{span::Spanned, word::Word, Span};

#[salsa::input]
pub struct InputFile {
    pub name: Word,

    #[return_ref]
    pub source_text: String,
}

#[derive(Debug, Clone)]
pub struct InFile<T> {
    pub inner: T,
    pub file: InputFile,
}

impl<T> Deref for InFile<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> InFile<T> {
    pub fn new(inner: T, file: InputFile) -> Self {
        Self { inner, file }
    }

    /// Maps the inner value of an [`InFile`]
    ///
    /// `InFile<A>` -> `InFile<B>`
    pub fn map<F, B>(self, f: F) -> InFile<B>
    where
        F: FnOnce(T) -> B,
    {
        InFile::new(f(self.inner), self.file)
    }

    /// Maps the inner value of an [`InFile`] passing the values to the closure by reference
    ///
    /// `InFile<A>` -> `InFile<B>`
    pub fn map_ref<F, B>(&self, f: F) -> InFile<B>
    where
        F: FnOnce(&T) -> B,
    {
        InFile::new(f(&self.inner), self.file)
    }
}

pub type FileSpanned<T> = InFile<Spanned<T>>;

impl InFile<Span> {
    pub fn to_file_spanned<T>(&self, inner: T) -> FileSpanned<T> {
        FileSpanned::new(Spanned::new(inner, self.inner), self.file)
    }

    pub fn to_file_span(&self) -> FileSpan {
        FileSpan {
            input_file: self.file,
            span: self.inner,
        }
    }
}

impl<T> InFile<Spanned<T>> {
    pub fn to_file_span(&self) -> FileSpan {
        FileSpan {
            input_file: self.file,
            span: self.span,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FileSpan {
    pub input_file: InputFile,
    pub span: Span,
}

impl FileSpan {
    pub fn new(input_file: InputFile, span: Span) -> Self {
        Self { input_file, span }
    }
}
