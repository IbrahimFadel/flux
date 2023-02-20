use ariadne::{Color, Label, Report, ReportKind};
use flux_span::{FileId, FileSpanned, InFile};
use lasso::Spur;

use crate::reporting::FileSpan;

pub trait ToDiagnostic {
    fn to_diagnostic(&self) -> Diagnostic;
}

#[derive(Debug, Clone, Copy)]
pub enum DiagnosticKind {
    Error,
    Warning,
}

impl DiagnosticKind {
    pub fn to_ariadne_report_kind(self) -> ReportKind {
        match self {
            Self::Error => ReportKind::Error,
            Self::Warning => ReportKind::Warning,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    kind: DiagnosticKind,
    offset: Option<InFile<usize>>,
    code: DiagnosticCode,
    msg: String,
    labels: Vec<FileSpanned<String>>,
    help: Option<String>,
}

impl Diagnostic {
    pub fn new(
        kind: DiagnosticKind,
        offset: InFile<usize>,
        code: DiagnosticCode,
        msg: String,
        labels: Vec<FileSpanned<String>>,
    ) -> Self {
        Self {
            kind,
            offset: Some(offset),
            code,
            msg,
            labels,
            help: None,
        }
    }

    pub fn error(
        offset: InFile<usize>,
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

    pub(crate) fn to_report(&self) -> Report<FileSpan> {
        let (file_id, offset) = match &self.offset {
            Some(offset) => (offset.file_id, offset.inner),
            None => (FileId(Spur::default()), 0),
        };
        let mut builder = Report::build(self.kind.to_ariadne_report_kind(), file_id, offset)
            .with_code(self.code)
            .with_message(&self.msg);

        if let Some(primary) = self.labels.get(0) {
            builder.add_label(
                Label::new(FileSpan(InFile::new(primary.span, file_id)))
                    .with_color(Color::Red)
                    .with_message(primary.inner.inner.clone()),
            )
        }

        if let Some(labels) = &self.labels.get(1..) {
            builder.add_labels(labels.iter().map(|msg| {
                Label::new(FileSpan(msg.map_ref(|msg| msg.span)))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCode {
    CouldNotFindEntryFile,
    CouldNotReadDir,
    CouldNotFindSubmodule,
    ParserExpected,
    HirMissing,
    TypeMismatch,
    UnknownLocal,
    CouldNotParseInt,
    UnknownFunction,
    UnknownStruct,
    IncorrectNumberOfArgsInCall,
    UnusedGenericParams,
    UninitializedFieldsInStructExpr,
    UnnecessaryFieldsInStructExpr,
    CouldNotResolveFunction,
    CouldNotResolvePath,
    CouldNotResolveStruct,
    ConflictingTraitImplementations,
    TraitInTraitRestrictionDoesNotExist,
    StmtFollowingTerminatorExpr,
    TraitMethodGenericsAlreadyDeclaredInTraitDecl,
    TraitNotImplementedForType,
}

impl std::fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "E{:04}", *self as u8)
    }
}
