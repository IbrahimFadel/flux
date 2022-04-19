use pi_ast::{Expr, FnDecl, FnParam, GenericTypes, Ident, PrimitiveKind, PrimitiveType, TypeDecl};
use pi_error::PIErrorCode;
use pi_lexer::token::TokenKind;
use smol_str::SmolStr;

use crate::{
	expr::{ident, type_expr},
	stmt::block,
	tok_val, ParseInput,
};

pub fn fn_decl(input: &mut ParseInput, pub_: bool) -> FnDecl {
	input.next();
	let program_clone = input.program.clone();
	let name = tok_val(
		&program_clone,
		input.expect(
			TokenKind::Ident,
			input.error(
				"expected identifier in function declaration".to_owned(),
				PIErrorCode::ParseExpectedIdentFnDecl,
				vec![
					(
						"expected identifier following `fn` keyword".to_owned(),
						input.tok().span.clone(),
					),
					(
						"(hint) name the function".to_owned(),
						input.tok().span.clone(),
					),
				],
			),
		),
	);
	if input.tok().kind != TokenKind::LParen && input.tok().kind != TokenKind::CmpLT {
		// if someone forgets an indentifier, then we shouldn't advance so that params / generics can be parsed
		input.next();
	}

	let mut generics = vec![];
	if input.tok().kind == TokenKind::CmpLT {
		generics = generic_types(input);
	}
	let params = params(input);
	let mut ret_ty = Expr::PrimitiveType(PrimitiveType::new(PrimitiveKind::Void));
	if input.tok().kind == TokenKind::Arrow {
		ret_ty = return_type(input);
	}
	let block = block(input);

	FnDecl::new(pub_, SmolStr::from(name), generics, params, ret_ty, block)
}

fn generic_types(input: &mut ParseInput) -> GenericTypes {
	input.next();
	let mut names = vec![];
	if input.tok().kind == TokenKind::CmpGT {
		input.errs.push(input.error(
			"expected indetifier in function generic type list".to_owned(),
			PIErrorCode::ParseExpectedIdentGenericTypeList,
			vec![(
				format!(
					"expected identifier, instead got `{}`",
					tok_val(&input.program, input.tok())
				),
				input.tok().span.clone(),
			)],
		))
	}
	while input.tok().kind != TokenKind::CmpGT {
		let name = ident(input);
		names.push(name);
		if input.tok().kind != TokenKind::CmpGT {
			input.expect(
				TokenKind::Comma,
				input.error(
					"expected `,` between identifiers in generic type list".to_owned(),
					PIErrorCode::ParseExpectedCommaInGenericTypeList,
					vec![],
				),
			);
			input.next();
		}
	}
	input.expect(
		TokenKind::CmpGT,
		input.error(
			"expected `>` after identifiers in generic type list".to_owned(),
			PIErrorCode::ParseExpectedGTAfterGenericTypeList,
			vec![],
		),
	);
	input.next();

	return names;
}

pub fn params(input: &mut ParseInput) -> Vec<FnParam> {
	input.expect(
		TokenKind::LParen,
		input.error(
			"expected `(` before function parameter list".to_owned(),
			PIErrorCode::ParseExpectedLParenBeforeParamList,
			vec![(
				format!(
					"expected `(` before function parameter list, instead got `{}`",
					tok_val(&input.program, input.tok())
				),
				input.tok().span.clone(),
			)],
		),
	);
	if input.tok().kind == TokenKind::LParen {
		input.next();
	}
	let mut params = vec![];
	while input.tok().kind != TokenKind::RParen {
		let param = param(input);
		params.push(param);
		if input.tok().kind != TokenKind::RParen {
			input.expect(
				TokenKind::Comma,
				input.error(
					"expected `,` between parameters in function parameter list".to_owned(),
					PIErrorCode::ParseExpectedCommaInParamList,
					vec![],
				),
			);
			input.next();
		}
	}
	input.expect(
		TokenKind::RParen,
		input.error(
			"expected `)` after function parameter list".to_owned(),
			PIErrorCode::ParseExpectedRParenAfterParamList,
			vec![],
		),
	);
	input.next();

	return params;
}

fn param(input: &mut ParseInput) -> FnParam {
	let mut mut_ = false;
	if input.tok().kind == TokenKind::Mut {
		mut_ = true;
		input.next();
	}

	let type_ = type_expr(input);
	let name = ident(input);

	FnParam::new(mut_, type_, name)
}

fn return_type(input: &mut ParseInput) -> Expr {
	input.next();
	let ty = type_expr(input);
	if ty == Expr::Error {
		if input.tok().kind != TokenKind::LBrace {
			input.next();
		}
	}
	return ty;
}

pub fn type_decl(input: &mut ParseInput, pub_: bool) -> TypeDecl {
	input.expect(
		TokenKind::Type,
		input.error(
			"expected `type` at beginning of type declaration".to_owned(),
			PIErrorCode::ParseExpectedTypeInTypeDecl,
			vec![],
		),
	);
	if input.tok().kind == TokenKind::Type {
		input.next();
	}

	input.expect(
		TokenKind::Ident,
		input.error(
			"expected identifier in type declaration".to_owned(),
			PIErrorCode::ParseExpectedTypeInTypeDecl,
			vec![(
				"(hint) give your type a name".to_owned(),
				input.tok().span.clone(),
			)],
		),
	);
	let mut name = String::new();
	let name_begin = input.tok().span.start;
	let mut name_end = input.tok().span.end;
	if input.tok().kind == TokenKind::Ident {
		name = tok_val(&input.program, &input.tok());
		input.next();
		name_end = input.tok().span.start;
	}

	let type_ = type_expr(input);

	input.expect(
		TokenKind::Semicolon,
		input.error(
			"expected `;` after type declaration".to_owned(),
			PIErrorCode::ParseExpectedSemicolonAfterTypeDecl,
			vec![(
				"(hint) insert `;` here".to_owned(),
				input.tok().span.clone(),
			)],
		),
	);
	if input.tok().kind == TokenKind::Semicolon {
		input.next();
	}

	input.typenames.push(name.clone());
	TypeDecl::new(
		pub_,
		Ident::new(name_begin..name_end, SmolStr::from(name)),
		type_,
	)
}
