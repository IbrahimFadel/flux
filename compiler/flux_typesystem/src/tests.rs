// use flux_span::{FileId, Span, WithSpan};
// use lasso::ThreadedRodeo;
// use once_cell::sync::Lazy;

// use crate::{ConcreteKind, TChecker, TraitApplication, TraitRestriction, Type, TypeKind};

// static INTERNER: Lazy<ThreadedRodeo> = Lazy::new(ThreadedRodeo::new);

// macro_rules! filespanned {
//     ($e:expr) => {
//         $e.file_span(FileId::poisoned(), Span::poisoned())
//     };
// }

// macro_rules! filespan {
//     () => {
//         Span::poisoned().in_file(FileId::poisoned())
//     };
// }

// macro_rules! generic {
//     ($name:ident) => {
//         filespanned!(Type::new(TypeKind::Generic(
//             INTERNER.get_or_intern(stringify!($name)),
//             vec![]
//         )))
//     };
//     ($name:ident: $(
//         $restrictions:expr
//     ),*) => {
//         filespanned!(Type::new(TypeKind::Generic(
//             INTERNER.get_or_intern(stringify!($name)),
//             vec![$($restrictions),*]
//         )))
//     };
// }

// macro_rules! restriction {
//     ($trait_id:expr, $path:ident) => {
//         TraitRestriction {
//             trait_id: $trait_id,
//             trait_name: filespanned!(INTERNER.get_or_intern(stringify!($path))),
//             args: vec![],
//         }
//     };
//     ($trait_id:expr, $path:ident, $args:expr) => {
//         TraitRestriction {
//             trait_id: $trait_id,
//             trait_name: filespanned!(INTERNER.get_or_intern(stringify!($path))),
//             args: $args,
//         }
//     };
// }

// macro_rules! path {
//     ($name:ident) => {
//         filespanned!(Type::new(TypeKind::Concrete(ConcreteKind::Path(
//             INTERNER.get_or_intern(stringify!($name))
//         ))))
//     };
// }

// macro_rules! int {
//     () => {
//         filespanned!(Type::new(TypeKind::Int(None)))
//     };
//     ($depends_on:expr) => {
//         filespanned!(Type::new(TypeKind::Int(Some($depends_on))))
//     };
// }

// macro_rules! float {
//     () => {
//         filespanned!(Type::new(TypeKind::Float(None)))
//     };
//     ($depends_on:expr) => {
//         filespanned!(Type::new(TypeKind::Float(Some($depends_on))))
//     };
// }

// macro_rules! unknown {
//     () => {
//         filespanned!(Type::new(TypeKind::Unknown))
//     };
// }

// macro_rules! never {
//     () => {
//         filespanned!(Type::new(TypeKind::Never))
//     };
// }

// macro_rules! declare_application {
//     ($tchk:expr, $trait_id:expr => $trait_args:expr, $impltor:expr, $impltor_args:expr) => {
//         $tchk.trait_applications.push_application(
//             $trait_id,
//             TraitApplication::new($trait_args, $impltor, $impltor_args),
//         )
//     };
// }

// macro_rules! is_ok {
//     ($tchk:expr, $a:expr, $b:expr) => {
//         assert!($tchk.unify($a, $b, filespan!()).is_ok());
//     };
// }

// macro_rules! is_err {
//     ($tchk:expr, $a:expr, $b:expr) => {
//         assert!($tchk.unify($a, $b, filespan!()).is_err());
//     };
// }

// #[test]
// fn primitives() {
//     let mut tchk = TChecker::new(&INTERNER);
//     let a = tchk.tenv.insert(path!(s64));
//     let b = tchk.tenv.insert(path!(s64));
//     is_ok!(tchk, a, b);
//     let a = tchk.tenv.insert(path!(s32));
//     let b = tchk.tenv.insert(path!(s32));
//     is_ok!(tchk, a, b);
//     let a = tchk.tenv.insert(path!(s16));
//     let b = tchk.tenv.insert(path!(s16));
//     is_ok!(tchk, a, b);
//     let a = tchk.tenv.insert(path!(s8));
//     let b = tchk.tenv.insert(path!(s8));
//     is_ok!(tchk, a, b);
//     let a = tchk.tenv.insert(path!(u64));
//     let b = tchk.tenv.insert(path!(u64));
//     is_ok!(tchk, a, b);
//     let a = tchk.tenv.insert(path!(u32));
//     let b = tchk.tenv.insert(path!(u32));
//     is_ok!(tchk, a, b);
//     let a = tchk.tenv.insert(path!(u16));
//     let b = tchk.tenv.insert(path!(u16));
//     is_ok!(tchk, a, b);
//     let a = tchk.tenv.insert(path!(u8));
//     let b = tchk.tenv.insert(path!(u8));
//     is_ok!(tchk, a, b);

//     let a = tchk.tenv.insert(path!(f64));
//     let b = tchk.tenv.insert(path!(f64));
//     is_ok!(tchk, a, b);
//     let a = tchk.tenv.insert(path!(f32));
//     let b = tchk.tenv.insert(path!(f32));
//     is_ok!(tchk, a, b);

