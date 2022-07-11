use flux_lexer::T;

use super::*;

enum InfixOp {
	Add,
	Sub,
	Mul,
	Div,
	CmpEq,
	CmpNeq,
	CmpLt,
	CmpGt,
	CmpLte,
	CmpGte,
}

impl InfixOp {
	fn binding_power(&self) -> (u8, u8) {
		match self {
			Self::CmpEq | Self::CmpNeq | Self::CmpLt | Self::CmpGt | Self::CmpLte | Self::CmpGte => {
				(1, 2)
			}
			Self::Add | Self::Sub => (3, 4),
			Self::Mul | Self::Div => (4, 5),
		}
	}
}

enum PrefixOp {
	Neg,
}

impl PrefixOp {
	fn binding_power(&self) -> ((), u8) {
		match self {
			Self::Neg => ((), 5),
		}
	}
}

pub(crate) fn type_expr(p: &mut Parser) -> Option<CompletedMarker> {
	let result = if p.at(T![iN]) || p.at(T!(uN)) || p.at(T!(f32)) || p.at(T!(f64)) {
		primitive_type(p)
	} else if p.at(T!(ident)) {
		let m = p.start();
		p.bump();
		m.complete(p, SyntaxKind::IdentType)
	} else if p.at(T!(lparen)) {
		tuple_type(p)
	} else if p.at(T!(struct)) {
		struct_type(p)
	} else {
		p.error(format!("expected type expression"));
		return None;
	};

	Some(result)
}

fn tuple_type(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(T!(lparen)));
	let m = p.start();
	p.bump();
	while p.loop_safe_not_at(T!(rparen)) {
		type_expr(p);
		if p.at(T!(comma)) {
			p.bump();
		} else if !p.at(T!(rparen)) {
			p.error(format!("expected `)` at end of tuple type"));
		}
	}
	p.expect(T!(rparen), format!("expected `)` at end of tuple type"));
	m.complete(p, SyntaxKind::TupleType)
}

fn struct_type(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(T!(struct)));
	let m = p.start();
	p.bump();
	if p.at(T!(<)) {
		decl::generic_list(p);
	}
	p.expect(T!(lbrace), format!("expected `{{` in struct type"));
	while !p.at(T!(rbrace)) && !p.at_end() {
		struct_type_field(p);
	}
	p.expect(T!(rbrace), format!("expected `}}` at end of struct type"));
	m.complete(p, SyntaxKind::StructType)
}

fn struct_type_field(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	if p.at(T!(pub)) {
		p.bump();
	}
	if p.at(T!(mut)) {
		p.bump();
	}
	p.expect(
		TokenKind::Ident,
		format!("expected identifier for struct type field name"),
	);
	type_expr(p);
	p.expect(
		TokenKind::Comma,
		format!("expepcted `,` after struct type field"),
	);
	m.complete(p, SyntaxKind::StructTypeField)
}

fn primitive_type(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(T![iN]) || p.at(T![uN]) || p.at(T![f32]) || p.at(T![f64]));
	let m = p.start();
	p.bump();
	m.complete(p, SyntaxKind::PrimitiveType)
}

pub(crate) fn expr(p: &mut Parser, allow_struct_expressions: bool) -> Option<CompletedMarker> {
	expr_binding_power(p, 0, allow_struct_expressions)
}

fn expr_binding_power(
	p: &mut Parser,
	minimum_binding_power: u8,
	allow_struct_expressions: bool,
) -> Option<CompletedMarker> {
	let mut lhs = lhs(p, allow_struct_expressions)?;
	loop {
		let op = if p.at(T!(+)) {
			InfixOp::Add
		} else if p.at(T!(-)) {
			InfixOp::Sub
		} else if p.at(T!(*)) {
			InfixOp::Mul
		} else if p.at(T!(/)) {
			InfixOp::Div
		} else if p.at(T!(==)) {
			InfixOp::CmpEq
		} else if p.at(T!(!=)) {
			InfixOp::CmpNeq
		} else if p.at(T!(<)) {
			InfixOp::CmpLt
		} else if p.at(T!(>)) {
			InfixOp::CmpGt
		} else if p.at(T!(<=)) {
			InfixOp::CmpLte
		} else if p.at(T!(>=)) {
			InfixOp::CmpGte
		} else {
			break;
		};
		let (left_binding_power, right_binding_power) = op.binding_power();

		if left_binding_power < minimum_binding_power {
			break;
		}

		p.bump();

		let m = lhs.precede(p);
		let parsed_rhs = expr_binding_power(p, right_binding_power, allow_struct_expressions).is_some();
		lhs = m.complete(p, SyntaxKind::BinExpr);

		if !parsed_rhs {
			break;
		}
	}
	if minimum_binding_power == 0 {
		lhs = postfix(p, lhs, allow_struct_expressions);
	}
	Some(lhs)
}

