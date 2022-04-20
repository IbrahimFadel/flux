use std::ops::Range;

use pi_ast::{
	ApplyBlock, Expr, FnDecl, FnParam, GenericTypes, Ident, PrimitiveKind, PrimitiveType, TypeDecl,
};
use pi_error::PIErrorCode;
use pi_lexer::token::TokenKind;
use smol_str::SmolStr;

use crate::{
	expr::{ident, type_expr},
	stmt::block,
	tok_val, ParseInput,
};

pub fn apply_block(input: &mut ParseInput) -> ApplyBlock {
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
	let mut struct_name = Ident {
		span: input.tok().span.start..input.tok().span.end,
		val: SmolStr::from(""),
	};
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
			methods.push(fn_decl(input, pub_, pub_start..pub_end));
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
	return ApplyBlock::new(interface_name, struct_name, methods);
}

pub fn fn_decl(input: &mut ParseInput, pub_: bool, pub_span: Range<usize>) -> FnDecl {
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

	let mut generics = vec![];
	if input.tok().kind == TokenKind::CmpLT {
		generics = generic_types(input);
	}
	let params_start = input.tok().span.start;
	let params = params(input);
	let params_end = input.tok().span.start;
	let mut ret_ty = Expr::PrimitiveType(PrimitiveType::new(PrimitiveKind::Void));
	let mut ret_ty_start = input.tok().span.start;
	if input.tok().kind == TokenKind::Arrow {
		input.next();
		ret_ty_start = input.tok().span.start;
		let ty = type_expr(input);
		if ty == Expr::Error {
			if input.tok().kind != TokenKind::LBrace {
				input.next();
			}
		}
		ret_ty = ty;
	}
	let ret_ty_end = input.tok().span.start;
	let block = block(input);

	FnDecl::new(
		pub_span,
		pub_,
		Ident::new(name_begin..name_end, SmolStr::from(name)),
		generics,
		params_start..params_end,
		params,
		ret_ty_start..ret_ty_end,
		ret_ty,
		block,
	)
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

fn param(input: &mut ParseInput) -> FnParam {
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
			return FnParam::new(
				mut_start..mut_end,
				mut_,
				0..0,        // doesn't matter
				Expr::Error, // it's not actually an error, but we don't care about this value and will never read it. An Option might work better, but it seems like overkill
				Ident::new(begin..input.tok().span.end, SmolStr::from("this")),
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

	let type_begin = input.tok().span.start;
	let type_ = type_expr(input);
	let type_end = input.tok().span.end;
	let name = ident(input);

	FnParam::new(mut_start..mut_end, mut_, type_begin..type_end, type_, name)
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
