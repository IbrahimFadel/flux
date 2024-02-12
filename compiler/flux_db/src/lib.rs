use flux_diagnostics::Diagnostic;
use flux_hir::FluxParseInputFileExt;
use flux_parser::{FluxHirInputFileExt, Parse};
use flux_span::{InputFile, Word};

#[salsa::db(flux_hir::Jar, flux_span::Jar, flux_diagnostics::Jar, flux_parser::Jar)]
#[derive(Default)]
pub struct Db {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for Db {}

impl salsa::ParallelDatabase for Db {
    fn snapshot(&self) -> salsa::Snapshot<Self> {
        salsa::Snapshot::new(Db {
            storage: self.storage.snapshot(),
        })
    }
}

impl Db {
    pub fn new_input_file(&mut self, name: impl ToString, source_text: String) -> InputFile {
        let name = Word::intern(self, name);
        InputFile::new(self, name, source_text)
    }

    pub fn cst(&self, input_file: InputFile) -> Parse {
        input_file.cst(self)
    }

    pub fn package(&self, entry_file: InputFile) {
        entry_file.package(self);
    }

    pub fn diagnostics(&self, input_file: InputFile) -> Vec<Diagnostic> {
        vec![]
        // dada_check::check_input_file::accumulated::<flux_hir::diagnostic::Diagnostics>(
        //     self, input_file,
        // )
    }
}
