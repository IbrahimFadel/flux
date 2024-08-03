use text_size::TextRange;

use crate::{FileId, InFile, Span, Spanned};

pub trait WithSpan: Sized {
    fn at(self, span: Span) -> Spanned<Self> {
        Spanned::new(self, span)
    }

    fn in_file(self, file_id: FileId) -> InFile<Self> {
        InFile::new(self, file_id)
    }

    fn file_span(self, file_id: FileId, span: Span) -> InFile<Spanned<Self>> {
        InFile::new(Spanned::new(self, span), file_id)
    }
}

impl<T> WithSpan for T where T: Sized {}

pub trait ToSpan: Into<TextRange> {
    fn to_span(self) -> Span {
        Span::new(self.into())
    }
}

impl<T> ToSpan for T where T: Into<TextRange> {}
