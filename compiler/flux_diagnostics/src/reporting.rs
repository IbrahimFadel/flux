use std::{collections::HashMap, fmt, ops::Deref};

use ariadne::Source;
use flux_span::{FileId, InFile, Span};
use lasso::ThreadedRodeo;

use crate::Diagnostic;

#[derive(Debug, Clone)]
pub(crate) struct FileSpan(pub InFile<Span>);

impl Deref for FileSpan {
    type Target = InFile<Span>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<InFile<Span>> for FileSpan {
    fn from(s: InFile<flux_span::Span>) -> Self {
        Self(s)
    }
}

impl ariadne::Span for FileSpan {
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

pub struct FileCache {
    interner: &'static ThreadedRodeo,
    map: HashMap<FileId, Source>,
}

impl FileCache {
    pub fn new(interner: &'static ThreadedRodeo) -> Self {
        Self {
            interner,
            map: HashMap::new(),
        }
    }

    pub fn add_file(&mut self, path: &str, content: &str) -> FileId {
        let id = self.interner.get_or_intern(path);
        let id = FileId(id);
        let src = Source::from(content);
        self.map.insert(id, src);
        id
    }

    pub fn report_diagnostic(&self, diagnostic: &Diagnostic) {
        let report = diagnostic.to_report();
        report.eprint(self).unwrap();
    }

    pub fn report_diagnostics(&self, diagnostics: &[Diagnostic]) {
        diagnostics.iter().for_each(|d| self.report_diagnostic(d));
    }
}

impl ariadne::Cache<FileId> for &FileCache {
    fn fetch(&mut self, id: &FileId) -> Result<&Source, Box<dyn fmt::Debug + '_>> {
        Ok(&self.map[id])
    }

    fn display<'a>(&self, id: &'a FileId) -> Option<Box<dyn fmt::Display + 'a>> {
        Some(Box::new(self.interner.resolve(&id.0)))
    }
}
