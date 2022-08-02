use crate::{recovery, TYPE_RECOVERY_SET};

use super::*;

pub(crate) fn type_expr(p: &mut Parser) -> Option<CompletedMarker> {
	let result = if p.at(TokenKind::INKw)
		|| p.at(TokenKind::UNKw)
		|| p.at(TokenKind::F32Kw)
		|| p.at(TokenKind::F64Kw)
	{
		primitive_type(p)
	} else if p.at(TokenKind::Ident) {
		ident_type(p)
	} else if p.at(TokenKind::LParen) {
		tuple_type(p)
	} else if p.at(TokenKind::StructKw) {
		struct_type(p)
	} else if p.at(TokenKind::EnumKw) {
		enum_type(p)
	} else if p.at(TokenKind::Star) {
		pointer_type(p)
	} else {
		p.expected(format!("type"));
		return None;
	};

	Some(result)
}

fn tuple_type(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(TokenKind::LParen));
	let m = p.start();
	p.bump();
	while p.loop_safe_not_at(TokenKind::RParen) {
		type_expr(p);
		if p.at(TokenKind::Comma) {
			p.bump();
		} else if !p.at(TokenKind::RParen) {
			p.expected(format!("`)` at end of tuple type"));
		}
	}
	p.expect(TokenKind::RParen, &recovery(&[TokenKind::RBrace]));
	m.complete(p, SyntaxKind::TupleType)
}

fn enum_type(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(TokenKind::EnumKw));
	let m = p.start();
	p.bump();
	p.expect(TokenKind::LBrace, &recovery(&[TokenKind::RBrace]));
	while !p.at(TokenKind::RBrace) && !p.at_end() {
		enum_type_field(p);
		if !p.at(TokenKind::RBrace) {
			p.expect(TokenKind::Comma, &recovery(&[TokenKind::RBrace]));
		}
	}
	p.expect(TokenKind::RBrace, &recovery(&[]));
	m.complete(p, SyntaxKind::EnumType)
}

fn enum_type_field(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.expect(
		TokenKind::Ident,
		&recovery(&[TokenKind::Comma, TokenKind::FatArrow]),
	);
	if p.at(TokenKind::FatArrow) {
		p.bump();
		type_expr(p);
	}
	m.complete(p, SyntaxKind::EnumTypeField)
}

fn struct_type(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(TokenKind::StructKw));
	let m = p.start();
	p.bump();
	p.expect(TokenKind::LBrace, &recovery(&[TokenKind::RBrace]));
	while !p.at(TokenKind::RBrace) && !p.at_end() {
		struct_type_field(p);
		if !p.at(TokenKind::RBrace) {
			p.expect(TokenKind::Comma, &recovery(&[TokenKind::RBrace]));
		}
	}
	p.expect(TokenKind::RBrace, &recovery(&[]));
	m.complete(p, SyntaxKind::StructType)
}

fn struct_type_field(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	if p.at(TokenKind::PubKw) {
		p.bump();
	}
	if p.at(TokenKind::MutKw) {
		p.bump();
	}
	p.expect(TokenKind::Ident, &recovery(TYPE_RECOVERY_SET));
	type_expr(p);
	m.complete(p, SyntaxKind::StructTypeField)
}

fn primitive_type(p: &mut Parser) -> CompletedMarker {
	assert!(
		p.at(TokenKind::INKw)
			|| p.at(TokenKind::UNKw)
			|| p.at(TokenKind::F32Kw)
			|| p.at(TokenKind::F64Kw)
	);
	let m = p.start();
	p.bump();
	m.complete(p, SyntaxKind::PrimitiveType)
}

fn ident_type(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.bump();
	if p.at(TokenKind::CmpLt) {
		type_params(p);
	}
	m.complete(p, SyntaxKind::IdentType)
}

fn type_params(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(TokenKind::CmpLt));
	let m = p.start();
	p.bump();
	while p.loop_safe_not_at(TokenKind::CmpGt) {
		type_expr(p);
		if !p.at(TokenKind::CmpGt) {
			p.expect(TokenKind::Comma, &recovery(&[TokenKind::CmpGt]));
		}
	}
	p.expect(TokenKind::CmpGt, &recovery(&[]));
	m.complete(p, SyntaxKind::TypeParams)
}

fn pointer_type(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(TokenKind::Star));
	let m = p.start();
	p.bump();
	type_expr(p);
	m.complete(p, SyntaxKind::PointerType)
}
