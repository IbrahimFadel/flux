use smol_str::SmolStr;
use std::{fmt, ops::Deref};
use text_size::TextRange;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileId(pub SmolStr);

impl fmt::Display for FileId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
	pub range: TextRange,
	pub file_id: FileId,
}

impl Span {
	pub fn new(range: TextRange, file_id: FileId) -> Span {
		Span { range, file_id }
	}

	/// Combine two spans in the same file
	/// a must come before b
	pub fn combine(a: &Self, b: &Self) -> Self {
		assert!(a.range.start() < b.range.end());
		Span {
			range: TextRange::new(a.range.start(), b.range.end()),
			file_id: a.file_id.clone(),
		}
	}
}

impl ariadne::Span for Span {
	type SourceId = FileId;

	fn start(&self) -> usize {
		self.range.start().into()
	}

	fn end(&self) -> usize {
		self.range.end().into()
	}

	fn len(&self) -> usize {
		self.range.len().into()
	}

	fn source(&self) -> &Self::SourceId {
		&self.file_id
	}

	fn contains(&self, offset: usize) -> bool {
		self.start() <= offset && self.end() > offset
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
	pub inner: T,
	pub span: Span,
}

impl<T> Spanned<T> {
	pub fn new(inner: T, span: Span) -> Self {
		Self { inner, span }
	}

	pub fn vec_span(v: &[Spanned<T>]) -> Option<Span> {
		match v.len() {
			0 => None,
			1 => {
				let range = v.first().unwrap().span.range;
				Some(Span::new(range, v.first().unwrap().span.file_id.clone()))
			}
			_ => Some(Span::new(
				TextRange::new(
					v.first().unwrap().span.range.start(),
					v.last().unwrap().span.range.end(),
				),
				v.first().unwrap().span.file_id.clone(),
			)),
		}
	}
}

impl<A: Clone> Spanned<A> {
	pub fn map<F, B>(&self, f: F) -> Spanned<B>
	where
		F: FnOnce(A) -> B,
	{
		Spanned::new(f(self.inner.clone()), self.span.clone())
	}
}

impl<T> Deref for Spanned<T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl Spanned<SmolStr> {
	pub fn to_spanned_string(&self) -> Spanned<String> {
		Spanned {
			inner: self.to_string(),
			span: self.span.clone(),
		}
	}
}
