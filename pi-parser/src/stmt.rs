use pi_ast::{BlockStmt, Expr, If, Mod, Return, Spanned, Stmt, VarDecl};
use pi_error::PIErrorCode;
use pi_lexer::token::TokenKind;

use crate::{
	expr::{expr, ident, type_expr},
	tok_val, ParseInput,
};

pub fn mod_stmt(input: &mut ParseInput, pub_: Spanned<bool>) -> Spanned<Mod> {
	let start = input.tok().span.start;
	input.next();
	let name = ident(input);
	input.expect(
		TokenKind::Semicolon,
		input.error(
			"expected `;` after mod statement".to_owned(),
			PIErrorCode::ParseExpectedSemicolonAfterModStmt,
			vec![],
		),
	);
	if input.tok().kind == TokenKind::Semicolon {
		input.next();
	}
	let end = input.tok().span.start;
	Spanned::new(Mod::new(pub_, name), start..end)
}

pub fn block(input: &mut ParseInput) -> BlockStmt {
	input.expect(
		TokenKind::LBrace,
		input.error(
			"expected `{` at beginning of block".to_owned(),
			PIErrorCode::ParseExpectedLBraceInBlock,
			vec![(
				format!(
					"expected `{{` at beginning of block, instead got {}",
					tok_val(&input.program, input.tok())
				),
				input.tok().span.clone(),
			)],
		),
	);
	if input.tok().kind == TokenKind::LBrace {
		input.next();
	}

	let mut stmts = vec![];
	while input.tok().kind != TokenKind::RBrace && input.tok().kind != TokenKind::EOF {
		if input.tok().kind == TokenKind::BlockComment || input.tok().kind == TokenKind::LineComment {
			input.next();
			continue;
		}
		let stmt = stmt(input);
		if stmt.node == Stmt::Error {
			break;
		}
		stmts.push(stmt);
	}

	input.expect(
		TokenKind::RBrace,
		input.error(
			"expected `}` at end of block".to_owned(),
			PIErrorCode::ParseExpectedRBraceInBlock,
			vec![(
				format!(
					"expected `}}` at end of block, instead got {}",
					tok_val(&input.program, input.tok())
				),
				input.tok().span.clone(),
			)],
		),
	);
	if input.tok().kind == TokenKind::RBrace {
		input.next();
	}

	return stmts;
}

fn stmt(input: &mut ParseInput) -> Spanned<Stmt> {
	if input.tok().kind > TokenKind::TypesBegin && input.tok().kind < TokenKind::TypesEnd {
		return var_decl_stmt(input);
	} else if input
		.typenames
		.contains(&tok_val(&input.program, input.tok()))
	{
		return var_decl_stmt(input);
	} else if input.tok().kind == TokenKind::Return {
		return return_stmt(input);
	} else if input.tok().kind == TokenKind::If {
		return if_stmt(input);
	} else {
		let expr = expr(input);
		input.expect(
			TokenKind::Semicolon,
			input.error(
				"expected `;` after expression".to_owned(),
				PIErrorCode::ParseExpectedSemicolonAfterExpr,
				vec![],
			),
		);
		if input.tok().kind == TokenKind::Semicolon {
			input.next();
		}
		if expr == Spanned::new(Expr::Error, 0..0) {
			return Spanned::new(Stmt::Error, 0..0);
		}
		return Spanned::new(Stmt::ExprStmt(expr.node), expr.span.start..expr.span.end);
	}
}

fn if_stmt(input: &mut ParseInput) -> Spanned<Stmt> {
	let start = input.tok().span.start;
	input.next();
	let condition = expr(input);
	let then = block(input);
	let mut else_ = None;
	if input.tok().kind == TokenKind::Else {
		input.next();
		if input.tok().kind == TokenKind::If {
			let if_ = if_stmt(input);
			else_ = Some(vec![if_]);
		} else {
			else_ = Some(block(input));
		}
	}
	let end = input.tok().span.start;
	Spanned::new(
		Stmt::If(If::new(Box::from(condition), then, else_)),
		start..end,
	)
}

fn return_stmt(input: &mut ParseInput) -> Spanned<Stmt> {
	let start = input.tok().span.start;
	input.next();
	if input.tok().kind == TokenKind::Semicolon {
		input.next();
		return Spanned::new(
			Stmt::Return(Return::new(None)),
			start..input.tok().span.start,
		);
	}
	let expr = expr(input);
	input.expect(
		TokenKind::Semicolon,
		input.error(
			"expected `;` after return statement".to_owned(),
			PIErrorCode::ParseExpectedSemicolonAfterReturnStmt,
			vec![],
		),
	);
	if input.tok().kind == TokenKind::Semicolon {
		input.next();
	}
	let end = input.tok().span.start;
	Spanned::new(Stmt::Return(Return::new(Some(expr))), start..end)
}

