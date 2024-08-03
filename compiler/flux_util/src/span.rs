use std::ops::{Deref, DerefMut};

use text_size::{TextRange, TextSize};

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

    pub fn combine(a: Span, b: Span) -> Self {
        assert!(a.range.start() < b.range.end());
        Span::new(a.range.start().into()..b.range.end().into())
    }

    pub fn poisoned() -> Self {
        Self {
            range: TextRange::new(0.into(), 0.into()),
        }
    }

    pub fn end_span(&self) -> Self {
        Self {
            range: TextRange::new(
                self.range.end(),
                self.range.end().checked_add(TextSize::new(1)).unwrap(),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<T> {
    pub inner: T,
    pub span: Span,
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> Spanned<T> {
    pub fn new(inner: T, span: Span) -> Self {
        Self { inner, span }
    }

    /// Maps the inner value of a [`Spanned`]
    ///
    /// `Spanned<A>` -> `Spanned<B>`
    ///
    pub fn map<F, B>(self, f: F) -> Spanned<B>
    where
        F: FnOnce(T) -> B,
    {
        Spanned::new(f(self.inner), self.span)
    }

    // /// Maps the inner value of an [`Spanned`] passing the values to the closure by reference
    // ///
    // /// `Spanned<A>` -> `Spanned<B>`
    // ///
    // pub fn map_ref<F, B>(&self, f: F) -> Spanned<B>
    // where
    //     F: FnOnce(&T) -> B,
    // {
    //     Spanned::new(f(&self.inner), self.span)
    // }

    pub fn as_ref(&self) -> Spanned<&T> {
        Spanned {
            inner: &self.inner,
            span: self.span,
        }
    }
}