pub(crate) fn path(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.expect(T!(ident), format!("expected identifier in path"));
	while p.at(T!(::)) && !p.at_end() {
		p.bump();
		p.expect(T!(ident), format!("expected identifier in path"));
	}
	m.complete(p, SyntaxKind::PathExpr)
}

fn postfix(p: &mut Parser, e: CompletedMarker, allow_struct_expressions: bool) -> CompletedMarker {
	if p.at(T!(lparen)) {
		call(p, e)
	} else if allow_struct_expressions && p.at(T!(lbrace)) {
		struct_initialization(p, e)
	} else {
		e
	}
}

fn struct_initialization(p: &mut Parser, e: CompletedMarker) -> CompletedMarker {
	assert!(p.at(T!(lbrace)));
	let m = e.precede(p);
	p.bump();

	while !p.at(T!(rbrace)) && !p.at_end() {
		let m = p.start();
		ident_expr(p);
		if p.at(T!(comma)) {
			p.bump();
		} else {
			p.expect(
				T!(:),
				format!("expected `:` in struct field initialization"),
			);
			expr(p, true);
			if !p.at(T!(rbrace)) {
				p.expect(
					T!(comma),
					format!("expected `,` after struct field initialization"),
				);
			}
		}
		m.complete(p, SyntaxKind::StructExprField);
	}

	p.expect(
		T!(rbrace),
		format!("expected `}}` after struct initialization"),
	);

	m.complete(p, SyntaxKind::StructExpr)
}

fn call(p: &mut Parser, e: CompletedMarker) -> CompletedMarker {
	let m = e.precede(p);

	p.bump();

	while !p.at(T!(rparen)) && !p.at_end() {
		expr(p, true);
		if !p.at(T!(rparen)) {
			if !p.at(T!(comma)) {
				p.error(format!("expected either `,` or `)` in call expression"));
				break;
			} else {
				p.bump();
			}
		}
	}
	p.expect(T!(rparen), format!("expected `)` in call expression"));

	m.complete(p, SyntaxKind::CallExpr)
}

fn lhs(p: &mut Parser, allow_struct_expressions: bool) -> Option<CompletedMarker> {
	let cm = if p.at(T!(intlit)) {
		int_lit(p)
	} else if p.at(T!(floatlit)) {
		float_lit(p)
	} else if p.at(T!(ident)) {
		ident_expr(p)
	} else if p.at(TokenKind::Minus) {
		prefix_neg(p, allow_struct_expressions)
	} else if p.at(T!(lparen)) {
		paren_or_tuple_expr(p, allow_struct_expressions)
	} else if p.at(T!(if)) {
		if_expr(p)
	} else if p.at(T!(lbrace)) {
		block_expr(p)
	} else {
		p.error(format!("expected expression lhs"));
		return None;
	};

	Some(cm)
}

fn int_lit(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.expect(TokenKind::IntLit, format!("expected int literal"));
	m.complete(p, SyntaxKind::IntExpr)
}

fn float_lit(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.expect(TokenKind::FloatLit, format!("expected float literal"));
	m.complete(p, SyntaxKind::FloatExpr)
}

pub(crate) fn ident_expr(p: &mut Parser) -> CompletedMarker {
	path(p)
}

fn prefix_neg(p: &mut Parser, allow_struct_expressions: bool) -> CompletedMarker {
	let m = p.start();

	let op = PrefixOp::Neg;
	let (_, right_binding_power) = op.binding_power();
	p.bump();
	expr_binding_power(p, right_binding_power, allow_struct_expressions);

	m.complete(p, SyntaxKind::PrefixExpr)
}

