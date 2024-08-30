use ariadne::{Color, Label, Report, ReportKind};
use flux_util::{FileId, FileSpan, FileSpanned};

pub trait ToDiagnostic {
    fn to_diagnostic(&self) -> Diagnostic;
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    kind: DiagnosticKind,
    pub offset: FileSpan,
    pub code: DiagnosticCode,
    msg: String,
    pub labels: Vec<FileSpanned<String>>,
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
            offset: offset,
            code,
            msg,
            labels,
            help: None,
        }
    }

    // /// Create a diagnostic that does not exist in a file
    // /// For example, an error writing/reading from disk
    // pub fn new_without_file(kind: DiagnosticKind, code: DiagnosticCode, msg: String) -> Self {
    //     Self {
    //         kind,
    //         offset: None,
    //         code,
    //         msg,
    //         labels: vec![],
    //         help: None,
    //     }
    // }

    pub fn with_help(mut self, help: String) -> Self {
        self.help = Some(help);
        self
    }

    pub fn opt_with_help(mut self, help: Option<String>) -> Self {
        self.help = help;
        self
    }

    pub(crate) fn as_report(&self, config: ariadne::Config) -> Report<ASpan> {
        // let (file_id, offset) = match &self.offset {
        //     Some(offset) => (Some(offset.file_id), offset.span.range.start().into()),
        //     None => (None, 0),
        // };
        let (file_id, offset) = (self.offset.file_id, self.offset.span.range.start().into());

        let mut builder = Report::build(self.kind.as_ariadne_report_kind(), file_id, offset)
            .with_config(config)
            .with_code(self.code)
            .with_message(&self.msg);

        if let Some(primary) = self.labels.get(0) {
            builder.add_label(
                Label::new(ASpan::new(FileSpan::new(primary.file_id, primary.span)))
                    .with_color(Color::Red)
                    .with_message(primary.inner.inner.clone()),
            )
        }

        if let Some(labels) = &self.labels.get(1..) {
            builder.add_labels(labels.iter().map(|msg| {
                Label::new(ASpan::new(FileSpan::new(msg.file_id, msg.span)))
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

    UnknownGeneric,
    CouldNotResolveModDecl,
    DuplicateGenericParams,
    UnusedGenerics,

    CannotResolveEmptyPath,
    UnresolvedPath,
    PrivateModule,
    UnexpectedItem,

    PositiveIntegerOverflow,
    StmtFollowingTerminatorExpr,
    ExpectedDifferentItem,
    MissingFieldsInStructExpr,
    MissingGenericArguments,
    UnknownLocal,
    UnknownIntrinsic,
    IncorrectNumberOfArgs,
    IncorrectStructFieldsInInitialization,
    MemberAccessOnNonStruct,
    UnknownStructField,
    CouldNotResolveStruct,
    CalleeNotFunction,

    TypeMismatch,
    CouldNotInfer,
    CouldBeMultipleTypes,
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
}

impl ariadne::Span for ASpan {
    type SourceId = FileId;

    fn source(&self) -> &Self::SourceId {
        &self.0.file_id
    }

    fn start(&self) -> usize {
        self.0.span.range.start().into()
    }

    fn end(&self) -> usize {
        self.0.span.range.end().into()
    }
}
