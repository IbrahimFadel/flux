use crate::errors;

errors! {test,
r"
//- ./main.flx

mod foo;
mod math;

fn foo() }

//- ./foo.flx

mod bar;

fn foo() {}

//- ./foo/bar.flx

//- ./math.flx

mod addition;

//- ./math/addition.flx
"}