fn var_decl_stmt(input: &mut ParseInput) -> Spanned<Stmt> {
	let start = input.tok().span.start;
	let ty = type_expr(input);

	let mut_start = 0;
	let mut_end = 0;

	if input.tok().kind != TokenKind::Ident {
		input.errs.push(input.error(
			"expected identifier in variable declaration".to_owned(),
			PIErrorCode::ParseExpectedIdentVarDecl,
			vec![(
				format!(
					"expected identifier in variable declaration, instead got `{}`",
					tok_val(&input.program, input.tok())
				),
				input.tok().span.clone(),
			)],
		));
	}

	let names_start = input.tok().span.start;
	let mut names_end = input.tok().span.end;
	let mut names = vec![];
	while input.tok().kind != TokenKind::Eq && input.tok().kind != TokenKind::Semicolon {
		names_end = input.tok().span.clone().end;
		let name = ident(input);
		names.push(name);
		if input.tok().kind != TokenKind::Eq && input.tok().kind != TokenKind::Semicolon {
			input.expect(
				TokenKind::Comma,
				input.error(
					"expected `,` in variable declaration identifier list".to_owned(),
					PIErrorCode::ParseExpectedCommaInVarDeclIdentList,
					vec![(
						format!(
							"expected `,` in variable declaration identifier list, instead got `{}`",
							tok_val(&input.program, input.tok())
						),
						input.tok().span.clone(),
					)],
				),
			);
			if input.tok().kind == TokenKind::Comma {
				input.next();
			}
		}
	}

	input.expect(
		TokenKind::Eq,
		input.error(
			"expected `=` after variable declaration identifier list".to_owned(),
			PIErrorCode::ParseExpectedEqVarDeclIdentList,
			vec![(
				format!(
					"expected `=` after variable declaration identifier list, instead got `{}`",
					tok_val(&input.program, input.tok())
				),
				input.tok().span.clone(),
			)],
		),
	);
	input.next();

	let values_start = input.tok().span.start;
	let mut values_end = input.tok().span.end;
	let mut values = vec![];
	while input.tok().kind != TokenKind::Semicolon {
		values_end = input.tok().span.end;
		let val = expr(input);
		values.push(val);
		if input.tok().kind == TokenKind::Comma {
			input.next();
			if input.tok().kind == TokenKind::Semicolon {
				input.errs.push(input.error(
					"expected expression after comma in variable declaration value list".to_owned(),
					PIErrorCode::ParseExpectedExprAfterCommaVarDeclValueList,
					vec![(
						format!(
								"expected expression after comma in variable declaration value list, instead got `{}`",
								tok_val(&input.program, input.tok())
							),
						input.tok().span.clone(),
					)],
				));
				break;
			}
		} else if input.tok().kind != TokenKind::Semicolon {
			input.errs.push(input.error(
				"expected `;` after variable declaration".to_owned(),
				PIErrorCode::ParseExpectedSemicolonAfterVarDecl,
				vec![(
					format!(
						"expected `;` after variable declaration, instead got `{}`",
						tok_val(&input.program, input.tok())
					),
					input.tok().span.clone(),
				)],
			));
			break;
		}
	}
	if input.tok().kind == TokenKind::Semicolon {
		input.next();
	}

	if names.len() > 1 && values.len() > 1 && values.len() != names.len() {
		if names.len() > values.len() {
			input.errs.push(input.error("more variables than values in variable declaration".to_owned(), PIErrorCode::ParseMoreIdentsThanValsVarDecl, vec![
				(format!("more variables than values in variable declaration: {} values assigned to {} variables", values.len(), names.len()), names_start..input.tok().span.end),
				(format!("{} variables declared", names.len()), names_start..names_end),
				(format!("{} values assigned", values.len()), values_start..values_end),
				("(hint) you can assign one value to multiple variables".to_owned(), names_start..input.tok().span.end),
			]));
		} else {
			input.errs.push(input.error("more values than variables in variable declaration".to_owned(), PIErrorCode::ParseMoreValsThanIdentsVarDecl, vec![
				(format!("more values than variables in variable declaration: {} values assigned to {} variables", values.len(), names.len()), names_start..input.tok().span.end),
				(format!("{} variables declared", names.len()), names_start..names_end),
				(format!("{} values assigned", values.len()), values_start..values_end),
			]));
		}
	}

	let end = input.tok().span.start;
	return Spanned::new(
		Stmt::VarDecl(VarDecl::new(
			Spanned::new(false, mut_start..mut_end),
			ty,
			names,
			values,
		)),
		start..end,
	);
}
