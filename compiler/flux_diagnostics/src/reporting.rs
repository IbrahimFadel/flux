use std::{
    collections::HashMap,
    fmt::{self, Debug},
    io::Write,
    ops::Deref,
    path::PathBuf,
};

use ariadne::{CharSet, Config, Source};
use flux_span::{FileId, InFile, Span};
use itertools::Itertools;
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

impl Debug for FileCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.map
                .keys()
                .map(|file_id| format!("{file_id:?}"))
                .join(", ")
        )
    }
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

    pub fn add_file_with_file_id(&mut self, file_id: FileId, content: &str) {
        let src = Source::from(content);
        self.map.insert(file_id, src);
    }

    pub fn get_file_path(&self, file_id: &FileId) -> &str {
        self.interner.resolve(&file_id.0)
    }

    pub fn get_file_dir(&self, file_id: &FileId) -> String {
        let path = self.get_file_path(file_id);
        let buf = PathBuf::from(path);
        buf.parent().unwrap().to_str().unwrap().to_string()
    }

    pub fn get_by_file_path(&self, path: &str) -> (FileId, String) {
        let file_id = FileId(self.interner.get_or_intern(path));
        let source = &self.map[&file_id];
        (file_id, source.chars().collect())
    }

    pub fn report_diagnostic(&self, diagnostic: &Diagnostic) {
        let report = diagnostic.to_report(Config::default());
        report.eprint(self).unwrap();
    }

    pub fn report_diagnostics(&self, diagnostics: &[Diagnostic]) {
        diagnostics.iter().for_each(|d| self.report_diagnostic(d));
    }

    pub fn write_diagnostics_to_buffer<W: Write>(&self, diagnostics: &[Diagnostic], buf: &mut W) {
        for diagnostic in diagnostics {
            let report = diagnostic.to_report(
                Config::default()
                    .with_char_set(CharSet::Ascii)
                    .with_color(false),
            );
            report.write(self, &mut *buf).unwrap();
        }
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
