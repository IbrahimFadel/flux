use ariadne::CharSet;
use ariadne::Config;

use ariadne::Source;
use flux_span::InputFile;
use std::collections::HashMap;

use super::Diagnostic;

pub struct SourceCache<'me> {
    db: &'me dyn crate::Db,
    map: HashMap<InputFile, Source>,
}

impl<'me> SourceCache<'me> {
    pub fn new(db: &'me dyn crate::Db) -> Self {
        Self {
            db,
            map: Default::default(),
        }
    }

    pub fn report_diagnostic(&self, diagnostic: &Diagnostic) {
        diagnostic
            .as_report(self.db, Config::default())
            .eprint(self)
            .unwrap()
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
            let report = diagnostic.as_report(self.db, cfg);
            report.write(self, &mut *buf).unwrap();
        }
    }

    pub fn add_input_file(&mut self, input_file: InputFile) {
        self.map.insert(
            input_file,
            Source::from(input_file.source_text(self.db).to_owned()),
        );
    }
}

impl<'cache> ariadne::Cache<InputFile> for &'cache SourceCache<'_> {
    type Storage = String;

    // I think Cache needs to be implemented for immutable reference. If i can figure out a way to allow &mut to be implemented, then we wont have to worry about inserting files, and just insert them automatically here
    fn fetch(&mut self, id: &InputFile) -> Result<&Source, Box<dyn std::fmt::Debug + '_>> {
        Ok(&self.map[id])
        // Ok(self.map.entry(*id).or_insert_with(|| {
        //     let source_text = id.source_text(self.db);
        //     Source::from(source_text.to_owned())
        // }))
    }

    fn display<'a>(&self, id: &'a InputFile) -> Option<Box<dyn std::fmt::Display + 'a>> {
        let s = id.name(self.db).as_str(self.db).to_string();
        Some(Box::new(s))
    }
}
