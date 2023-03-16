use crate::errors;

errors! {
    field_type_mismatches,
    r#"
//- main.flx

struct Foo {
    x s32,
    y Bar,
    z f32,
}

struct Bar {
    x u32
}

fn main() {
    let foo_instance = Foo {
        x: 0.0,
        y: 10,
        z: 1,
    };
}
"#
}

errors! {
    uninitialized_fields,
    r#"
//- main.flx

struct Foo {
    x s32,
    y Bar,
    z f32,
}

struct Bar {
    x u32
}

fn main() {
    let foo_instance = Foo {
        x: 1,
    };
}
"#
}

errors! {
    unknown_fields,
    r#"
//- main.flx

struct Foo {
    x s32,
    y Bar,
    z f32,
}

struct Bar {
    x u32
}

fn main() {
    let foo_instance = Foo {
        x: 0,
        foo: 0,
        bar: 0,
        bazz: 0,
    };
}
"#
}

errors! {
    typecheck_field_access,
    r#"
//- main.flx

struct Foo {
    x u32,
}

fn main() {
    let foo = Foo {
        x: 0
    };

    let x s32 = foo.x;
}
"#
}

errors! {
    typecheck_method_call,
    r#"
//- main.flx

struct Foo {}

apply to Foo {
    fn foo() {}
}

fn main() {
    let foo = Foo {};

    let x s32 = foo.foo();
}
"#
}

errors! {
    unknown_method_called,
    r#"
//- main.flx

struct Foo {}

fn main() {
    let foo = Foo {};

    foo.foo();
}
"#
}

errors! {
    unknown_field_referenced,
    r#"
//- main.flx

struct Foo {}

fn main() {
    let foo = Foo {};

    let x = foo.x;
}
"#
}

errors! {
    generic_typecheck_in_initialization,
    r#"
//- main.flx

trait Bar {}

struct Foo<T>
    where T is Bar
{
    x T
}

struct Bazz {}

fn main() {
    let bazz = Bazz {};
    let foo = Foo {
        x: bazz,
    };
}
"#
}
