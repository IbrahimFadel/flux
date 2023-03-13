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
