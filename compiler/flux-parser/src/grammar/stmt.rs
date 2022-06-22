use flux_lexer::T;

use super::{expr::type_expr, *};

pub(crate) fn stmt(p: &mut Parser) -> CompletedMarker {
	if p.at(T!(let))
		|| p.at(T!(iN))
		|| p.at(T!(uN))
		|| p.at(T!(f32))
		|| p.at(T!(f64))
		|| (p.at(T!(ident)) && matches!(p.peek_next(), Some(T!(ident)) | Some(T!(eq))))
		|| p.at(T!(lparen))
	{
		var_decl(p)
	} else if p.at(T!(return)) {
		return_stmt(p)
	} else {
		let m = p.start();
		expr::expr(p, true);
		if p.at(T!(semicolon)) {
			p.bump();
		}
		m.complete(p, SyntaxKind::ExprStmt)
	}
}

fn return_stmt(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	p.bump();
	if p.at(T!(semicolon)) {
		p.bump();
		return m.complete(p, SyntaxKind::ReturnStmt);
	}
	expr::expr(p, true);
	p.expect(
		T!(semicolon),
		format!("expected `;` after return statement"),
	);
	m.complete(p, SyntaxKind::ReturnStmt)
}

// fn else_if_stmt(p: &mut Parser) -> CompletedMarker {
// 	let m = p.start();
// 	p.expect(T!(if), format!("expected `if` in else if statement"));
// 	expr::expr(p, false);
// 	block(p);
// 	m.complete(p, SyntaxKind::IfStmt)
// }

fn var_decl(p: &mut Parser) -> CompletedMarker {
	let m = p.start();
	if p.at(T!(let)) {
		p.bump();
	} else {
		type_expr(p);
	}
	p.expect(
		TokenKind::Ident,
		format!("expected identifier in variable declaration"),
	);
	p.expect(
		TokenKind::Eq,
		format!("expected `=` in variable declaration"),
	);
	expr::expr(p, true);
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
