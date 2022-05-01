use pi_lexer::T;

use super::*;

enum InfixOp {
	Add,
	Sub,
	Mul,
	Div,
}

impl InfixOp {
	fn binding_power(&self) -> (u8, u8) {
		match self {
			Self::Add | Self::Sub => (1, 2),
			Self::Mul | Self::Div => (3, 4),
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

// const PRIMITIVE_TYPE_SET: [TokenKind; 11] = [
// 	T!(u8),
// 	T!(f64),
// 	T!(f32),
// 	T!(bool),
// ];

pub(crate) fn type_expr(p: &mut Parser) -> Option<CompletedMarker> {
	let result = if p.at(T![iN]) {
		// let m = p.start();
		primitive_type_expr(p)
	// m.complete(p, SyntaxKind::TypeExpr)
	} else {
		p.error(format!("could not parse type expression"));
		return None;
	};

	Some(result)
}

fn primitive_type_expr(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(T![iN]));
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
	let cm = if p.at(TokenKind::IntLit) {
		int_lit(p)
	} else if p.at(TokenKind::Ident) {
		ident(p)
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
	m.complete(p, SyntaxKind::IntLit)
}

fn ident(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.expect(TokenKind::Ident, format!("expected identifier"));
	m.complete(p, SyntaxKind::Ident)
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
