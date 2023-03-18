use flux_span::{FileId, Span, WithSpan};
use lasso::ThreadedRodeo;
use once_cell::sync::Lazy;

use crate::{ConcreteKind, TChecker, TraitRestriction, Type, TypeKind};

static INTERNER: Lazy<ThreadedRodeo> = Lazy::new(ThreadedRodeo::new);

macro_rules! filespanned {
    ($e:expr) => {
        $e.file_span(FileId::poisoned(), Span::poisoned())
    };
}

macro_rules! filespan {
    () => {
        Span::poisoned().in_file(FileId::poisoned())
    };
}

macro_rules! generic {
    ($name:ident) => {
        filespanned!(Type::new(TypeKind::Generic(
            INTERNER.get_or_intern(stringify!($name)),
            vec![]
        )))
    };
    ($name:ident: $(
        $restrictions:expr
    ),*) => {
        filespanned!(Type::new(TypeKind::Generic(
            INTERNER.get_or_intern(stringify!($name)),
            vec![$($restrictions),*]
        )))
    };
}

macro_rules! restriction {
    ($path:ident) => {
        TraitRestriction {
            name: filespanned!(INTERNER.get_or_intern(stringify!($path))),
            args: vec![],
        }
    };
    ($path:ident, $args:expr) => {
        TraitRestriction {
            name: filespanned!(INTERNER.get_or_intern(stringify!($path))),
            args: $args,
        }
    };
}

macro_rules! declare_trait {
    ($tchk:expr, $name:ident) => {
        $tchk.add_trait_to_context(INTERNER.get_or_intern(stringify!($name)))
    };
}

macro_rules! path {
    ($name:ident) => {
        filespanned!(Type::new(TypeKind::Concrete(ConcreteKind::Path(
            INTERNER.get_or_intern(stringify!($name))
        ))))
    };
}

macro_rules! int {
    () => {
        filespanned!(Type::new(TypeKind::Int(None)))
    };
    ($depends_on:expr) => {
        filespanned!(Type::new(TypeKind::Int(Some($depends_on))))
    };
}

macro_rules! float {
    () => {
        filespanned!(Type::new(TypeKind::Float(None)))
    };
    ($depends_on:expr) => {
        filespanned!(Type::new(TypeKind::Float(Some($depends_on))))
    };
}

macro_rules! unknown {
    () => {
        filespanned!(Type::new(TypeKind::Unknown))
    };
}

macro_rules! declare_application {
    ($tchk:expr, $trait_name:ident with $trait_args:expr, $impltor:expr) => {
        $tchk.add_trait_application_to_context(
            &filespanned!(INTERNER.get_or_intern(stringify!($trait_name))),
            $trait_args,
            $impltor,
        )
    };
}

macro_rules! is_ok {
    ($tchk:expr, $a:expr, $b:expr) => {
        assert!($tchk.unify($a, $b, filespan!()).is_ok());
    };
}

macro_rules! is_err {
    ($tchk:expr, $a:expr, $b:expr) => {
        assert!($tchk.unify($a, $b, filespan!()).is_err());
    };
}

