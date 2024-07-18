use flux_diagnostics::SourceCache;
use flux_span::{Interner, Span, WithSpan};
use flux_typesystem::{TChecker, TEnv, TypeKind::*};

use crate::helpers::{Primitives, Traits};

mod helpers;

#[test]
fn bar() {
    let interner = Box::leak(Box::new(Interner::new()));
    let mut source_cache = SourceCache::new(interner);
    let file_id = source_cache.add_input_file("test.flx", String::new());
    let span = Span::poisoned();
    let file_span = span.in_file(file_id);
    let mut tenv = TEnv::new(interner);
    let mut tchk = TChecker::new(&mut tenv);

    let primitives = Primitives::new(file_id);

    let a = primitives.s16(&mut tchk);
    let b = primitives.s16(&mut tchk);
    assert!(tchk.unify(a, b, file_span).is_ok());

    let a = primitives.u64(&mut tchk);
    let b = primitives.u32(&mut tchk);
    assert!(tchk.unify(a, b, file_span).is_err());

    // Int

    let a = primitives.int(&mut tchk, None);
    let b = primitives.u32(&mut tchk);
    assert!(tchk.unify(a, b, file_span).is_ok());
    assert!(matches!(tchk.tenv.get(&a).inner.inner, Int(Some(tid)) if tid == b));

    let a = primitives.int(&mut tchk, None);
    let b = primitives.int(&mut tchk, None);
    assert!(tchk.unify(a, b, file_span).is_ok());

    let a = primitives.int(&mut tchk, None);
    let intermediate = primitives.u32(&mut tchk);
    let b = primitives.int(&mut tchk, Some(intermediate));
    assert!(tchk.unify(a, b, file_span).is_ok());
    assert!(matches!(tchk.tenv.get(&a).inner.inner, Int(Some(tid)) if tid == intermediate));

    // Float

    let a = primitives.float(&mut tchk, None);
    let b = primitives.f32(&mut tchk);
    assert!(tchk.unify(a, b, file_span).is_ok());
    assert!(matches!(tchk.tenv.get(&a).inner.inner, Float(Some(tid)) if tid == b));

    let a = primitives.float(&mut tchk, None);
    let b = primitives.float(&mut tchk, None);
    assert!(tchk.unify(a, b, file_span).is_ok());

    let a = primitives.float(&mut tchk, None);
    let intermediate = primitives.f32(&mut tchk);
    let b = primitives.float(&mut tchk, Some(intermediate));
    assert!(tchk.unify(a, b, file_span).is_ok());
    assert!(matches!(tchk.tenv.get(&a).inner.inner, Float(Some(tid)) if tid == intermediate));

    // Unknown
    let a = primitives.int(&mut tchk, None);
    let b = primitives.unknown(&mut tchk);
    assert!(tchk.unify(a, b, file_span).is_ok());
    assert!(matches!(tchk.tenv.get(&b).inner.inner, Int(None)));

    let intermediate = primitives.u32(&mut tchk);
    let a = primitives.int(&mut tchk, Some(intermediate));
    let b = primitives.unknown(&mut tchk);
    assert!(tchk.unify(a, b, file_span).is_ok());
    assert!(matches!(tchk.tenv.get(&b).inner.inner, Int(Some(tid)) if tid == intermediate));

    // Generics & Traits

    let traits = Traits::new(interner, file_id, span);

    let a = traits.generic(&mut tchk, "A", vec![]);
    let b = traits.generic(&mut tchk, "B", vec![]);
    assert!(tchk.unify(a, b, file_span).is_ok());

    let foo_trait_restriction = traits.restriction("Foo", vec![]);
    let a = traits.generic(&mut tchk, "A", vec![foo_trait_restriction.clone()]);
    let b = traits.generic(&mut tchk, "B", vec![foo_trait_restriction]);
    assert!(tchk.unify(a, b, file_span).is_ok());
}

// #[rustfmt::skip]
// #[test]
// fn primitives() {
//     tenv! {
// 			Unify {
// 				u64 == u64;
// 				u32 == u32;
// 				u16 == u16;
// 				u8 == u8;
// 				s64 == s64;
// 				s32 == s32;
// 				s16 == s16;
// 				s8 == s8;

// 				u64 != s64;
// 				u32 != s32;
// 				u16 != s16;
// 				u8 != s8;

// 				int == int;
// 				int == int(s32);
// 				int(u8) != int(s32);

// 				float == float;
// 				float == float(f32);
// 				float(f32) != float(f64);

// 				never == never;
// 				unknown == unknown;
// 			}
//     }
// }

// #[rustfmt::skip]
// #[test]
// fn foo() {
//     tenv! {
//         With {
//             A: [Foo, Bar<B>],
//             B: [Foo]
//         }
//         Unify {
//             s32<u8<f64>, f32> == u32;
//                         Foo<A> == Bar;
//         }
//     };
// }
