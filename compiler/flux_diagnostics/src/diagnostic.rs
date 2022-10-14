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
        }
    }

    pub(crate) fn to_report(&self) -> Report<FileSpan> {
        let (file_id, offset) = match &self.offset {
            Some(offset) => (offset.file_id, offset.inner),
            None => (FileId(Spur::default()), 0),
        };
        let mut builder = Report::build(self.kind.to_ariadne_report_kind(), file_id, offset)
            .with_code(self.code)
            .with_message(&self.msg);
        // .with_label(Label::new(FileSpan(InFile::new(
        //     Span::new((offset as u32)..(offset as u32)),
        //     file_id,
        // ))));

        for msg in &self.labels {
            builder.add_label(
                Label::new(FileSpan(msg.cloned_map(|msg| msg.span)))
                    .with_color(Color::Blue)
                    .with_message(&msg.inner.inner),
            );
        }

        builder.finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCode {
    ParserExpected,
    HirMissing,
    // CouldNotInitializeCacheDir,
    // UnexpectedToken,
    // Expected,
    // MissingDataInLowering,
    // TypeMismatch,
    // AppliedUnknownTrait,
    // AppliedUnknownMethodToTrait,
    // UnimplementedTraitMethods,
    // IncorrectNumberOfParamsInTraitMethodDefinition,
    // UnknownStruct,
    // NoSuchStructField,
    // TraitBoundsUnsatisfied,
    // NoSuchIntrinsic,
    // UninitializedFieldsInStructExpr,
    // StmtAfterTerminalStmt,
    // CouldNotInfer,
    // CouldNotOpenModule,
    // IndexMemAccessOnNonPtrExpr,
    // IncorrectNumberOfTypeParamsSuppliedToTrait,
    // UseOfUndeclaredGenerics,
    // UnknownType,
    // UnknownTrait,
}

impl std::fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "E{:04}", *self as u8)
    }
}