#[test]
fn primitives() {
    let mut tchk = TChecker::new(&INTERNER);
    let a = tchk.tenv.insert(path!(s64));
    let b = tchk.tenv.insert(path!(s64));
    is_ok!(tchk, a, b);
    let a = tchk.tenv.insert(path!(s32));
    let b = tchk.tenv.insert(path!(s32));
    is_ok!(tchk, a, b);
    let a = tchk.tenv.insert(path!(s16));
    let b = tchk.tenv.insert(path!(s16));
    is_ok!(tchk, a, b);
    let a = tchk.tenv.insert(path!(s8));
    let b = tchk.tenv.insert(path!(s8));
    is_ok!(tchk, a, b);
    let a = tchk.tenv.insert(path!(u64));
    let b = tchk.tenv.insert(path!(u64));
    is_ok!(tchk, a, b);
    let a = tchk.tenv.insert(path!(u32));
    let b = tchk.tenv.insert(path!(u32));
    is_ok!(tchk, a, b);
    let a = tchk.tenv.insert(path!(u16));
    let b = tchk.tenv.insert(path!(u16));
    is_ok!(tchk, a, b);
    let a = tchk.tenv.insert(path!(u8));
    let b = tchk.tenv.insert(path!(u8));
    is_ok!(tchk, a, b);

    let a = tchk.tenv.insert(path!(f64));
    let b = tchk.tenv.insert(path!(f64));
    is_ok!(tchk, a, b);
    let a = tchk.tenv.insert(path!(f32));
    let b = tchk.tenv.insert(path!(f32));
    is_ok!(tchk, a, b);

    let a = tchk.tenv.insert(path!(s32));
    let b = tchk.tenv.insert(path!(s64));
    is_err!(tchk, a, b);
    let a = tchk.tenv.insert(path!(f32));
    let b = tchk.tenv.insert(path!(f64));
    is_err!(tchk, a, b);

    let a = tchk.tenv.insert(int!());
    let b = tchk.tenv.insert(int!());
    is_err!(tchk, a, b);
    let a = tchk.tenv.insert(float!());
    let b = tchk.tenv.insert(float!());
    is_err!(tchk, a, b);

    let a = tchk.tenv.insert(unknown!());
    let b = tchk.tenv.insert(int!());
    is_ok!(tchk, a, b);
    let a = tchk.tenv.insert(unknown!());
    let u8 = tchk.tenv.insert(path!(u8));
    let b = tchk.tenv.insert(int!(u8));
    is_ok!(tchk, a, b);
    let a = tchk.tenv.insert(unknown!());
    let b = tchk.tenv.insert(float!());
    is_ok!(tchk, a, b);
    let a = tchk.tenv.insert(unknown!());
    let f32 = tchk.tenv.insert(path!(f32));
    let b = tchk.tenv.insert(float!(f32));
    is_ok!(tchk, a, b);

    let a = tchk.tenv.insert(unknown!());
    let b = tchk.tenv.insert(unknown!());
    is_ok!(tchk, a, b);

    let a = tchk.tenv.insert(path!(Foo));
    let b = tchk.tenv.insert(path!(Foo));
    is_ok!(tchk, a, b);
    let a = tchk.tenv.insert(path!(Foo));
    let b = tchk.tenv.insert(path!(Bar));
    is_err!(tchk, a, b);
}

#[test]
fn generics() {
    let mut tchk = TChecker::new(&INTERNER);

    let a = tchk.tenv.insert(generic!(A));
    let b = tchk.tenv.insert(generic!(B));
    is_ok!(tchk, a, b);

    let a = tchk.tenv.insert(generic!(A: restriction!(Foo)));
    let b = tchk.tenv.insert(generic!(B));
    is_err!(tchk, a, b);

    let a = tchk.tenv.insert(generic!(A: restriction!(Foo)));
    let b = tchk.tenv.insert(generic!(B: restriction!(Foo)));
    is_ok!(tchk, a, b);

    let a = tchk
        .tenv
        .insert(generic!(A: restriction!(Foo), restriction!(Bar)));
    let b = tchk.tenv.insert(generic!(B: restriction!(Foo)));
    is_err!(tchk, a, b);

    let a = tchk
        .tenv
        .insert(generic!(A: restriction!(Foo), restriction!(Bar)));
    let b = tchk
        .tenv
        .insert(generic!(B: restriction!(Foo), restriction!(Bar)));
    is_ok!(tchk, a, b);

    declare_trait!(tchk, FooTrait);

    let impltor = tchk.tenv.insert(path!(FooStruct));
    assert!(declare_application!(tchk, FooTrait with &[], impltor).is_ok());

    let a = tchk.tenv.insert(generic!(A: restriction!(FooTrait)));
    is_ok!(tchk, a, impltor);

    let non_impltor = tchk.tenv.insert(path!(BarStruct));
    is_err!(tchk, a, non_impltor);

    let s32 = tchk.tenv.insert(path!(s32));

    // A: FooTrait<s32>
    let a = tchk
        .tenv
        .insert(generic!(A: restriction!(FooTrait, vec![s32])));

    // apply FooTrait<s32> to FooStruct
    assert!(declare_application!(tchk, FooTrait with &[s32], impltor).is_ok());

    // A: FooTrait<s32> == FooStruct
    is_ok!(tchk, a, impltor);
}