//     let a = tchk.tenv.insert(path!(s32));
//     let b = tchk.tenv.insert(path!(s64));
//     is_err!(tchk, a, b);
//     let a = tchk.tenv.insert(path!(f32));
//     let b = tchk.tenv.insert(path!(f64));
//     is_err!(tchk, a, b);

//     let a = tchk.tenv.insert(int!());
//     let b = tchk.tenv.insert(int!());
//     is_ok!(tchk, a, b);
//     let a = tchk.tenv.insert(float!());
//     let b = tchk.tenv.insert(float!());
//     is_err!(tchk, a, b);

//     let a = tchk.tenv.insert(unknown!());
//     let b = tchk.tenv.insert(int!());
//     is_ok!(tchk, a, b);
//     let a = tchk.tenv.insert(unknown!());
//     let u8 = tchk.tenv.insert(path!(u8));
//     let b = tchk.tenv.insert(int!(u8));
//     is_ok!(tchk, a, b);
//     let a = tchk.tenv.insert(unknown!());
//     let b = tchk.tenv.insert(float!());
//     is_ok!(tchk, a, b);
//     let a = tchk.tenv.insert(unknown!());
//     let f32 = tchk.tenv.insert(path!(f32));
//     let b = tchk.tenv.insert(float!(f32));
//     is_ok!(tchk, a, b);

//     let a = tchk.tenv.insert(unknown!());
//     let b = tchk.tenv.insert(unknown!());
//     is_ok!(tchk, a, b);

//     let a = tchk.tenv.insert(path!(Foo));
//     let b = tchk.tenv.insert(path!(Foo));
//     is_ok!(tchk, a, b);
//     let a = tchk.tenv.insert(path!(Foo));
//     let b = tchk.tenv.insert(path!(Bar));
//     is_err!(tchk, a, b);

//     let a = tchk.tenv.insert(never!());
//     let b = tchk.tenv.insert(path!(s32));
//     is_ok!(tchk, a, b);
//     let a = tchk.tenv.insert(never!());
//     let b = tchk.tenv.insert(path!(u32));
//     is_ok!(tchk, a, b);
//     let a = tchk.tenv.insert(never!());
//     let b = tchk.tenv.insert(path!(f32));
//     is_ok!(tchk, a, b);
//     let a = tchk.tenv.insert(never!());
//     let b = tchk.tenv.insert(path!(bool));
//     is_ok!(tchk, a, b);
//     let a = tchk.tenv.insert(never!());
//     let b = tchk.tenv.insert(int!());
//     is_ok!(tchk, a, b);
//     let a = tchk.tenv.insert(never!());
//     let b = tchk.tenv.insert(float!());
//     is_ok!(tchk, a, b);
//     let a = tchk.tenv.insert(never!());
//     let b = tchk.tenv.insert(unknown!());
//     is_ok!(tchk, a, b);
//     let a = tchk.tenv.insert(never!());
//     let b = tchk.tenv.insert(path!(Foo));
//     is_ok!(tchk, a, b);
// }

// #[test]
// fn generics() {
//     let mut tchk = TChecker::new(&INTERNER);

//     let a = tchk.tenv.insert(generic!(A));
//     let b = tchk.tenv.insert(generic!(B));
//     is_ok!(tchk, a, b);

//     let a = tchk.tenv.insert(generic!(A: restriction!(0, Foo)));
//     let b = tchk.tenv.insert(generic!(B));
//     is_err!(tchk, a, b);

//     let a = tchk.tenv.insert(generic!(A: restriction!(0, Foo)));
//     let b = tchk.tenv.insert(generic!(B: restriction!(0, Foo)));
//     is_ok!(tchk, a, b);

//     let a = tchk
//         .tenv
//         .insert(generic!(A: restriction!(0, Foo), restriction!(1, Bar)));
//     let b = tchk.tenv.insert(generic!(B: restriction!(0, Foo)));
//     is_err!(tchk, a, b);

//     let a = tchk
//         .tenv
//         .insert(generic!(A: restriction!(0, Foo), restriction!(1, Bar)));
//     let b = tchk
//         .tenv
//         .insert(generic!(B: restriction!(0, Foo), restriction!(1, Bar)));
//     is_ok!(tchk, a, b);

//     let impltor = tchk.tenv.insert(path!(FooStruct));
//     let impltor_args = vec![];

//     // apply FooTrait to FooStruct
//     declare_application!(tchk, 2 => vec![], impltor, impltor_args);

//     let a = tchk.tenv.insert(generic!(A: restriction!(2, FooTrait)));
//     is_ok!(tchk, a, impltor);

//     let non_impltor = tchk.tenv.insert(path!(BarStruct));
//     is_err!(tchk, a, non_impltor);

//     let s32 = tchk.tenv.insert(path!(s32));

//     // A: FooTrait<s32>
//     let a = tchk
//         .tenv
//         .insert(generic!(A: restriction!(2, FooTrait, vec![s32])));

//     // apply FooTrait<s32> to FooStruct
//     let impltor_args = vec![];
//     declare_application!(tchk, 2 => vec![s32], impltor, impltor_args);

//     // A: FooTrait<s32> == FooStruct
//     is_ok!(tchk, a, impltor);
// }
