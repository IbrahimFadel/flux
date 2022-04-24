use pi_ast::{
	ApplyBlock, Expr, FnDecl, FnParam, GenericTypes, Ident, PrimitiveKind, PrimitiveType, Spanned,
	TypeDecl,
};
use pi_error::PIErrorCode;
use pi_lexer::token::TokenKind;

use crate::{
	expr::{ident, type_expr},
	stmt::block,
	tok_val, ParseInput,
};

pub fn apply_block(input: &mut ParseInput) -> Spanned<ApplyBlock> {
	let start = input.tok().span.start;
	input.next();

	input.expect(
		TokenKind::Ident,
		input.error(
			"expected identifier following `apply`".to_owned(),
			PIErrorCode::ParseExpectedIdentAfterApply,
			vec![],
		),
	);
	let mut interface_name = None;
	let mut struct_name = Spanned::new(
		Ident::from(""),
		input.tok().span.start..input.tok().span.end,
	);
	if input.tok().kind == TokenKind::Ident {
		struct_name = ident(input);
	}

	if input.tok().kind == TokenKind::To {
		input.next();
		if input.tok().kind == TokenKind::Ident {
			interface_name = Some(struct_name);
			struct_name = ident(input);
		} else {
			input.errs.push(input.error(
				"expected ident following `to`".to_owned(),
				PIErrorCode::ParseExpectedIdentAfterTo,
				vec![],
			));
		}
	}

	input.expect(
		TokenKind::LBrace,
		input.error(
			"expected `{` at start of `apply` block".to_owned(),
			PIErrorCode::ParseExpectedLBraceInApplyBlock,
			vec![],
		),
	);
	if input.tok().kind == TokenKind::LBrace {
		input.next();
	}

	let mut methods = vec![];
	input.inside_apply_or_interface = true;

	if input.tok().kind != TokenKind::Pub
		&& input.tok().kind != TokenKind::Fn
		&& input.tok().kind != TokenKind::RBrace
	{
		input.errs.push(input.error(
			"expected either function declaration or `}` in apply block".to_owned(),
			PIErrorCode::ParseExpectedFnOrRBraceInApplyBlock,
			vec![],
		));
	} else {
		while input.tok().kind != TokenKind::RBrace {
			let mut pub_ = false;
			let pub_start = input.tok().span.start;
			let mut pub_end = input.tok().span.start;
			if input.tok().kind == TokenKind::Pub {
				input.next();
				pub_end = input.tok().span.start;
				pub_ = true;
			}
			methods.push(fn_decl(input, Spanned::new(pub_, pub_start..pub_end)));
		}

		input.expect(
			TokenKind::RBrace,
			input.error(
				"expected `}` after `apply` block".to_owned(),
				PIErrorCode::ParseExpectedRBraceAfterApplyBlock,
				vec![],
			),
		);
		if input.tok().kind == TokenKind::RBrace {
			input.next();
		}
	}

	input.inside_apply_or_interface = false;
	let end = input.tok().span.end;

	return Spanned::new(
		ApplyBlock::new(interface_name, struct_name, methods),
		start..end,
	);
}

pub fn fn_decl(input: &mut ParseInput, pub_: Spanned<bool>) -> Spanned<FnDecl> {
	let start = input.tok().span.start;
	input.next();
	let program_clone = input.program.clone();
	let name_begin = input.tok().span.start;
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
	let name_end = input.tok().span.start;

	let mut generics: Spanned<GenericTypes> =
		Spanned::new(vec![], input.tok().span.start..input.tok().span.start);
	if input.tok().kind == TokenKind::CmpLT {
		generics = generic_types(input);
	}
	let params_start = input.tok().span.start;
	let params = params(input);
	let params_end = input.tok().span.start;
	let mut ret_ty = Spanned::new(
		Expr::PrimitiveType(PrimitiveType::new(PrimitiveKind::Void)),
		input.tok().span.start..input.tok().span.start,
	);
	if input.tok().kind == TokenKind::Arrow {
		input.next();
		let ty = type_expr(input);
		if ty.node == Expr::Error {
			if input.tok().kind != TokenKind::LBrace {
				input.next();
			}
		}
		ret_ty = ty;
	}
	let block = block(input);

	let end = input.tok().span.start;
	Spanned::new(
		FnDecl::new(
			pub_,
			Spanned::new(Ident::from(name), name_begin..name_end),
			generics,
			Spanned::new(params, params_start..params_end),
			ret_ty,
			block,
		),
		start..end,
	)
}

fn generic_types(input: &mut ParseInput) -> Spanned<GenericTypes> {
	let start = input.tok().span.start;
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
	let end = input.tok().span.start;
	return Spanned::new(names, start..end);
}

pub fn params(input: &mut ParseInput) -> Vec<Spanned<FnParam>> {
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
	while input.tok().kind != TokenKind::RParen && input.tok().kind != TokenKind::EOF {
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

fn param(input: &mut ParseInput) -> Spanned<FnParam> {
	let mut mut_ = false;
	let mut_start = input.tok().span.start;
	let mut mut_end = input.tok().span.start;
	if input.tok().kind == TokenKind::Mut {
		mut_ = true;
		input.next();
		mut_end = input.tok().span.start;
	}

	if input.tok().kind == TokenKind::Ident && tok_val(&input.program, input.tok()) == "this" {
		if input.inside_apply_or_interface {
			let begin = input.tok().span.start;
			input.next();
			return Spanned::new(
				FnParam::new(
					Spanned::new(mut_, mut_start..mut_end),
					Spanned::new(Expr::Error, 0..0), // it's not actually an error, but we don't care about this value and will never read it. An Option might work better, but it seems like overkill
					Spanned::new(Ident::from("this"), begin..input.tok().span.end),
				),
				mut_start..input.tok().span.end,
			);
		} else {
			input.errs.push(input.error(
				"unexpected keyword `this` outside of allow block".to_owned(),
				PIErrorCode::ParseUnexpectedThisOutsideApply,
				vec![],
			));
			input.next();
		}
	}

	let type_ = type_expr(input);
	let name = ident(input);
	let end = input.tok().span.start;

	Spanned::new(
		FnParam::new(Spanned::new(mut_, mut_start..mut_end), type_, name),
		mut_start..end,
	)
}

pub fn type_decl(input: &mut ParseInput, pub_: Spanned<bool>) -> Spanned<TypeDecl> {
	let start = input.tok().span.start;
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
	let end = input.tok().span.start;
	Spanned::new(
		TypeDecl::new(
			pub_,
			Spanned::new(Ident::from(name), name_begin..name_end),
			type_,
		),
		start..end,
	)
}
