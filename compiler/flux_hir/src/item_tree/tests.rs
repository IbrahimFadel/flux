use flux_diagnostics::reporting::FileCache;
use lasso::ThreadedRodeo;
use once_cell::sync::Lazy;
use owo_colors::OwoColorize;

use super::{generate_item_tree, ItemTree};
use flux_parser::parse;

static INTERNER: Lazy<ThreadedRodeo> = Lazy::new(ThreadedRodeo::new);

#[cfg(test)]
macro_rules! test_input {
    ($name:ident, $src:literal) => {
        paste::paste! {
                #[test]
                fn [<item_tree_ $name>]() {
                    let input = $src;
                    let mut file_cache = FileCache::new(&INTERNER);
                    let file_id = file_cache.add_file("foo.flx", &input);
                    let result = parse(&input, file_id, &INTERNER);
                    let (root, _diagnostics) = (result.syntax(), result.diagnostics);
                    let item_tree = generate_item_tree(file_id, root, &INTERNER);
                    let s = fmt_item_tree(&item_tree, &INTERNER);
                    insta::assert_snapshot!(s);
                }
        }
    };
}

test_input!(one_tiny_normal_function, r#"fn main() {}"#);
test_input!(one_tiny_broken_functionr, r#"fn main( {}"#);
test_input!(
    bunch_of_functions_some_broken,
    r#"
fn main() {}
fn foo() -> i32 {}
fn bar) {}
fn bazz()
"#
);
test_input!(
    multiple_items,
    r#"
struct Foo {}
enum Bar {}
trait Bazz {}
apply Foo {}
apply Bazz to Foo {}
fn test() {}
"#
);

// #[test]
// fn test() {
//     let input = r#"fn main() {}"#;

//     let mut file_cache = FileCache::new(&INTERNER);
//     let file_id = file_cache.add_file("foo.flx", &input);
//     let result = parse(&input, file_id, &INTERNER);
//     let (root, diagnostics) = (result.syntax(), result.diagnostics);
//     let item_tree = generate_item_tree(file_id, root, &INTERNER);
//     println!("{:#?}", item_tree);
//     let s = fmt_item_tree(&item_tree, &INTERNER);
//     insta::assert_snapshot!(s);
// }

fn fmt_item_tree(item_tree: &ItemTree, interner: &'static ThreadedRodeo) -> String {
    let mut s = String::new();
    for (idx, function) in item_tree.data.functions.iter() {
        let f = format!(
            "{}\n\t{} {} {}\n\t{} {} {}\n",
            "-- Function --".green(),
            "name".yellow(),
            "->".blue(),
            interner.resolve(&function.name.inner).purple(),
            "visibility".yellow(),
            "->".blue(),
            format!("{:?}", function.visibility).purple()
        );
        s += &f;
    }
    s
}
