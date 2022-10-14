use std::ops::Deref;

use lasso::Spur;
use text_size::TextRange;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub range: TextRange,
}

impl Span {
    pub fn new<T: Into<std::ops::Range<u32>>>(range: T) -> Self {
        let std::ops::Range { start, end } = range.into();
        let range: TextRange = TextRange::new(start.into(), end.into());
        Self { range }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<T> {
    pub inner: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(inner: T, span: Span) -> Self {
        Self { inner, span }
    }

    /// Convert an iterator of spanned items into a spanned collection of unspanned items
    ///
    /// `[Spanned<T>]` -> `Spanned<[T]>`
    ///
    /// Returns `None` if the iterator has no items, as there can be no span
    pub fn span_iter<C: FromIterator<T>>(
        iter: impl IntoIterator<Item = Spanned<T>>,
    ) -> Option<Spanned<C>> {
        let mut iter = iter.into_iter();
        let first = iter.next()?;
        let start = first.span.range.start();
        let mut end = first.span.range.end();
        let c = C::from_iter(
            std::iter::once(first.inner)
                .chain(iter.inspect(|t| end = t.span.range.end()).map(|v| v.inner)),
        );
        let span = Span::new(TextRange::new(start, end));
        Some(Spanned::new(c, span))
    }
}

impl<A> Spanned<A> {
    /// Maps the inner value of a [`Spanned`]
    ///
    /// `Spanned<A>` -> `Spanned<B>`
    ///
    pub fn map<F, B>(self, f: F) -> Spanned<B>
    where
        F: FnOnce(A) -> B,
    {
        Spanned::new(f(self.inner), self.span)
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct FileId(pub Spur);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InFile<T> {
    pub inner: T,
    pub file_id: FileId,
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
}

impl<A: Clone> InFile<A> {
    /// Maps the inner value of an [`InFile`] cloning the values
    ///
    /// `InFile<A>` -> `InFile<B>`
    pub fn cloned_map<F, B>(&self, f: F) -> InFile<B>
    where
        F: FnOnce(A) -> B,
    {
        InFile::new(f(self.inner.clone()), self.file_id)
    }
}

impl<T> Deref for InFile<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub type FileSpanned<T> = InFile<Spanned<T>>;
