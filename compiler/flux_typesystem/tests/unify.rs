use flux_proc_macros::tenv;

#[rustfmt::skip]
#[test]
fn primitives() {
    tenv! {
			Unify {
				u64 == u64;
				u32 == u32;
				u16 == u16;
				u8 == u8;
				s64 == s64;
				s32 == s32;
				s16 == s16;
				s8 == s8;

				u64 != s64;
				u32 != s32;
				u16 != s16;
				u8 != s8;

				int == int;
				int == int(s32);
				int(u8) != int(s32);

				float == float;
				float == float(f32);
				float(f32) != float(f64);

				never == never;
				unknown == unknown;
			}
    }
}

#[rustfmt::skip]
#[test]
fn foo() {
    tenv! {
        With {
            A: [Foo, Bar<B>],
            B: [Foo]
        }
        Unify {
            s32<u8<f64>, f32> == u32;
                        Foo<A> == Bar;
        }
    };
}
