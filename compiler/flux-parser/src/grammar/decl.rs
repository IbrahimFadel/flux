use super::{expr::type_expr, *};
use flux_lexer::T;

pub(crate) fn top_level_decl(p: &mut Parser) {
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
			p.error(format!("expected top level declaration"));
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
			p.error(format!("expected top level declaration"));
		}
	}
}

fn trait_decl(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(T!(trait)));
	let m = p.start();
	p.bump();
	p.expect(T!(ident), format!("expected identifier after `trait`"));
	p.expect(
		T!(lbrace),
		format!("expected `{{` before trait method declarations"),
	);
	while p.loop_safe_not_at(T!(rbrace)) {
		trait_method(p);
	}
	p.expect(
		T!(rbrace),
		format!("expected `}}` after trait method declarations"),
	);
	m.complete(p, SyntaxKind::TraitDecl)
}

fn trait_method(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.expect(T!(fn), format!("expected `fn` in trait type method"));
	p.expect(
		T!(ident),
		format!("expected identifier for trait type method name"),
	);
	p.expect(
		T!(lparen),
		format!("expected `(` at beginning of trait type method parameter list"),
	);
	while !p.at(T!(rparen)) && !p.at_end() {
		trait_method_param(p);
		if !p.at(T!(rparen)) {
			p.expect(
				T!(comma),
				format!("expected either `}}` or `,` in trait type method parameter list"),
			);
		}
	}
	p.expect(
		T!(rparen),
		format!("expected `)` at end of trait type method parameter list"),
	);
	if p.at(T!(->)) {
		p.bump();
		type_expr(p);
	}
	p.expect(
		TokenKind::SemiColon,
		format!("expected `;` at end of trait type method"),
	);
	m.complete(p, SyntaxKind::TraitMethod)
}

fn trait_method_param(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	if p.at(T![mut]) {
		p.bump();
	}
	type_expr(p);
	p.expect(
		T![ident],
		format!("expected identifier in trait type method parameter"),
	);
	m.complete(p, SyntaxKind::FnParam)
}

fn apply_decl(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(T!(apply)));
	let m = p.start();
	p.bump();
	if p.at(T!(to)) {
		p.bump();
		p.expect(T!(ident), format!("expected identifier after `to`"));
	} else {
		p.expect(
			T!(ident),
			format!("expected identifier or `to` after `apply`"),
		);
		p.expect(
			T!(to),
			format!("expected `to` after the trait being applied"),
		);
		p.expect(T!(ident), format!("expected identifier after `to`"));
	}
	apply_block(p);
	m.complete(p, SyntaxKind::ApplyDecl)
}

fn apply_block(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.expect(T!(lbrace), format!("expected `{{` at start of apply block"));

	while p.loop_safe_not_at(T!(rbrace)) {
		fn_decl(p);
	}

	p.expect(T!(rbrace), format!("expected `}}` at end of apply block"));
	m.complete(p, SyntaxKind::ApplyBlock)
}

fn mod_decl(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(T!(mod)));
	let m = p.start();
	p.bump();
	p.expect(T!(ident), format!("expected identifier in mod declaration"));
	p.expect(T!(semicolon), format!("expected `;` after mod declaration"));
	m.complete(p, SyntaxKind::ModDecl)
}

fn use_decl(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(T!(use)));
	let m = p.start();
	p.bump();
	expr::path(p);
	p.expect(T!(semicolon), format!("expected `;` after `use` path"));
	m.complete(p, SyntaxKind::UseDecl)
}

fn type_decl(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(T!(type)) || p.at(T!(pub)));
	let m = p.start();
	if p.at(T!(pub)) {
		p.bump()
	}
	p.expect(T!(type), format!("expected `type` in type declaration"));
	p.expect(
		T!(ident),
		format!("expected identifier in type declaration"),
	);
	type_expr(p);
	m.complete(p, SyntaxKind::TypeDecl)
}

fn fn_decl(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(T!(fn)) || p.at(T!(pub)));
	let m = p.start();
	if p.at(T!(pub)) {
		p.bump()
	}
	p.expect(T!(fn), format!("expected `fn` in function declaration"));
	p.expect(
		T![ident],
		format!("expected identifier in function declaration"),
	);
	if p.at(T![<]) {
		generic_list(p);
	}
	fn_params(p);
	if p.at(T!(->)) {
		p.bump();
		type_expr(p);
	}
	p.expect(T!(=>), format!("expected `=>` before function body"));
	expr::expr(p, true);
	m.complete(p, SyntaxKind::FnDecl)
}

pub(crate) fn generic_list(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(T![<]));
	let m = p.start();
	p.bump();
	while p.loop_safe_not_at(T!(>)) {
		p.expect(T![ident], format!("expected identifier in generic list"));
		if !p.at(T!(comma)) {
			if !p.at(T!(>)) {
				p.error(format!("expected `>` at end of generic list"));
			}
			break;
		} else {
			p.bump();
		}
	}
	p.expect(T!(>), format!("expected `>` at end of generic list"));
	m.complete(p, SyntaxKind::GenericList)
}

fn fn_params(p: &mut Parser) {
	p.expect(
		T![lparen],
		format!("expected `(` at beginning of function parameter list"),
	);
	while !p.at(T![rparen]) && !p.at_end() {
		fn_param(p);
		if !p.at(T![rparen]) {
			if !p.at(T![comma]) {
				p.error(format!(
					"expected either `,` or `)` in function parameter list"
				));
				break;
			} else {
				p.bump();
			}
		}
	}
	p.expect(
		T![rparen],
		format!("expected `)` at end of function parameter list"),
	);
}

fn fn_param(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	if p.at(T![mut]) {
		p.bump();
	}
	type_expr(p);
	p.expect(
		T![ident],
		format!("expected identifier in function parameter"),
	);
	m.complete(p, SyntaxKind::FnParam)
}

#[cfg(test)]
mod test {
	use crate::test_decl_str;

	test_decl_str!(fn_decl, r#"fn main(i15 argc, mut u4 test) -> f32 {}"#);
	test_decl_str!(fn_decl_ret_void, r#"fn main(i15 argc, mut u4 test) {}"#);
	test_decl_str!(ty_decl_prim, r#"type Foo i32"#);
	test_decl_str!(
		ty_decl_struct,
		r#"type Foo struct {
		i7 x;
		pub u1 y;
		mut f64 z;
		pub mut Bar a;
	}"#
	);
	test_decl_str!(
		ty_decl_trait,
		r#"type Foo trait {
			fn foo();
			pub fn bar(i32 x) -> u5;
			fn bazz(mut f64 x) -> f32;
	}"#
	);
	test_decl_str!(
		ty_decl_trait_method_missing_ret_ty,
		r#"type Foo trait {
			fn foo() -> ;
	}"#
	);
}
