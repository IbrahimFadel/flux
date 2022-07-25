use crate::{recovery, EXPR_RECOVERY_SET, TYPE_RECOVERY_SET};

use super::{r#type::type_expr, *};

pub fn top_level_decl(p: &mut Parser) {
	if p.at(TokenKind::PubKw) {
		pub_top_level_decl(p);
	} else if p.at(TokenKind::ModKw) {
		mod_decl(p);
	} else if p.at(TokenKind::UseKw) {
		use_decl(p);
	} else if p.at(TokenKind::FnKw) {
		fn_decl(p);
	} else if p.at(TokenKind::TypeKw) {
		type_decl(p);
	} else if p.at(TokenKind::TraitKw) {
		trait_decl(p);
	} else if p.at(TokenKind::ApplyKw) {
		apply_decl(p);
	} else {
		if !p.at(TokenKind::Comment) {
			p.expected(format!("top level declaration"));
		}
	}
}

fn pub_top_level_decl(p: &mut Parser) {
	if p.next_at(TokenKind::FnKw) {
		fn_decl(p);
	} else if p.next_at(TokenKind::TypeKw) {
		type_decl(p);
	} else {
		if !p.at(TokenKind::Comment) {
			p.expected(format!("top level declaration"));
		}
	}
}

fn trait_decl(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(TokenKind::TraitKw));
	let m = p.start();
	p.bump();
	p.expect(TokenKind::Ident, &recovery(&[TokenKind::LBrace]));
	p.expect(TokenKind::LBrace, &recovery(&[TokenKind::RBrace]));
	while p.loop_safe_not_at(TokenKind::RBrace) {
		trait_method(p);
	}
	p.expect(TokenKind::RBrace, &recovery(&[]));
	m.complete(p, SyntaxKind::TraitDecl)
}

fn trait_method(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.expect(TokenKind::FnKw, &recovery(&[TokenKind::Ident]));
	p.expect(TokenKind::Ident, &recovery(&[TokenKind::LParen]));
	p.expect(TokenKind::LParen, &recovery(&[TokenKind::RBrace]));
	while !p.at(TokenKind::RParen) && !p.at_end() {
		trait_method_param(p);
		if !p.at(TokenKind::RParen) {
			p.expect(TokenKind::Comma, &recovery(&[TokenKind::RBrace]));
		}
	}
	p.expect(
		TokenKind::RParen,
		&recovery(&[&[TokenKind::Arrow], TYPE_RECOVERY_SET].concat()),
	);
	if p.at(TokenKind::Arrow) {
		p.bump();
		type_expr(p);
	}
	p.expect(TokenKind::SemiColon, &recovery(&[]));
	m.complete(p, SyntaxKind::TraitMethod)
}

fn trait_method_param(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	if p.at(TokenKind::MutKw) {
		p.bump();
	}
	type_expr(p);
	p.expect(TokenKind::Ident, &recovery(&[]));
	m.complete(p, SyntaxKind::FnParam)
}

fn apply_decl(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(TokenKind::ApplyKw));
	let m = p.start();
	p.bump();
	let has_generics = if p.at(TokenKind::CmpLt) {
		generic_list(p);
		true
	} else {
		false
	};
	if p.at(TokenKind::ToKw) {
		p.bump();
		type_expr(p);
	} else {
		p.expect(TokenKind::Ident, &recovery(&[TokenKind::ToKw]));
		p.expect(TokenKind::ToKw, &recovery(&[TokenKind::Ident]));
		type_expr(p);
	}
	if has_generics && p.at(TokenKind::WhereKw) {
		where_clause(p);
	}
	apply_block(p);
	m.complete(p, SyntaxKind::ApplyDecl)
}

fn apply_block(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.expect(TokenKind::LBrace, &recovery(&[TokenKind::RBrace]));

	while p.loop_safe_not_at(TokenKind::RBrace) {
		fn_decl(p);
	}

	p.expect(TokenKind::RBrace, &recovery(&[]));
	m.complete(p, SyntaxKind::ApplyBlock)
}

fn mod_decl(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(TokenKind::ModKw));
	let m = p.start();
	p.bump();
	p.expect(TokenKind::Ident, &recovery(&[TokenKind::SemiColon]));
	p.expect(TokenKind::SemiColon, &recovery(&[]));
	m.complete(p, SyntaxKind::ModDecl)
}

