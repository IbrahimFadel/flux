use crate::{errors, no_errors};

no_errors! {
    call_function_with_use,
    r#"
//- ./main.flx

mod foo;

use foo::some_function;

fn main() {
    some_function();
    let x u32 = some_function();
    let x u32 = foo::some_function();
}

//- ./foo.flx

pub fn some_function() -> u32 {
    0
}
"#
}

errors! {
    unresolved_type,
    r#"
//- main.flx

trait Foo {}

apply Foo to Bar {}

fn main() {
    let x Bar = Bar {};
}
"#
}

no_errors! {
    resolved_primitive_types,
    r#"
//- main.flx

trait Foo {}

apply Foo to s64 {}
apply Foo to s32 {}
apply Foo to s16 {}
apply Foo to s8 {}
apply Foo to u64 {}
apply Foo to u32 {}
apply Foo to u16 {}
apply Foo to u8 {}
apply Foo to f64 {}
apply Foo to f32 {}

fn main() {
    let x s64 = 0;
    let x s32 = 0;
    let x s16 = 0;
    let x s8 = 0;
    let x u64 = 0;
    let x u32 = 0;
    let x u16 = 0;
    let x u8 = 0;
}
"#
}

errors! {
    use_private_function,
    r#"
//- ./main.flx

use foo::some_function;
mod foo;

//- ./foo.flx

fn some_function() {}
"#
}

errors! {
    call_private_function,
    r#"
//- ./main.flx

mod foo;

fn main() {
    foo::some_function();
}

//- ./foo.flx

fn some_function() {}
"#
}

errors! {
    use_private_struct,
    r#"
//- ./main.flx

use foo::Foo;
mod foo;

//- ./foo.flx

struct Foo {}
"#
}

errors! {
    apply_to_private_struct,
    r#"
//- ./main.flx

mod foo;

trait Bar {}

apply Bar to foo::Foo {}

//- ./foo.flx

struct Foo {}
"#
}

errors! {
    assign_var_with_private_struct,
    r#"
//- ./main.flx

mod foo;

fn main() {
    let x foo::Foo = foo::Foo {};
}

//- ./foo.flx

struct Foo {}
"#
}

no_errors! {
    use_path_works,
    r#"
//- ./main.flx

mod foo;

use foo::Foo;
use foo::some_function;

fn main() {
    let x Foo = Foo {};
    let x s32 = some_function();
}

//- ./foo.flx

pub struct Foo {}

pub fn some_function() -> s32 { 0 }
"#
}

errors! {
    use_private_mod,
    r#"
//- ./main.flx

mod foo;

use foo::bar;

fn main() {

}

//- ./foo.flx

mod bar;

//- ./foo/bar.flx

pub fn test() {}
"#
}

no_errors! {
    use_mod_path_works,
    r#"
//- ./main.flx

mod foo;

use foo::bar;

fn main() {
    bar::test();
}

//- ./foo.flx

pub mod bar;

//- ./foo/bar.flx

pub fn test() {}
"#
}
