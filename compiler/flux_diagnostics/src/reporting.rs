use ariadne::CharSet;
use ariadne::Config;

use ariadne::Source;
use flux_span::FileId;
use flux_span::Interner;
use std::collections::HashMap;
use std::path::PathBuf;

use super::Diagnostic;

pub struct SourceCache {
    interner: &'static Interner,
    map: HashMap<FileId, Source<String>>,
}

impl SourceCache {
    pub fn new(interner: &'static Interner) -> Self {
        Self {
            interner,
            map: Default::default(),
        }
    }

    pub fn report_diagnostic(&self, diagnostic: &Diagnostic) {
        diagnostic
            .as_report(Config::default())
            .eprint(self)
            .unwrap()
    }

    pub fn report_diagnostics(&self, diagnostics: impl Iterator<Item = Diagnostic>) {
        diagnostics.for_each(|diagnostic| {
            diagnostic
                .as_report(Config::default())
                .eprint(self)
                .unwrap()
        })
    }

    pub fn write_diagnostics_to_buffer<W: std::io::Write>(
        &self,
        diagnostics: &[Diagnostic],
        buf: &mut W,
    ) {
        let cfg = Config::default()
            .with_char_set(CharSet::Ascii)
            .with_color(false);
        for diagnostic in diagnostics {
            let report = diagnostic.as_report(cfg);
            report.write(self, &mut *buf).unwrap();
        }
    }

    pub fn add_input_file(&mut self, path: &str, content: String) -> FileId {
        let id = self.interner.get_or_intern(path);
        let id = FileId::new(id);
        let src = Source::from(content);
        self.map.insert(id, src);
        id
    }

    #[inline]
    pub fn get_file_path(&self, file_id: &FileId) -> &str {
        self.interner.resolve(&file_id.key())
    }

    pub fn get_file_dir(&self, file_id: &FileId) -> String {
        let path = self.get_file_path(file_id);
        let buf = PathBuf::from(path);
        buf.parent().unwrap().to_str().unwrap().to_string()
    }

    pub fn get_file_content(&self, file_id: &FileId) -> String {
        self.map[file_id].chars().collect()
    }

    pub fn get_by_file_path(&self, path: &str) -> (FileId, String) {
        let id = FileId::new(self.interner.get_or_intern(path));
        let source = &self.map[&id];
        let s = source.chars().collect();
        (id, s)
    }
}

impl<'cache> ariadne::Cache<FileId> for &'cache SourceCache {
    type Storage = String;

    fn fetch(
        &mut self,
        id: &FileId,
    ) -> Result<&Source<Self::Storage>, Box<dyn std::fmt::Debug + '_>> {
        Ok(&self.map[id])
    }

    fn display<'b>(&self, id: &'b FileId) -> Option<Box<dyn std::fmt::Display + 'b>> {
        let s = self.interner.resolve(id.key()).to_string();
        Some(Box::new(s))
    }
}
