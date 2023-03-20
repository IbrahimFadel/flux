use crate::errors;

errors! {
    unuses_generic_params,
    r#"
//- main.flx

enum Result<A, B, C> {
    A -> A,
    B,
    C -> C
}
"#
}

errors! {
    type_mismatch,
    r#"
//- main.flx

enum Foo {
    A,
    B -> u32,
}

fn main() {
    let x u32 = Foo::A;
    let x u32 = Foo::B(0);
}
"#
}

errors! {
    missing_arg,
    r#"
//- main.flx

enum Foo {
    A -> s32,
}

fn main() {
    let x = Foo::A;
}
"#
}

errors! {
    incorrect_num_args,
    r#"
//- main.flx

enum Foo {
    A -> s32,
}

fn main() {
    let x = Foo::A(0, 0);
}
"#
}

errors! {
    type_mismatch_in_args,
    r#"
//- main.flx

enum Foo {
    A -> s32,
}

fn main() {
    let x = Foo::A(0.0);
}
"#
}
