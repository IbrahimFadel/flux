use ariadne::{sources, Color, Fmt, Label, Report, ReportKind};
use flux_span::{FileId, Span};
use smol_str::SmolStr;

pub trait Error {
	fn to_report(&self) -> Report<Span>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FluxErrorCode {
	UnexpectedToken,
	TypeMismatch,
	AppliedUnknownTrait,
	AppliedUnknownMethodToTrait,
	UnimplementedTraitMethods,
}

impl std::fmt::Display for FluxErrorCode {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "E{:04}", *self as u8)
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct FluxError {
	pub msg: String,
	pub span: Span,
	pub code: FluxErrorCode,
	pub primary: (String, Span),
	pub labels: Vec<(String, Span)>,
	pub notes: Vec<String>,
}

impl FluxError {
	pub fn build(msg: String, span: Span, code: FluxErrorCode, primary: (String, Span)) -> Self {
		Self {
			msg,
			span,
			code,
			primary,
			labels: vec![],
			notes: vec![],
		}
	}

	pub fn with_msg(mut self, msg: String) -> FluxError {
		self.msg = msg;
		self
	}

	pub fn with_code(mut self, code: FluxErrorCode) -> FluxError {
		self.code = code;
		self
	}

	pub fn with_primary(mut self, msg: String, span: Span) -> FluxError {
		self.primary = (msg, span);
		self
	}

	pub fn with_label(mut self, msg: String, span: Span) -> FluxError {
		self.labels.push((msg, span));
		self
	}

	pub fn with_labels(mut self, labels: &mut Vec<(String, Span)>) -> FluxError {
		self.labels.append(labels);
		self
	}

	pub fn with_note(mut self, msg: String) -> FluxError {
		self.notes.push(msg);
		self
	}

	pub fn prefix_msg(mut self, msg: String) -> FluxError {
		self.msg = format!("{}{}", msg, self.msg);
		self
	}

	pub(crate) fn to_diagnostic(&self) -> Report<Span> {
		let primary = Color::Red;
		let mut report = Report::build(
			ReportKind::Error,
			self.span.file_id.clone(),
			self.span.range.start().into(),
		)
		.with_code(self.code)
		.with_message(&self.msg)
		.with_label(
			Label::new(self.primary.1.clone())
				.with_message(self.primary.0.clone().fg(primary))
				.with_color(primary),
		);

		for label in &self.labels {
			let colour = Color::Blue;
			report = report.with_label(
				Label::new(label.1.clone())
					.with_message(label.0.clone().fg(colour))
					.with_color(colour),
			);
		}

		for note in &self.notes {
			report = report.with_note(note)
		}

		report.finish()
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
