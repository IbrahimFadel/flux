use super::{expr::type_expr, *};
use crate::grammar::stmt::block;
use flux_lexer::T;

pub(crate) fn top_level_decl(p: &mut Parser) {
	if p.at(T!(pub)) {
		pub_tob_level_decl(p);
	} else if p.at(T!(fn)) {
		fn_decl(p);
	} else if p.at(T!(type)) {
		type_decl(p);
	} else {
		p.error(format!("expected top level declaration"));
	}
}

fn pub_tob_level_decl(p: &mut Parser) {
	if p.next_at(T!(fn)) {
		fn_decl(p);
	} else if p.next_at(T!(type)) {
		type_decl(p);
	} else {
		p.error(format!("expected top level declaration"));
	}
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
	if p.at(T![cmplt]) {
		generic_list(p);
	}
	fn_params(p);
	if p.at(T!(->)) {
		p.bump();
		type_expr(p);
	}
	block(p);
	m.complete(p, SyntaxKind::FnDecl)
}

fn generic_list(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(T![cmplt]));
	let m = p.start();
	p.expect(
		T![ident],
		format!("expected identifier in function generic list"),
	);
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

	test_decl_str!(fn_decl, r#"fn main(i32 argc, mut i32 test) -> i32 {}"#);
}
