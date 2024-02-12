use text_size::TextRange;

use crate::{InFile, InputFile, Span, Spanned};

pub trait WithSpan: Sized {
    fn at(self, span: Span) -> Spanned<Self> {
        Spanned::new(self, span)
    }

    fn in_file(self, file: InputFile) -> InFile<Self> {
        InFile::new(self, file)
    }

    fn file_span(self, file: InputFile, span: Span) -> InFile<Spanned<Self>> {
        InFile::new(Spanned::new(self, span), file)
    }

    fn at_ref(&self, span: Span) -> Spanned<&Self> {
        Spanned::new(self, span)
    }

    fn in_file_ref(&self, file: InputFile) -> InFile<&Self> {
        InFile::new(self, file)
    }

    fn file_span_ref(&self, file: InputFile, span: Span) -> InFile<Spanned<&Self>> {
        InFile::new(Spanned::new(self, span), file)
    }
}

impl<T> WithSpan for T where T: Sized {}

pub trait ToSpan: Into<TextRange> {
    fn to_span(self) -> Span {
        Span::new(self.into())
    }
}

impl<T> ToSpan for T where T: Into<TextRange> {}
