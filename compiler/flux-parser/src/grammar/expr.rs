use crate::{recovery, EXPR_RECOVERY_SET};

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
	Access,
	Assign,
}

impl InfixOp {
	fn binding_power(&self) -> (u8, u8) {
		match self {
			Self::Assign => (1, 2),
			Self::CmpEq | Self::CmpNeq | Self::CmpLt | Self::CmpGt | Self::CmpLte | Self::CmpGte => {
				(3, 4)
			}
			Self::Add | Self::Sub => (5, 6),
			Self::Mul | Self::Div => (7, 8),
			Self::Access => (9, 10),
		}
	}
}

enum PrefixOp {
	Neg,
	Addr,
}

impl PrefixOp {
	fn binding_power(&self) -> ((), u8) {
		match self {
			Self::Neg => ((), 5),
			Self::Addr => ((), 10), // TODO: not sure
		}
	}
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
		let op = if p.at(TokenKind::Plus) {
			InfixOp::Add
		} else if p.at(TokenKind::Minus) {
			InfixOp::Sub
		} else if p.at(TokenKind::Star) {
			InfixOp::Mul
		} else if p.at(TokenKind::Slash) {
			InfixOp::Div
		} else if p.at(TokenKind::CmpEq) {
			InfixOp::CmpEq
		} else if p.at(TokenKind::CmpNeq) {
			InfixOp::CmpNeq
		} else if p.at(TokenKind::CmpLt) {
			InfixOp::CmpLt
		} else if p.at(TokenKind::CmpGt) {
			InfixOp::CmpGt
		} else if p.at(TokenKind::CmpLte) {
			InfixOp::CmpLte
		} else if p.at(TokenKind::CmpGte) {
			InfixOp::CmpGte
		} else if p.at(TokenKind::Period) {
			InfixOp::Access
		} else if p.at(TokenKind::Eq) {
			InfixOp::Assign
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
		lhs = postfix(p, lhs, allow_struct_expressions);

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
	p.expect(TokenKind::Ident, &recovery(&[TokenKind::DoubleColon]));
	while p.at(TokenKind::DoubleColon) && !p.at_end() {
		p.bump();
		p.expect(TokenKind::Ident, &recovery(&[TokenKind::DoubleColon]));
	}
	m.complete(p, SyntaxKind::PathExpr)
}

fn postfix(p: &mut Parser, e: CompletedMarker, allow_struct_expressions: bool) -> CompletedMarker {
	if p.at(TokenKind::LParen) {
		call(p, e)
	} else if allow_struct_expressions && p.at(TokenKind::LBrace) {
		struct_initialization(p, e)
	} else if p.at(TokenKind::LSquare) {
		index_memory(p, e)
	} else {
		e
	}
}

fn index_memory(p: &mut Parser, e: CompletedMarker) -> CompletedMarker {
	assert!(p.at(TokenKind::LSquare));
	let m = e.precede(p);
	p.bump();
	expr(p, true); // TODO: should this be true?
	p.expect(TokenKind::RSquare, &recovery(&[]));
	m.complete(p, SyntaxKind::IndexMemoryExpr)
}

fn struct_initialization(p: &mut Parser, e: CompletedMarker) -> CompletedMarker {
	assert!(p.at(TokenKind::LBrace));
	let m = e.precede(p);
	p.bump();

	while !p.at(TokenKind::RBrace) && !p.at_end() {
		let m = p.start();
		p.expect(TokenKind::Ident, &recovery(&[TokenKind::Comma]));
		if p.at(TokenKind::Comma) {
			p.bump();
		} else {
			p.expect(TokenKind::Colon, &recovery(EXPR_RECOVERY_SET));
			expr(p, true);
			if !p.at(TokenKind::RBrace) {
				p.expect(TokenKind::Comma, &recovery(&[TokenKind::Ident]));
			}
		}
		m.complete(p, SyntaxKind::StructExprField);
	}

	p.expect(TokenKind::RBrace, &recovery(&[]));
	m.complete(p, SyntaxKind::StructExpr)
}

fn call(p: &mut Parser, e: CompletedMarker) -> CompletedMarker {
	let m = e.precede(p);

	p.bump();

	while !p.at(TokenKind::RParen) && !p.at_end() {
		expr(p, true);
		if !p.at(TokenKind::RParen) {
			if !p.at(TokenKind::Comma) {
				p.expected(format!("either `,` or `)` in call expression"));
				break;
			} else {
				p.bump();
			}
		}
	}
	p.expect(TokenKind::RParen, &recovery(&[]));

	m.complete(p, SyntaxKind::CallExpr)
}

fn lhs(p: &mut Parser, allow_struct_expressions: bool) -> Option<CompletedMarker> {
	let cm = if p.at(TokenKind::IntLit) {
		int_lit(p)
	} else if p.at(TokenKind::FloatLit) {
		float_lit(p)
	} else if p.at(TokenKind::Ident) {
		ident_expr(p)
	} else if p.at(TokenKind::Minus) {
		prefix_neg(p, allow_struct_expressions)
	} else if p.at(TokenKind::Ampersand) {
		prefix_addr(p, allow_struct_expressions)
	} else if p.at(TokenKind::LParen) {
		paren_or_tuple_expr(p, allow_struct_expressions)
	} else if p.at(TokenKind::IfKw) {
		if_expr(p)
	} else if p.at(TokenKind::LBrace) {
		block_expr(p)
	} else if p.at(TokenKind::Intrinsic) {
		intrinsic_expr(p)
	} else {
		p.expected(format!("expression lhs"));
		return None;
	};

	Some(cm)
}

fn int_lit(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.expect(TokenKind::IntLit, &recovery(&[]));
	m.complete(p, SyntaxKind::IntExpr)
}

fn float_lit(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.expect(TokenKind::FloatLit, &recovery(&[]));
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

fn prefix_addr(p: &mut Parser, allow_struct_expressions: bool) -> CompletedMarker {
	assert!(p.at(TokenKind::Ampersand));
	let m = p.start();
	let op = PrefixOp::Addr;
	let (_, right_binding_power) = op.binding_power();
	p.bump();
	expr_binding_power(p, right_binding_power, allow_struct_expressions);
	m.complete(p, SyntaxKind::AddressExpr)
}

fn paren_or_tuple_expr(p: &mut Parser, allow_struct_expressions: bool) -> CompletedMarker {
	assert!(p.at(TokenKind::LParen));
	let m = p.start();
	p.bump();
	expr(p, allow_struct_expressions);
	if p.at(TokenKind::Comma) {
		p.bump();

		while p.loop_safe_not_at(TokenKind::RParen) {
			expr(p, true);
			if p.at(TokenKind::Comma) {
				p.bump();
			} else if !p.at(TokenKind::RParen) {
				p.expected(format!("`)` at end of tuple expression"));
			}
		}

		p.bump();
		return m.complete(p, SyntaxKind::TupleExpr);
	}
	p.expect(TokenKind::RParen, &recovery(&[]));
	m.complete(p, SyntaxKind::ParenExpr)
}

fn if_expr(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.expect(TokenKind::IfKw, &recovery(&[]));
	expr::expr(p, false);
	expr::block_expr(p);
	if p.at(TokenKind::ElseKw) {
		p.bump();
		if p.at(TokenKind::IfKw) {
			if_expr(p);
		} else {
			expr::block_expr(p);
		}
	}
	m.complete(p, SyntaxKind::IfExpr)
}

pub(crate) fn block_expr(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.expect(TokenKind::LBrace, &recovery(&[TokenKind::RBrace]));
	while p.loop_safe_not_at(TokenKind::RBrace) {
		stmt::stmt(p);
	}
	p.expect(TokenKind::RBrace, &recovery(&[]));
	m.complete(p, SyntaxKind::BlockExpr)
}

fn intrinsic_expr(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(TokenKind::Intrinsic));
	let m = p.start();
	p.bump();
	m.complete(p, SyntaxKind::IntrinsicExpr)
}

#[cfg(test)]
mod tests {
	use crate::test_expr_str;

	// -- Integers --

	// test_expr_str!(basic_literals_b10_int, "1");
	// test_expr_str!(basic_literals_b10_int_neg, "-10");
	// test_expr_str!(basic_literals_b10_int_sep, "1_000_000");
	// test_expr_str!(basic_literals_b10_int_neg_sep, "-1_000_000");
	// test_expr_str!(basic_literals_b16_int, "0xff");
	// test_expr_str!(basic_literals_b16_int_neg, "-0xabcdef");
	// test_expr_str!(basic_literals_b16_int_sep, "0xa_b_ff");
	// test_expr_str!(basic_literals_b16_int_neg_sep, "-0xa_b_ff");
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
	// test_expr_str!(binops_cmp_lt, "-1.0_1<1");
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

	// // ------- Call ---------
	// test_expr_str!(call_basic, "foo()");
	// test_expr_str!(call_double_colon, "foo::bar::bazz()");

	// test_expr_str!(
	// 	infix_with_comments,
	// 	r#"// hello
	// 	-1 + 1"#
	// );
	// test_expr_str!(unclosed_paren_expr, "(foo");
}
