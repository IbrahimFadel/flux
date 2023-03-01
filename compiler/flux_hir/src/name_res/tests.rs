use std::io::{BufWriter, Write};

use flux_diagnostics::{reporting::FileCache, Diagnostic};
use flux_span::FileId;
use la_arena::Idx;
use lasso::ThreadedRodeo;
use once_cell::sync::Lazy;
use pretty::BoxAllocator;

use crate::{
    body::LoweredBodies, hir::Function, item_tree::ItemTree, ModuleDefId, ModuleId, TypeInterner,
};

use super::{mod_res::FileResolver, DefMap};

static STRING_INTERNER: Lazy<ThreadedRodeo> = Lazy::new(ThreadedRodeo::new);
static TYPE_INTERNER: Lazy<TypeInterner> = Lazy::new(|| TypeInterner::new(&STRING_INTERNER));

fn check(content: &str) -> (DefMap, LoweredBodies, Vec<Diagnostic>) {
    let mut file_cache = FileCache::new(&STRING_INTERNER);
    let files = content.split("//-").skip(1);
    let mut entry_file_path = None;
    for (i, file) in files.enumerate() {
        let newline_loc = file
            .find("\n")
            .expect("malformated input to `check` function in name resolution unit test");
        let file_path = &file[..newline_loc];
        let content = &file[newline_loc..];
        file_cache.add_file(file_path, content);
        if i == 0 {
            entry_file_path = Some(file_path);
        }
    }
    let (def_map, mut diagnostics) = match entry_file_path {
        Some(entry_path) => crate::build_def_map(
            entry_path,
            &mut file_cache,
            &STRING_INTERNER,
            &TYPE_INTERNER,
            &TestFileResolver,
        ),
        None => panic!("malformated input to `check` function in name resolution unit test"),
    };
    let (lowered_bodies, mut diagnostics2) =
        crate::lower_def_map_bodies(&def_map, &STRING_INTERNER, &TYPE_INTERNER);
    diagnostics.append(&mut diagnostics2);
    (def_map, lowered_bodies, diagnostics)
}

fn fmt_file_id(file_id: FileId, string_interner: &'static ThreadedRodeo) -> &str {
    string_interner.resolve(&file_id.0)
}

fn fmt_function(
    f: &Function,
    f_idx: Idx<Function>,
    module_id: ModuleId,
    allocator: &BoxAllocator,
    buf: &mut BufWriter<Vec<u8>>,
    lowered_bodies: &LoweredBodies,
) {
    f.pretty::<_, ()>(&allocator, &STRING_INTERNER, &TYPE_INTERNER)
        .1
        .render(50, buf)
        .unwrap();
    let body = lowered_bodies
        .indices
        .get(&(module_id, ModuleDefId::FunctionId(f_idx)))
        .unwrap();
    body.pretty::<_, ()>(
        &allocator,
        &STRING_INTERNER,
        &TYPE_INTERNER,
        &lowered_bodies.exprs,
    )
    .1
    .render(50, buf)
    .unwrap();
    buf.write("\n\n".as_bytes()).unwrap();
}

fn fmt_item_tree(
    module_id: ModuleId,
    item_tree: &ItemTree,
    lowered_bodies: &LoweredBodies,
) -> String {
    let allocator = BoxAllocator;
    let mut buf = BufWriter::new(Vec::new());
    item_tree.functions.iter().for_each(|(f_idx, f)| {
        fmt_function(f, f_idx, module_id, &allocator, &mut buf, lowered_bodies);
    });
    let bytes = buf.into_inner().unwrap();
    String::from_utf8(bytes).unwrap()
}

fn fmt_def_map(def_map: &DefMap, lowered_bodies: &LoweredBodies) -> String {
    let mut item_tree_s = String::from("Item Tree\n\n");
    let mut mod_map_s = String::from("Module Id Map\n\n");
    for (module_id, item_tree) in def_map.item_trees.iter() {
        let parent = def_map[module_id].parent;
        if let Some(parent) = parent {
            mod_map_s += &format!("{} -> {}\n", parent.into_raw(), module_id.into_raw());
        }

        let file_id = def_map[module_id].file_id;
        item_tree_s += &format!(
            "File Id: {}\nModule Id: {}\n\n",
            fmt_file_id(file_id, &STRING_INTERNER),
            module_id.into_raw()
        );
        item_tree_s += &fmt_item_tree(module_id, item_tree, lowered_bodies);
    }
    format!("{}\n\n{}", mod_map_s, item_tree_s)
}

fn no_errors(content: &str) {
    let (def_map, lowered_bodies, diagnostics) = check(content);
    assert_eq!(diagnostics.len(), 0);
    let s = fmt_def_map(&def_map, &lowered_bodies);
    insta::assert_snapshot!(s);
}

#[test]
fn basic() {
    no_errors(
        r#"
//- ./main.flx

mod foo;
mod bar;

fn main() -> i32 {
    0
}

//- ./foo.flx

pub fn inside_foo() {}

fn also_inside_foo(x X, y i32) -> u32 {
    0
}

//- ./bar.flx

mod bazz;

fn inside_bar() {}

//- ./bar/bazz.flx

fn inside_bazz() {

}
"#,
    );
}

struct TestFileResolver;

impl FileResolver for TestFileResolver {
    fn resolve_absolute_path(
        &self,
        path: &str,
        file_cache: &mut FileCache,
    ) -> Option<(flux_span::FileId, String)> {
        Some(file_cache.get_by_file_path(path))
    }

    fn resolve_relative_path(
        &self,
        path: super::mod_res::RelativePath,
        file_cache: &mut flux_diagnostics::reporting::FileCache,
    ) -> Option<(flux_span::FileId, String)> {
        let anchor_path = file_cache.get_file_dir(&path.anchor);
        let absolute_path = format!("{anchor_path}/{}", path.path);
        Some(file_cache.get_by_file_path(&absolute_path))
    }
}
