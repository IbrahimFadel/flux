use std::fmt::Display;

use ariadne::{sources, Report};
use flux_span::{FileId, Span};
use smol_str::SmolStr;

pub trait Error {
	fn to_report(&self) -> Report<Span>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FluxErrorCode {
	UnexpectedToken,
	Expected,
	MissingDataInLowering,
	TypeMismatch,
	AppliedUnknownTrait,
	AppliedUnknownMethodToTrait,
	UnimplementedTraitMethods,
	IncorrectNumberOfParamsInTraitMethodDefinition,
	UnknownStruct,
	NoSuchStructField,
	TraitBoundsUnsatisfied,
	NoSuchIntrinsic,
	UninitializedFieldsInStructExpr,
	StmtAfterTerminalStmt,
	CouldNotInfer,
	CouldNotOpenModule,
	IndexMemAccessOnNonPtrExpr,
}

impl std::fmt::Display for FluxErrorCode {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "E{:04}", *self as u8)
	}
}

pub struct FluxErrorReporting {
	pub files: Vec<(FileId, String)>,
}

impl FluxErrorReporting {
	pub fn add_file(&mut self, name: SmolStr, src: String) -> FileId {
		self.files.push((FileId(name.clone()), src));
		FileId(name)
	}

	pub fn report(&self, err: &Report<Span>) {
		err.print(sources(self.files.clone())).unwrap();
	}
}

pub fn comma_separated_end_with_or<T: Display>(els: &[T]) -> String {
	let mut els: Vec<String> = els.iter().map(|el| format!("`{}`", el)).collect();
	let len = els.len();
	if len > 1 {
		els[len - 1] = format!("or {}", els[len - 1]);
	}
	els.join(", ")
}

pub fn comma_separated_end_with_and<T: Display>(els: impl Iterator<Item = T>) -> String {
	let mut els: Vec<String> = els.map(|el| format!("`{}`", el)).collect();
	let len = els.len();
	if len > 1 {
		els[len - 1] = format!("and {}", els[len - 1]);
	}
	els.join(", ")
}
