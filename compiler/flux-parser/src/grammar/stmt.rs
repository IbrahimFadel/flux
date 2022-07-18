use super::{expr::type_expr, *};

pub(crate) fn stmt(p: &mut Parser) -> CompletedMarker {
	if p.at(TokenKind::LetKw)
		|| p.at(TokenKind::INKw)
		|| p.at(TokenKind::UNKw)
		|| p.at(TokenKind::F32Kw)
		|| p.at(TokenKind::F64Kw)
		|| (p.at(TokenKind::Ident)
			&& matches!(p.peek_next(), Some(TokenKind::Ident) | Some(TokenKind::Eq)))
	{
		var_decl(p)
	} else if p.at(TokenKind::ReturnKw) {
		return_stmt(p)
	} else {
		let m = p.start();
		expr::expr(p, true);
		if p.at(TokenKind::SemiColon) {
			p.bump();
		}
		m.complete(p, SyntaxKind::ExprStmt)
	}
}

fn return_stmt(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.bump();
	if p.at(TokenKind::SemiColon) {
		p.bump();
		return m.complete(p, SyntaxKind::ReturnStmt);
	}
	expr::expr(p, true);
	p.expect(TokenKind::SemiColon);
	m.complete(p, SyntaxKind::ReturnStmt)
}

fn var_decl(p: &mut Parser) -> CompletedMarker {
	assert!(p.at(TokenKind::LetKw));
	let m = p.start();
	p.bump();
	p.expect(TokenKind::Ident);
	if !p.at(TokenKind::Eq) {
		type_expr(p);
	}
	p.expect(TokenKind::Eq);
	expr::expr(p, true);
	p.expect(TokenKind::SemiColon);
	m.complete(p, SyntaxKind::VarDecl)
}

#[cfg(test)]
mod tests {
	use crate::test_stmt_str;
	// test_stmt_str!(var_decl, "i32 x = 1;");
	// test_stmt_str!(
	// 	var_decl_fcont,
	// 	r#"i32 y =
	// i32 x = 1;"#
	// );
}
