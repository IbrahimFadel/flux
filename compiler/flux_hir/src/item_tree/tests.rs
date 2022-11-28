use flux_diagnostics::reporting::FileCache;
use flux_span::Spanned;
use itertools::Itertools;
use la_arena::Arena;
use lasso::ThreadedRodeo;
use once_cell::sync::Lazy;

use crate::hir::{
    GenericParamList, ParamList, Path, StructFieldList, Type, TypeBound, TypeIdx, Visibility,
    WhereClause,
};

use super::{generate_item_tree, Function, ItemTree, Struct};
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
                    let (item_tree, _diagnostics, types) = generate_item_tree(file_id, root, &INTERNER);
                    let fmt = ItemTreeFormatter::new(&item_tree,  &INTERNER, &types);
                    let s = fmt.fmt();
                    println!("{s}");
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
pub fn foo<T, Y>(x i32, y T, test Y) -> i32
    where T is Foo, T is Bar, Y is Foo<T>
{}
fn bar) {}
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

test_input!(
    basic_struct,
    r#"
struct Foo {
    x T,
    y i32,
    z Bar,
}
"#
);

test_input!(
    struct_with_generics_and_where_clause,
    r#"
struct Foo<T>
    where T is Bar
{
    T x,
    Bazz y
}
"#
);

struct ItemTreeFormatter<'a> {
    item_tree: &'a ItemTree,
    interner: &'static ThreadedRodeo,
    types: &'a Arena<Spanned<Type>>,
}

impl<'a> ItemTreeFormatter<'a> {
    fn new(
        item_tree: &'a ItemTree,
        interner: &'static ThreadedRodeo,
        types: &'a Arena<Spanned<Type>>,
    ) -> Self {
        Self {
            item_tree,
            interner,
            types,
        }
    }

    pub fn fmt(&self) -> String {
        let mut s = String::new();
        let functions = self
            .item_tree
            .data
            .functions
            .iter()
            .format_with("\n", |(_, function), f| f(&self.fmt_fn_decl(function)))
            .to_string();
        s += &functions;
        s += "\n";
        let structs = self
            .item_tree
            .data
            .structs
            .iter()
            .format_with("\n", |(_, strukt), f| f(&self.fmt_struct_decl(strukt)))
            .to_string();
        s += &structs;
        s += "\n";
        s
    }

    fn fmt_fn_decl(&self, function: &Function) -> String {
        format!(
            "{}fn {}{}({}) -> {}{}",
            self.fmt_visibility(function.visibility),
            self.interner.resolve(&function.name),
            self.fmt_generic_params(&function.generic_param_list),
            self.fmt_fn_params(&function.params),
            self.fmt_ty(function.ret_type),
            self.fmt_where_clause(&function.where_clause),
        )
    }

    fn fmt_struct_decl(&self, strukt: &Struct) -> String {
        format!(
            "{}struct {}{}{}{}",
            self.fmt_visibility(strukt.visibility),
            self.interner.resolve(&strukt.name),
            self.fmt_generic_params(&strukt.generic_param_list),
            self.fmt_where_clause(&strukt.where_clause),
            if strukt.where_clause.len() > 0 {
                self.fmt_struct_field_list(&strukt.field_list)
            } else {
                format!(" {}", self.fmt_struct_field_list(&strukt.field_list))
            }
        )
    }

    fn fmt_visibility(&self, visibility: Visibility) -> String {
        if visibility == Visibility::Public {
            "pub ".to_string()
        } else {
            String::new()
        }
    }

    fn fmt_generic_params(&self, generic_params: &GenericParamList) -> String {
        // If we don't sort it, they will be different each time and cause false errors (HashSet wont always be in the same order)
        let sorted_by_name = generic_params
            .iter()
            .sorted_by_key(|name| self.interner.resolve(name));
        if generic_params.len() > 0 {
            format!(
                "<{}>",
                sorted_by_name
                    .map(|param| self.interner.resolve(param))
                    .join(", ")
            )
        } else {
            String::new()
        }
    }

    fn fmt_where_clause(&self, where_clause: &WhereClause) -> String {
        if where_clause.len() > 0 {
            format!(
                "\n\twhere\n\t\t{}\n",
                where_clause
                    .iter()
                    .format_with(",\n\t\t", |where_predicate, f| {
                        f(&format_args!(
                            "{}{}",
                            self.interner.resolve(&where_predicate.generic),
                            if where_predicate.trait_restrictions.len() > 0 {
                                format!(
                                    ": {}",
                                    where_predicate
                                        .trait_restrictions
                                        .iter()
                                        .map(|restriction| self.fmt_type_bound(restriction))
                                        .join(", ")
                                )
                            } else {
                                String::new()
                            }
                        ))
                    })
            )
        } else {
            String::new()
        }
    }

    fn fmt_type_bound(&self, type_bound: &TypeBound) -> String {
        format!(
            "{}{}",
            self.interner.resolve(&type_bound.name),
            if !type_bound.args.is_empty() {
                format!(
                    "<{}>",
                    type_bound
                        .args
                        .iter()
                        .map(|idx| self.fmt_ty(*idx))
                        .join(", ")
                )
            } else {
                String::new()
            }
        )
    }

    fn fmt_fn_params(&self, params: &ParamList) -> String {
        params
            .iter()
            .format_with(", ", |param, f| {
                f(&format_args!(
                    "{} {}",
                    self.interner.resolve(&param.name),
                    self.fmt_ty(param.ty),
                ))
            })
            .to_string()
    }

    fn fmt_struct_field_list(&self, field_list: &StructFieldList) -> String {
        format!(
            "{{\n\t{}\n}}",
            field_list.iter().format_with("\n\t", |field, f| {
                f(&format_args!(
                    "{} {}",
                    self.interner.resolve(&field.name),
                    self.fmt_ty(field.ty),
                ))
            })
        )
    }

    fn fmt_ty(&self, ty: TypeIdx) -> String {
        let ty = &self.types[ty];
        match &ty.inner {
            Type::Path(path, generic_args) => self.fmt_path_type(path, generic_args),
            Type::Generic(name) => self.interner.resolve(name).to_string(),
            Type::Tuple(idxs) => {
                format!("({})", idxs.iter().map(|idx| self.fmt_ty(*idx)).join(", "))
            }
            Type::Unknown => "unknown".to_string(),
        }
    }

    fn fmt_path_type(&self, path: &Path, generic_args: &[TypeIdx]) -> String {
        format!(
            "{}{}",
            path.to_string(self.interner),
            if !generic_args.is_empty() {
                format!(
                    "<{}>",
                    generic_args.iter().map(|arg| self.fmt_ty(*arg)).join(", ")
                )
            } else {
                String::new()
            }
        )
    }
}
