use ariadne::{Color, Label, Report, ReportKind};
use flux_span::{FileSpan, FileSpanned, InputFile, ToSpan, Word};
use text_size::{TextRange, TextSize};

pub trait ToDiagnostic {
    fn to_diagnostic(&self) -> Diagnostic;
}

#[salsa::accumulator]
pub struct Diagnostics(Diagnostic);

#[derive(Debug, Clone)]
pub struct Diagnostic {
    kind: DiagnosticKind,
    offset: Option<FileSpan>,
    pub code: DiagnosticCode,
    msg: String,
    labels: Vec<FileSpanned<String>>,
    help: Option<String>,
}

impl Diagnostic {
    pub fn error(
        offset: FileSpan,
        code: DiagnosticCode,
        msg: String,
        labels: Vec<FileSpanned<String>>,
    ) -> Self {
        Self {
            kind: DiagnosticKind::Error,
            offset: Some(offset),
            code,
            msg,
            labels,
            help: None,
        }
    }

    /// Create a diagnostic that does not exist in a file
    /// For example, an error writing/reading from disk
    pub fn new_without_file(kind: DiagnosticKind, code: DiagnosticCode, msg: String) -> Self {
        Self {
            kind,
            offset: None,
            code,
            msg,
            labels: vec![],
            help: None,
        }
    }

    pub fn with_help(mut self, help: String) -> Self {
        self.help = Some(help);
        self
    }

    pub fn opt_with_help(mut self, help: Option<String>) -> Self {
        self.help = help;
        self
    }

    pub(crate) fn as_report(&self, db: &dyn crate::Db, config: ariadne::Config) -> Report<ASpan> {
        let (file_id, offset) = match &self.offset {
            Some(offset) => (offset.input_file, offset.span.range.start().into()),
            None => (
                InputFile::new(db, Word::intern(db, ""), String::with_capacity(0)),
                0,
            ),
        };

        let mut builder = Report::build(self.kind.as_ariadne_report_kind(), file_id, offset)
            .with_config(config)
            .with_code(self.code)
            .with_message(&self.msg);

        if let Some(primary) = self.labels.get(0) {
            builder.add_label(
                Label::new(ASpan::new(FileSpan::new(primary.file, primary.span)))
                    .with_color(Color::Red)
                    .with_message(primary.inner.inner.clone()),
            )
        }

        if let Some(labels) = &self.labels.get(1..) {
            builder.add_labels(labels.iter().map(|msg| {
                Label::new(ASpan::new(FileSpan::new(msg.file, msg.span)))
                    .with_color(Color::Blue)
                    .with_message(&msg.inner.inner)
            }))
        }

        if let Some(help) = &self.help {
            builder.set_help(help);
        }

        builder.finish()
    }
}

#[derive(Debug, Clone)]
pub enum DiagnosticKind {
    Warning,
    Error,
}

impl DiagnosticKind {
    pub fn as_ariadne_report_kind(&self) -> ReportKind<'static> {
        match self {
            Self::Error => ReportKind::Error,
            Self::Warning => ReportKind::Warning,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DiagnosticCode {
    CouldNotReadConfigFile,
    CouldNotReadEntryFile,
    ParserExpected,
}

impl std::fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "E{:04}", *self as u8)
    }
}

#[derive(Debug, Clone)]
pub struct ASpan(FileSpan);

impl ASpan {
    pub fn new(file_span: FileSpan) -> Self {
        Self(file_span)
    }

    pub fn empty(db: &dyn crate::Db) -> Self {
        Self(FileSpan {
            input_file: InputFile::new(
                db,
                Word::new(db, String::with_capacity(0)),
                String::with_capacity(0),
            ),
            span: TextRange::empty(TextSize::new(0)).to_span(),
        })
    }
}

impl ariadne::Span for ASpan {
    type SourceId = InputFile;

    fn source(&self) -> &Self::SourceId {
        &self.0.input_file
    }

    fn start(&self) -> usize {
        self.0.span.range.start().into()
    }

    fn end(&self) -> usize {
        self.0.span.range.end().into()
    }
}
