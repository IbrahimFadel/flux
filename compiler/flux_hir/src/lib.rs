#![feature(trait_upcasting)]

use flux_span::InputFile;
use pkg::PkgBuilder;

mod body;
mod hir;
mod item;
mod item_scope;
mod item_tree;
mod module;
mod pkg;

fn lower_package(db: &dyn Db, entry: InputFile) {
    let mut pkg_builder = PkgBuilder::new(db);
    pkg_builder.seed_with_entry(entry);
}

#[extension_trait::extension_trait]
pub impl FluxParseInputFileExt for InputFile {
    fn package(self, db: &dyn crate::Db) {
        lower_package(db, self)
    }

    fn item_tree(self, db: &dyn Db) {}
}

#[salsa::jar(db = Db)]
pub struct Jar(crate::hir::Function);

pub trait Db: salsa::DbWithJar<Jar> + flux_parser::Db {}

impl<DB> Db for DB where DB: ?Sized + salsa::DbWithJar<Jar> + flux_parser::Db {}