fn paren_or_tuple_expr(p: &mut Parser, allow_struct_expressions: bool) -> CompletedMarker {
	assert!(p.at(TokenKind::LParen));
	let m = p.start();
	p.bump();
	expr(p, allow_struct_expressions);
	if p.at(T!(comma)) {
		p.bump();

		while p.loop_safe_not_at(T!(rparen)) {
			expr(p, true);
			if p.at(T!(comma)) {
				p.bump();
			} else if !p.at(T!(rparen)) {
				p.error(format!("expected `)` at end of tuple expression"));
			}
		}

		p.bump();
		return m.complete(p, SyntaxKind::TupleExpr);
	}
	p.expect(
		TokenKind::RParen,
		format!("expected `)` at end of paren expression"),
	);
	m.complete(p, SyntaxKind::ParenExpr)
}

fn if_expr(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.expect(T!(if), format!("expected `if` in if statement"));
	expr::expr(p, false);
	expr::block_expr(p);
	if p.at(T!(else)) {
		p.bump();
		if p.at(T!(if)) {
			if_expr(p);
		} else {
			expr::block_expr(p);
		}
	}
	m.complete(p, SyntaxKind::IfExpr)
}

pub(crate) fn block_expr(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.expect(T![lbrace], format!("expected `{{` at start of block"));
	while p.loop_safe_not_at(T![rbrace]) {
		stmt::stmt(p);
	}
	p.expect(T![rbrace], format!("expected `}}` at end of block"));
	m.complete(p, SyntaxKind::BlockExpr)
}

#[cfg(test)]
mod tests {
	use crate::test_expr_str;

	// -- Integers --

	test_expr_str!(basic_literals_b10_int, "1");
	test_expr_str!(basic_literals_b10_int_neg, "-10");
	test_expr_str!(basic_literals_b10_int_sep, "1_000_000");
	test_expr_str!(basic_literals_b10_int_neg_sep, "-1_000_000");
	test_expr_str!(basic_literals_b16_int, "0xff");
	test_expr_str!(basic_literals_b16_int_neg, "-0xabcdef");
	test_expr_str!(basic_literals_b16_int_sep, "0xa_b_ff");
	test_expr_str!(basic_literals_b16_int_neg_sep, "-0xa_b_ff");
	test_expr_str!(basic_literals_b2_int, "0b1010001");
	test_expr_str!(basic_literals_b2_int_neg, "-0b1010001");
	test_expr_str!(basic_literals_b2_int_sep, "0b1_010_001");
	test_expr_str!(basic_literals_b2_int_neg_sep, "-0b1_010_001");

	// -- Floats --

	test_expr_str!(basic_literals_float, "1.0");
	test_expr_str!(basic_literals_float_neg, "-1.0");
	test_expr_str!(basic_literals_float_sep, "1.0_000");
	test_expr_str!(basic_literals_float_neg_sep, "-1.0_000");

	// ------- Ident -------

	test_expr_str!(ident, "foo");
	test_expr_str!(ident_nums, "f0o");
	test_expr_str!(ident_nums_seps, "f0o_B8rr");

	// ------- BinOp -------

	test_expr_str!(binops_int_plus, "1+1");
	test_expr_str!(binops_int_minus, "0xff-0834");
	test_expr_str!(binops_int_mult, "0b1001*250");
	test_expr_str!(binops_int_div, "919/0xadb");
	test_expr_str!(binops_float_plus, "1.02+2.40_14");
	test_expr_str!(binops_float_minus, "92.12_10-0.25");
	test_expr_str!(binops_float_mult, "-2.13*0.2_5");
	test_expr_str!(binops_float_div, "-0.1_2/1.0");
	test_expr_str!(binops_cmp_lt, "-1.0_1<1");
	test_expr_str!(binops_cmp_lte, "1<=1");
	test_expr_str!(binops_cmp_gt, "1>1");
	test_expr_str!(binops_cmp_gte, "1>=1");
	test_expr_str!(binops_cmp_eq, "1==1");
	test_expr_str!(binops_cmp_ne, "1!=1");
	test_expr_str!(binops_and, "1&&1");
	test_expr_str!(binops_or, "1||1");
	test_expr_str!(binops_eq, "x=1");
	test_expr_str!(binops_period, "foo.bar");
	test_expr_str!(binops_double_colon, "foo::bar");

	// ------- Call ---------
	test_expr_str!(call_basic, "foo()");
	test_expr_str!(call_double_colon, "foo::bar::bazz()");

	test_expr_str!(
		infix_with_comments,
		r#"// hello
		-1 + 1"#
	);
	// test_expr_str!(unclosed_paren_expr, "(foo");
}
