use colored::Colorize;
use flux_span::Interner;
use pretty::RcDoc;

use crate::Generic;

impl Generic {
    pub fn to_doc(&self, interner: &'static Interner) -> RcDoc {
        RcDoc::text(interner.resolve(&self.name).yellow().to_string())
    }
}
