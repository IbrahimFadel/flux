use flux_lexer::T;

use super::*;

enum InfixOp {
	Add,
	Sub,
	Mul,
	Div,
	CmpEq,
}

impl InfixOp {
	fn binding_power(&self) -> (u8, u8) {
		match self {
			Self::CmpEq => (1, 2),
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
	} else if p.at(T!(struct)) {
		struct_type(p)
	} else if p.at(T!(interface)) {
		interface_type(p)
	} else {
		p.error(format!("could not parse type expression"));
		return None;
	};

	Some(result)
}

fn interface_type(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(T!(interface)));
	let m = p.start();
	p.bump();
	p.expect(T!(lbrace), format!("expected `{{` in interface type"));
	while !p.at(T!(rbrace)) && !p.at_end() {
		interface_type_method(p);
	}
	p.expect(
		T!(rbrace),
		format!("expected `}}` at end of interface type"),
	);
	m.complete(p, SyntaxKind::InterfaceType)
}

fn interface_type_method(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	if p.at(T!(pub)) {
		p.bump();
	}
	p.expect(T!(fn), format!("expected `fn` in interface type method"));
	p.expect(
		T!(ident),
		format!("expected identifier for interface type method name"),
	);
	p.expect(
		T!(lparen),
		format!("expected `(` at beginning of interface type method parameter list"),
	);
	while !p.at(T!(rparen)) && !p.at_end() {
		interface_type_method_param(p);
		if !p.at(T!(rparen)) {
			p.expect(
				T!(comma),
				format!("expected either `}}` or `,` in interface type method parameter list"),
			);
		}
	}
	p.expect(
		T!(rparen),
		format!("expected `)` at end of interface type method parameter list"),
	);
	if p.at(T!(->)) {
		p.bump();
		type_expr(p);
	}
	p.expect(
		TokenKind::SemiColon,
		format!("expected `;` at end of interface type method"),
	);
	m.complete(p, SyntaxKind::InterfaceMethod)
}

fn interface_type_method_param(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	if p.at(T![mut]) {
		p.bump();
	}
	type_expr(p);
	p.expect(
		T![ident],
		format!("expected identifier in interface type method parameter"),
	);
	m.complete(p, SyntaxKind::FnParam)
}

fn struct_type(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(T!(struct)));
	let m = p.start();
	p.bump();
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
	type_expr(p);
	p.expect(
		TokenKind::Ident,
		format!("expected identifier for struct type field name"),
	);
	p.expect(
		TokenKind::SemiColon,
		format!("expepcted `;` after struct type field"),
	);
	m.complete(p, SyntaxKind::StructTypeField)
}

fn primitive_type(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(T![iN]) || p.at(T![uN]) || p.at(T![f32]) || p.at(T![f64]));
	let m = p.start();
	p.bump();
	m.complete(p, SyntaxKind::PrimitiveType)
}

pub(crate) fn expr(p: &mut Parser) -> Option<CompletedMarker> {
	expr_binding_power(p, 0)
}

fn expr_binding_power(p: &mut Parser, minimum_binding_power: u8) -> Option<CompletedMarker> {
	let mut lhs = lhs(p)?;

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
		} else {
			break;
		};
		let (left_binding_power, right_binding_power) = op.binding_power();

		if left_binding_power < minimum_binding_power {
			break;
		}

		p.bump();

		let m = lhs.precede(p);
		let parsed_rhs = expr_binding_power(p, right_binding_power).is_some();
		lhs = m.complete(p, SyntaxKind::BinExpr);

		if !parsed_rhs {
			break;
		}
	}

	Some(lhs)
}

fn lhs(p: &mut Parser) -> Option<CompletedMarker> {
	let cm = if p.at(T!(intlit)) {
		int_lit(p)
	} else if p.at(T!(floatlit)) {
		float_lit(p)
	} else if p.at(T!(ident)) {
		ident_expr(p)
	} else if p.at(TokenKind::Minus) {
		prefix_neg(p)
	} else if p.at(TokenKind::LParen) {
		paren_expr(p)
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
	let m = p.start();
	p.expect(TokenKind::Ident, format!("expected identifier"));
	m.complete(p, SyntaxKind::IdentExpr)
}

fn prefix_neg(p: &mut Parser) -> CompletedMarker {
	let m = p.start();

	let op = PrefixOp::Neg;
	let (_, right_binding_power) = op.binding_power();
	p.bump();
	expr_binding_power(p, right_binding_power);

	m.complete(p, SyntaxKind::PrefixExpr)
}

fn paren_expr(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(TokenKind::LParen));
	let m = p.start();
	p.bump();
	expr_binding_power(p, 0);
	p.expect(
		TokenKind::RParen,
		format!("expected `)` in paren expression"),
	);
	m.complete(p, SyntaxKind::ParenExpr)
}

#[cfg(test)]
mod tests {
	use crate::test_stmt_str;
	test_stmt_str!(
		infix_with_comments,
		r#"// hello
		-1 + 1"#
	);
	test_stmt_str!(unclosed_paren_expr, "(foo");
}
