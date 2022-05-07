// use paste::paste;
// use flux_error::filesystem::FileId;
// use flux_lexer::tokenize;
// use flux_parser::{
// 	expr::{expr, type_expr},
// 	ParseInput,
// };
// use std::fmt::Write;

// #[path = "utils/mod.rs"]
// mod utils;

// // ------- Basic Literals -------

// // -- Integers --

// test_expr_str!(basic_literals_b10_int, "1");
// test_expr_str!(basic_literals_b10_int_neg, "-10");
// test_expr_str!(basic_literals_b10_int_sep, "1_000_000");
// test_expr_str!(basic_literals_b10_int_neg_sep, "-1_000_000");
// test_expr_str!(basic_literals_b16_int, "0xff");
// test_expr_str!(basic_literals_b16_int_neg, "-0xabcdef");
// test_expr_str!(basic_literals_b16_int_sep, "0xa_b_ff");
// test_expr_str!(basic_literals_b16_int_neg_sep, "-0xa_b_ff");
// test_expr_str!(basic_literals_b8_int, "081234567");
// test_expr_str!(basic_literals_b8_int_neg, "-0812345678");
// test_expr_str!(basic_literals_b8_int_sep, "081_234_567");
// test_expr_str!(basic_literals_b8_int_neg_sep, "-081_234_567");
// test_expr_str!(basic_literals_b2_int, "0b1010001");
// test_expr_str!(basic_literals_b2_int_neg, "-0b1010001");
// test_expr_str!(basic_literals_b2_int_sep, "0b1_010_001");
// test_expr_str!(basic_literals_b2_int_neg_sep, "-0b1_010_001");

// // -- Floats --

// test_expr_str!(basic_literals_float, "1.0");
// test_expr_str!(basic_literals_float_neg, "-1.0");
// test_expr_str!(basic_literals_float_sep, "1.0_000");
// test_expr_str!(basic_literals_float_neg_sep, "-1.0_000");

// // ------- Ident -------

// test_expr_str!(ident, "foo");
// test_expr_str!(ident_nums, "f0o");
// test_expr_str!(ident_nums_seps, "f0o_B8rr");

// // ------- BinOp -------

// test_expr_str!(binops_int_plus, "1+1");
// test_expr_str!(binops_int_minus, "0xff-0834");
// test_expr_str!(binops_int_mult, "0b1001*250");
// test_expr_str!(binops_int_div, "919/0xadb");
// test_expr_str!(binops_float_plus, "1.02+2.40_14");
// test_expr_str!(binops_float_minus, "92.12_10-0.25");
// test_expr_str!(binops_float_mult, "-2.13*0.2_5");
// test_expr_str!(binops_float_div, "-0.1_2/1.0");
// test_expr_str!(binops_cmp_lt, "-0.1_2<1");
// test_expr_str!(binops_cmp_lte, "1<=1");
// test_expr_str!(binops_cmp_gt, "1>1");
// test_expr_str!(binops_cmp_gte, "1>=1");
// test_expr_str!(binops_cmp_eq, "1==1");
// test_expr_str!(binops_cmp_ne, "1!=1");
// test_expr_str!(binops_and, "1&&1");
// test_expr_str!(binops_or, "1||1");
// test_expr_str!(binops_eq, "x=1");
// test_expr_str!(binops_period, "foo.bar");
// test_expr_str!(binops_double_colon, "foo::bar");

// // ------- Types -------

// // -- Primitives --

// test_type_expr_str!(types_prims_i64, "i64");
// test_type_expr_str!(types_prims_u64, "u64");
// test_type_expr_str!(types_prims_i32, "i32");
// test_type_expr_str!(types_prims_u32, "u32");
// test_type_expr_str!(types_prims_i16, "i16");
// test_type_expr_str!(types_prims_u16, "u16");
// test_type_expr_str!(types_prims_i8, "i8");
// test_type_expr_str!(types_prims_u8, "u8");
// test_type_expr_str!(types_prims_f64, "f64");
// test_type_expr_str!(types_prims_f32, "f32");
// // test_expr_str!(types_prims_bool, "bool");

// // -- Pointers --

// test_type_expr_str!(types_ptr_prim, "i32*");
// test_type_expr_str!(types_ptr_ident, "foo*");

// // -- Ident --

// test_type_expr_str!(types_ident, "foo");

// // -- Struct --

// // test_type_expr_str!(
// // 	types_struct,
// // 	r#"struct {
// // 	i32 x = 0;
// // 	pub u64 y;
// // 	Foo foo;
// // 	Bar *bar;
// // }"#
// // );

// // test_type_expr_str!(
// // 	types_interface,
// // 	r#"interface {
// // 		pub fn foo(this);
// // 		fn bar(mut this) -> i32;
// // 		fn bazz(i32 x, mut u8 y) -> void;
// // }"#
// // );
