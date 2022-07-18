use super::{expr::type_expr, *};
use flux_lexer::T;

pub fn top_level_decl(p: &mut Parser) {
	if p.at(T!(pub)) {
		pub_top_level_decl(p);
	} else if p.at(T!(mod)) {
		mod_decl(p);
	} else if p.at(T!(use)) {
		use_decl(p);
	} else if p.at(T!(fn)) {
		fn_decl(p);
	} else if p.at(T!(type)) {
		type_decl(p);
	} else if p.at(T!(trait)) {
		trait_decl(p);
	} else if p.at(T!(apply)) {
		apply_decl(p);
	} else {
		if !p.at(T!(comment)) {
			p.expected(format!("top level declaration"));
		}
	}
}

fn pub_top_level_decl(p: &mut Parser) {
	if p.next_at(T!(fn)) {
		fn_decl(p);
	} else if p.next_at(T!(type)) {
		type_decl(p);
	} else {
		if !p.at(T!(comment)) {
			p.expected(format!("top level declaration"));
		}
	}
}

fn trait_decl(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(T!(trait)));
	let m = p.start();
	p.bump();
	p.expect(T!(ident));
	p.expect(T!(lbrace));
	while p.loop_safe_not_at(T!(rbrace)) {
		trait_method(p);
	}
	p.expect(T!(rbrace));
	m.complete(p, SyntaxKind::TraitDecl)
}

fn trait_method(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.expect(T!(fn));
	p.expect(T!(ident));
	p.expect(T!(lparen));
	while !p.at(T!(rparen)) && !p.at_end() {
		trait_method_param(p);
		if !p.at(T!(rparen)) {
			p.expect(T!(comma));
		}
	}
	p.expect(T!(rparen));
	if p.at(T!(->)) {
		p.bump();
		type_expr(p);
	}
	p.expect(TokenKind::SemiColon);
	m.complete(p, SyntaxKind::TraitMethod)
}

fn trait_method_param(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	if p.at(T![mut]) {
		p.bump();
	}
	type_expr(p);
	p.expect(T![ident]);
	m.complete(p, SyntaxKind::FnParam)
}

fn apply_decl(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(T!(apply)));
	let m = p.start();
	p.bump();
	if p.at(T!(to)) {
		p.bump();
	} else {
		p.expect(T!(ident));
		p.expect(T!(to));
		p.expect(T!(ident));
	}
	apply_block(p);
	m.complete(p, SyntaxKind::ApplyDecl)
}

fn apply_block(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.expect(T!(lbrace));

	while p.loop_safe_not_at(T!(rbrace)) {
		fn_decl(p);
	}

	p.expect(T!(rbrace));
	m.complete(p, SyntaxKind::ApplyBlock)
}

fn mod_decl(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(T!(mod)));
	let m = p.start();
	p.bump();
	p.expect(T!(ident));
	p.expect(T!(semicolon));
	m.complete(p, SyntaxKind::ModDecl)
}

fn use_decl(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(T!(use)));
	let m = p.start();
	p.bump();
	expr::path(p);
	p.expect(T!(semicolon));
	m.complete(p, SyntaxKind::UseDecl)
}

fn type_decl(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(T!(type)) || p.at(T!(pub)));
	let m = p.start();
	if p.at(T!(pub)) {
		p.bump()
	}
	p.expect(T!(type));
	p.expect(T!(ident));
	type_expr(p);
	m.complete(p, SyntaxKind::TypeDecl)
}

fn fn_decl(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(T!(fn)) || p.at(T!(pub)));
	let m = p.start();
	if p.at(T!(pub)) {
		p.bump()
	}
	p.expect(T!(fn));
	p.expect(T![ident]);
	if p.at(T![<]) {
		generic_list(p);
	}
	fn_params(p);
	if p.at(T!(->)) {
		p.bump();
		type_expr(p);
	}
	p.expect(T!(=>));
	expr::expr(p, true);
	m.complete(p, SyntaxKind::FnDecl)
}

pub(crate) fn generic_list(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(T![<]));
	let m = p.start();
	p.bump();
	while p.loop_safe_not_at(T!(>)) {
		p.expect(T![ident]);
		if !p.at(T!(comma)) {
			if !p.at(T!(>)) {
				p.expected(format!("`>` at end of generic list"));
			}
			break;
		} else {
			p.bump();
		}
	}
	p.expect(T!(>));
	m.complete(p, SyntaxKind::GenericList)
}

fn fn_params(p: &mut Parser) {
	p.expect(T![lparen]);
	while !p.at(T![rparen]) && !p.at_end() {
		fn_param(p);
		if !p.at(T![rparen]) {
			if !p.at(T![comma]) {
				p.expected(format!("either `,` or `)` in function parameter list"));
				break;
			} else {
				p.bump();
			}
		}
	}
	p.expect(T![rparen]);
}

fn fn_param(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	if p.at(T![mut]) {
		p.bump();
	}
	type_expr(p);
	p.expect(T![ident]);
	m.complete(p, SyntaxKind::FnParam)
}

#[cfg(test)]
mod test {
	use crate::test_decl_str;

	// test_decl_str!(fn_decl, r#"fn main(i15 argc, mut u4 test) -> f32 {}"#);
	// test_decl_str!(fn_decl_ret_void, r#"fn main(i15 argc, mut u4 test) {}"#);
	// test_decl_str!(ty_decl_prim, r#"type Foo i32"#);
	// test_decl_str!(
	// 	ty_decl_struct,
	// 	r#"type Foo struct {
	// 	i7 x;
	// 	pub u1 y;
	// 	mut f64 z;
	// 	pub mut Bar a;
	// }"#
	// );
	// test_decl_str!(
	// 	ty_decl_trait,
	// 	r#"type Foo trait {
	// 		fn foo();
	// 		pub fn bar(i32 x) -> u5;
	// 		fn bazz(mut f64 x) -> f32;
	// }"#
	// );
	// test_decl_str!(
	// 	ty_decl_trait_method_missing_ret_ty,
	// 	r#"type Foo trait {
	// 		fn foo() -> ;
	// }"#
	// );
}
