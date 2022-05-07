use pi_lexer::T;

use super::{expr::type_expr, *};

pub(crate) fn block(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.expect(T![lbrace], format!("expected `{{` at start of block"));
	while !p.at(T![rbrace]) && !p.at_end() {
		stmt(p);
	}
	p.expect(T![rbrace], format!("expected `}}` at end of block"));
	m.complete(p, SyntaxKind::BlockStmt)
}

pub(crate) fn stmt(p: &mut Parser) -> Option<CompletedMarker> {
	if p.at(T!(iN)) {
		Some(var_decl(p))
	} else if p.at(T!(if)) {
		Some(if_stmt(p))
	} else {
		expr::expr(p)
	}
}

fn if_stmt(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.expect(T!(if), format!("expected `if` in if statement"));
	expr::expr(p);
	block(p);
	while p.at(T!(else)) {
		p.bump();
		if p.at(T!(lbrace)) {
			block(p);
		} else {
			else_if_stmt(p);
		}
	}
	m.complete(p, SyntaxKind::IfStmt)
}

fn else_if_stmt(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.expect(T!(if), format!("expected `if` in else if statement"));
	expr::expr(p);
	block(p);
	m.complete(p, SyntaxKind::IfStmt)
}

fn var_decl(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	type_expr(p);
	p.expect(
		TokenKind::Ident,
		format!("expected identifier in variable declaration"),
	);
	p.expect(
		TokenKind::Eq,
		format!("expected `=` in variable declaration"),
	);
	expr::expr(p);
	p.expect(
		TokenKind::SemiColon,
		format!("expected `;` after variable declaration"),
	);
	m.complete(p, SyntaxKind::VarDecl)
}

#[cfg(test)]
mod tests {
	use crate::test_stmt_str;
	test_stmt_str!(var_decl, "i32 x = 1;");
	test_stmt_str!(
		var_decl_fcont,
		r#"i32 y =
	i32 x = 1;"#
	);
}
