use flux_span::{FileId, Span, WithSpan};
use lasso::ThreadedRodeo;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{ConcreteKind, TChecker, TEnv, TraitRestriction, Type, TypeId, TypeKind};

static INTERNER: Lazy<ThreadedRodeo> = Lazy::new(ThreadedRodeo::new);

fn str_to_tid(s: &str, env: &mut TEnv) -> TypeId {
    let mut kind = TypeKind::Concrete(ConcreteKind::Path(INTERNER.get_or_intern(s)));
    let unknown_regex = Regex::new(r"unknown").unwrap();
    let int_regex = Regex::new(r"int(\(.+\))?").unwrap();
    let float_regex = Regex::new(r"float(\(.+\))?").unwrap();

    if let Some(captures) = int_regex.captures(s) {
        if let Some(depend_type) = captures.get(1) {
            let s = depend_type.as_str();
            let s = &s[1..s.len() - 1];
            let tid = str_to_tid(s, env);
            kind = TypeKind::Int(Some(tid));
        } else {
            kind = TypeKind::Int(None);
        }
    } else if let Some(captures) = float_regex.captures(s) {
        if let Some(depend_type) = captures.get(1) {
            let s = depend_type.as_str();
            let s = &s[1..s.len() - 1];
            let tid = str_to_tid(s, env);
            kind = TypeKind::Float(Some(tid));
        } else {
            kind = TypeKind::Float(None);
        }
    } else if unknown_regex.is_match(s) {
        kind = TypeKind::Unknown;
    }

    env.insert(Type::new(kind).file_span(FileId::poisoned(), Span::poisoned()))
}

fn parse(content: &str) -> String {
    let mut tchk = TChecker::new(&INTERNER);
    let lines = content.split("\n");

    let mut s = String::new();

    for line in lines {
        let line = line.trim();
        if line.len() == 0 {
            continue;
        }
        let mut segments = line.split(" ");
        let t1 = segments
            .next()
            .expect("test input malformatted: expected type");
        let op = segments
            .next()
            .expect("test input malformatted: expected operator");
        let t2 = segments
            .next()
            .expect("test input malformatted: expected type");

        let tid1 = str_to_tid(t1, &mut tchk.tenv);
        let tid2 = str_to_tid(t2, &mut tchk.tenv);
        s += line;

        let result = tchk.unify(tid1, tid2, Span::poisoned().in_file(FileId::poisoned()));
        if op == "==" {
            if result.is_ok() {
                s += " ✅\n";
            } else {
                s += " ❌\n";
            }
        } else if op == "!=" {
            if result.is_ok() {
                s += " ❌\n";
            } else {
                s += " ✅\n";
            }
        }
    }

    s
}

macro_rules! unify {
    ($name:ident, $content:literal) => {
        paste::paste! {
            #[test]
            fn [<unify_literals_ $name>]() {
                insta::assert_snapshot!(parse($content));
            }
        }
    };
}

unify!(
    primitives,
    r#"
    s64 == s64
    s32 == s32
    s16 == s16
    s8 == s8
    s64 == s64
    s32 == s32
    s16 == s16
    s8 == s8
    f64 == f64
    f32 == f32

    s32 != s16
    f32 != f64
    int != int
    float != float

    unknown == int
    unknown == int(u8)
    unknown == float
    unknown == float(f64)

    unknown == unknown

    Foo == Foo
    Foo != Bar
"#
);

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

macro_rules! declare_application {
    ($tchk:expr, $trait_name:ident with $trait_args:expr, $impltor:expr) => {
        $tchk.add_trait_application_to_context(
            &filespanned!(INTERNER.get_or_intern(stringify!($trait_name))),
            $trait_args,
            $impltor,
        )
    };
}

#[test]
fn generics() {
    let mut tchk = TChecker::new(&INTERNER);

    let a = tchk.tenv.insert(generic!(A));
    let b = tchk.tenv.insert(generic!(B));

    assert!(tchk.unify(a, b, filespan!()).is_ok());

    let a = tchk.tenv.insert(generic!(A: restriction!(Foo)));
    let b = tchk.tenv.insert(generic!(B));
    assert!(tchk.unify(a, b, filespan!()).is_err());

    let a = tchk.tenv.insert(generic!(A: restriction!(Foo)));
    let b = tchk.tenv.insert(generic!(B: restriction!(Foo)));
    assert!(tchk.unify(a, b, filespan!()).is_ok());

    let a = tchk
        .tenv
        .insert(generic!(A: restriction!(Foo), restriction!(Bar)));
    let b = tchk.tenv.insert(generic!(B: restriction!(Foo)));
    assert!(tchk.unify(a, b, filespan!()).is_err());

    let a = tchk
        .tenv
        .insert(generic!(A: restriction!(Foo), restriction!(Bar)));
    let b = tchk
        .tenv
        .insert(generic!(B: restriction!(Foo), restriction!(Bar)));
    assert!(tchk.unify(a, b, filespan!()).is_ok());

    declare_trait!(tchk, FooTrait);

    let impltor = tchk.tenv.insert(path!(FooStruct));
    assert!(declare_application!(tchk, FooTrait with &[], impltor).is_ok());

    let a = tchk.tenv.insert(generic!(A: restriction!(FooTrait)));
    assert!(tchk.unify(a, impltor, filespan!()).is_ok());

    let non_impltor = tchk.tenv.insert(path!(BarStruct));
    assert!(tchk.unify(a, non_impltor, filespan!()).is_err());

    let s32 = tchk.tenv.insert(path!(s32));

    // A: FooTrait<s32>
    let a = tchk
        .tenv
        .insert(generic!(A: restriction!(FooTrait, vec![s32])));

    // apply FooTrait<s32> to FooStruct
    assert!(declare_application!(tchk, FooTrait with &[s32], impltor).is_ok());

    // A: FooTrait<s32> == FooStruct
    assert!(tchk.unify(a, impltor, filespan!()).is_ok());
}
