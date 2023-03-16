use crate::{errors, no_errors};

no_errors! {
    basic_generic_function,
    r#"
//- main.flx

fn foo<X>(x X) -> X {
    x
}    
"#
}

errors! {
    unused_generics_in_function,
    r#"
//- main.flx

fn foo<X, Y>(x X) -> X {
    x
}
"#
}

// no_errors! {
//     generic_in_body_not_marked_unused,
//     r#"
// //- main.flx

// fn foo<X, Y>(x X) -> X {
//     x
// }
// "#
// }

errors! {
    unknown_generic_in_function_where_predicate,
    r#"
//- main.flx

trait Foo {}

fn foo<T>() -> u32 where X is Foo {
    0
}
"#
}

errors! {
    unknown_generic_in_method_where_predicate,
    r#"
//- main.flx

trait Foo {}

struct Bar {}

apply to Bar {
    fn foo<T>() where X is Foo {

    }
}
"#
}

errors! {
    unknown_trait_in_where_predicate,
    r#"
//- main.flx

fn foo<T>() where T is Foo {}
"#
}

errors! {
    trait_method_trait_bounds_not_met_in_apply_method,
    r#"
//- main.flx

trait Bar {
    fn bar<T>() where T is Foo;
}

trait Foo {}

trait NotFoo {}

struct Bazz {}

apply Bar to Bazz {
    fn bar<T>() where T is NotFoo {

    }
}
"#
}

errors! {
    unused_generics_in_struct,
    r#"
//- main.rs

struct Bazz<T> {}

struct Bazz<A, B, C> {
    a A,
}
"#
}

errors! {
    missing_generic_params_in_trait_restriction,
    r#"
//- main.rs

trait Foo {}
trait Bar<T> {}
struct Bazz<T> {
    t T
}

apply<T> Foo to Bazz<T>
    where T is Bar
{

}
"#
}

errors! {
    missing_generic_params_in_nested_trait_restriction,
    r#"
//- main.rs

trait Bar {}
trait Dummy<T> where T is Bar {}
trait Foo<A, B>
    where
        A is Dummy<B>,
        B is Bar,
{}

fn foo<T, A, B>()
    where
        T is Foo<A, B>,
        A is Dummy<B>,
{

}
"#
}

errors! {
    trait_methods_not_implemented_in_apply,
    r#"
//- main.flx

trait Foo {
    fn foo();
    fn bar();
    fn bazz();
}

struct Bar {}

apply Foo to Bar {

}
"#
}

errors! {
    methods_do_not_belong_in_apply,
    r#"
//- main.flx

trait Foo {
    fn foo();
    fn bar();
    fn bazz();
}

struct Bar {}

apply Foo to Bar {
    fn test() {}
}
"#
}

errors! {
    duplicate_generics_when_combining_trait_and_method_generics,
    r#"
//- main.flx

trait Foo<A, B> {
    fn foo<A>();
    fn bar<B>();
    fn foo_bar<A, B>();
    fn foo_bar<C>();
}
"#
}

errors! {
    duplicate_generics_when_combining_apply_and_method_generics,
    r#"
//- main.flx
struct Foo {}

apply<A, B> to Foo {
    fn foo<A>() {}
    fn bar<B>() {}
    fn foo_bar<A, B>() {}
    fn foo_bar<C>() {}
}
"#
}

errors! {
    restriction_not_met_struct_initialization,
    r#"
//- main.flx

trait Bar {}

struct Foo<T>
    where T is Bar
{
    x T
}

fn main() {
    let foo = Foo {
        x: 0,
    };
}
"#
}

no_errors! {
    restriction_met_struct_initialization,
    r#"
//- main.flx

trait Bar {}

struct Foo<T>
    where T is Bar
{
    x T
}

struct Bazz {}

apply Bar to Bazz {}

fn main() {
    let foo = Foo {
        x: Bazz {},
    };
}
"#
}

// no_errors! {
//     int_subtyping,
//     r#"
// //- main.flx

// trait Bar {}

// struct Foo<T>
//     where T is Bar
// {
//     x T
// }

// apply Bar to s32 {

// }

// fn main() {
//     let foo = Foo {
//         x: 0,
//     };
// }
// "#
// }