fn use_decl(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(TokenKind::UseKw));
	let m = p.start();
	p.bump();
	expr::path(p);
	p.expect(TokenKind::SemiColon, &recovery(&[]));
	m.complete(p, SyntaxKind::UseDecl)
}

fn type_decl(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(TokenKind::TypeKw) || p.at(TokenKind::PubKw));
	let m = p.start();
	if p.at(TokenKind::PubKw) {
		p.bump()
	}
	p.expect(TokenKind::TypeKw, &recovery(&[TokenKind::Ident]));
	p.expect(TokenKind::Ident, &recovery(TYPE_RECOVERY_SET));
	type_expr(p);
	m.complete(p, SyntaxKind::TypeDecl)
}

fn fn_decl(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(TokenKind::FnKw) || p.at(TokenKind::PubKw));
	let m = p.start();
	if p.at(TokenKind::PubKw) {
		p.bump()
	}
	p.expect(TokenKind::FnKw, &recovery(&[TokenKind::Ident]));
	p.expect(
		TokenKind::Ident,
		&recovery(&[TokenKind::CmpLt, TokenKind::LParen]),
	);
	if p.at(TokenKind::CmpLt) {
		generic_list(p);
	}
	fn_params(p);
	if p.at(TokenKind::Arrow) {
		p.bump();
		type_expr(p);
	}
	p.expect(TokenKind::FatArrow, &recovery(EXPR_RECOVERY_SET));
	expr::expr(p, true);
	m.complete(p, SyntaxKind::FnDecl)
}

pub(crate) fn generic_list(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(TokenKind::CmpLt));
	let m = p.start();
	p.bump();
	while p.loop_safe_not_at(TokenKind::CmpGt) {
		p.expect(TokenKind::Ident, &recovery(&[TokenKind::Comma]));
		if !p.at(TokenKind::Comma) {
			if !p.at(TokenKind::CmpGt) {
				p.expected(format!("`>` at end of generic list"));
			}
			break;
		} else {
			p.bump();
		}
	}
	p.expect(TokenKind::CmpGt, &recovery(&[]));
	m.complete(p, SyntaxKind::GenericList)
}

fn fn_params(p: &mut Parser) {
	p.expect(TokenKind::LParen, &recovery(&[TokenKind::RParen]));
	while !p.at(TokenKind::RParen) && !p.at_end() {
		fn_param(p);
		if !p.at(TokenKind::RParen) {
			if !p.at(TokenKind::Comma) {
				p.expected(format!("either `,` or `)` in function parameter list"));
				break;
			} else {
				p.bump();
			}
		}
	}
	p.expect(TokenKind::RParen, &recovery(&[]));
}

fn fn_param(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	if p.at(TokenKind::MutKw) {
		p.bump();
	}
	type_expr(p);
	p.expect(TokenKind::Ident, &recovery(&[]));
	m.complete(p, SyntaxKind::FnParam)
}

pub(crate) fn where_clause(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(TokenKind::WhereKw));
	let m = p.start();
	p.bump();

	// TODO: this can be infinite
	while p.loop_safe_not_at(TokenKind::LBrace) {
		type_restriction(p);
		if !p.at(TokenKind::LBrace) {
			p.expect(TokenKind::Comma, &recovery(&[]))
		}
	}

	m.complete(p, SyntaxKind::WhereClause)
}

fn type_restriction(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.expect(
		TokenKind::Ident,
		&recovery(&[TokenKind::IsKw, TokenKind::Ident]),
	);
	p.expect(TokenKind::IsKw, &recovery(&[TokenKind::Ident]));
	p.expect(
		TokenKind::Ident,
		&recovery(&[TokenKind::Comma, TokenKind::LBrace]),
	);
	m.complete(p, SyntaxKind::TypeRestriction)
}

#[cfg(test)]
mod test {
	use crate::test_decl_str;

	test_decl_str!(fn_decl, r#"fn main(i15 argc, mut u4 test) -> f32 => {}"#);
	test_decl_str!(fn_decl_ret_void, r#"fn main(i15 argc, mut u4 test) => {}"#);
	test_decl_str!(ty_decl_prim, r#"type Foo i32"#);
	test_decl_str!(
		ty_decl_struct,
		r#"type Foo struct {
		x i7,
		y u1,
		z f64,
		a Bar
	}"#
	);
	// test_decl_str!(
	// 	ty_decl_trait,
	// 	r#"trait Foo {
	// 		fn foo();
	// 		pub fn bar(i32 x) -> u5;
	// 		fn bazz(mut f64 x) -> f32;
	// }"#
	// );
}
