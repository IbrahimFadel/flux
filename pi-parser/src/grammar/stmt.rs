use pi_lexer::T;

use super::{expr::type_expr, *};

pub(crate) fn block(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.expect(T![lbrace], format!("expected `{{` at end of block"));
	while !p.at(T![rbrace]) && !p.at_end() {
		stmt(p);
	}
	p.expect(T![rbrace], format!("expected `}}` at end of block"));
	m.complete(p, SyntaxKind::BlockStmt)
}

pub(crate) fn stmt(p: &mut Parser) -> Option<CompletedMarker> {
	if p.at(TokenKind::INKw) {
		Some(var_decl(p))
	} else {
		expr::expr(p)
	}
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
