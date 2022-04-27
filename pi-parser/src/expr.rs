use std::{collections::HashMap, vec};

use indexmap::IndexMap;
use pi_ast::{
	BinOp, CallExpr, CharLit, EnumType, Expr, Field, FloatLit, Ident, IntLit, InterfaceType, Method,
	OpKind, PrimitiveType, PtrType, Spanned, StringLit, StructExpr, Unary,
};
use pi_error::{PIErrorCode, Span};
use pi_lexer::token::TokenKind;

use crate::{decl::params, get_tokprec, tok_val, token_kind_to_op_kind, ParseInput};

pub fn expr(input: &mut ParseInput) -> Spanned<Expr> {
	binop_expr(input, 1)
}

fn binop_expr(input: &mut ParseInput, prec1: i8) -> Spanned<Expr> {
	let mut x = unary_expr(input);
	loop {
		let oprec = get_tokprec(&input.tok().kind);
		let op = input.tok().kind.clone();
		if oprec < prec1 {
			// this is kind of a dirty hack, but i think it works
			// basically, when you encounter `foo()`, we should return `postfix()`
			// but if we have, say, `foo.bar()`, we wait til afterwards to return `postfix()`
			if prec1 == 1 {
				return postfix(input, x);
			} else {
				return x;
			}
		}

		input.next();

		let y = binop_expr(input, oprec + 1);

		let binop_start = x.span.range.start.clone();
		let binop_end = y.span.range.end.clone();
		let binop = Expr::BinOp(BinOp::new(
			Box::from(x.clone()),
			token_kind_to_op_kind(&op),
			Box::from(y),
		));
		let post = postfix(
			input,
			Spanned::new(binop, Span::new(binop_start..binop_end, input.file_id)),
		);
		x = post;
	}
}

fn postfix(input: &mut ParseInput, x: Spanned<Expr>) -> Spanned<Expr> {
	match input.tok().kind {
		TokenKind::LParen => call(input, x),
		_ => x,
	}
}

fn call(input: &mut ParseInput, x: Spanned<Expr>) -> Spanned<Expr> {
	let start = input.tok().span.start;
	input.expect(
		TokenKind::LParen,
		input.error(
			"expected `(` at beginning of call expression".to_owned(),
			PIErrorCode::ParseExpectedLParenBeforeCallExpr,
			vec![],
		),
	);
	if input.tok().kind == TokenKind::LParen {
		input.next();
	}

	let args_start = input.tok().span.start;
	let mut args = vec![];
	while input.tok().kind != TokenKind::RParen && input.tok().kind != TokenKind::EOF {
		let arg = expr(input);
		args.push(Box::from(arg));

		if input.tok().kind != TokenKind::RParen {
			input.expect(
				TokenKind::Comma,
				input.error(
					"expected `,` in call args".to_owned(),
					PIErrorCode::ParseExpectedCommaInCallArgs,
					vec![],
				),
			);
			if input.tok().kind == TokenKind::Comma {
				input.next();
			} else {
				break;
			}
		}
	}
	let args_end = input.tok().span.start;

	input.expect(
		TokenKind::RParen,
		input.error(
			"expected `)` at beginning of call expression".to_owned(),
			PIErrorCode::ParseExpectedRParenAfterCallExpr,
			vec![],
		),
	);
	if input.tok().kind == TokenKind::RParen {
		input.next();
	}
	let end = input.tok().span.start;
	return Spanned::new(
		Expr::CallExpr(CallExpr::new(
			Box::from(x),
			Spanned::new(args, Span::new(args_start..args_end, input.file_id)),
		)),
		Span::new(start..end, input.file_id),
	);
}

fn unary_expr(input: &mut ParseInput) -> Spanned<Expr> {
	let kind = input.tok().kind.clone();
	match kind {
		TokenKind::Ampersand => {
			let start = input.tok().span.start;
			input.next();
			let e = expr(input);
			let end = input.tok().span.start;
			Spanned::new(
				Expr::Unary(Unary::new(OpKind::Ampersand, Box::from(e))),
				Span::new(start..end, input.file_id),
			)
		}
		_ => primary_expr(input),
	}
}

fn primary_expr(input: &mut ParseInput) -> Spanned<Expr> {
	let x = operand(input);
	// if let Expr::Ident(ident) = &x.node {
	// 	if input.tok().kind == TokenKind::LBrace {
	// 		return struct_expr(input, ident);
	// 	}
	// }
	return x;
}

fn struct_expr(input: &mut ParseInput, name: &Ident) -> Spanned<Expr> {
	let fields_start = input.tok().span.start;
	input.next();
	let mut fields = IndexMap::new();
	while input.tok().kind != TokenKind::RBrace && input.tok().kind != TokenKind::EOF {
		let ident = ident(input);
		if input.tok().kind == TokenKind::Colon {
			input.next();
			let val = expr(input);
			fields.insert(ident, Some(Box::from(val)));

			if input.tok().kind != TokenKind::Comma {
				input.expect(
					TokenKind::RBrace,
					input.error(
						"expected either `,` or `}` in struct expression".to_owned(),
						PIErrorCode::ParseExpectedCommaOrRBraceStructExpr,
						vec![],
					),
				);
				if input.tok().kind != TokenKind::RBrace {
					break;
				}
			} else {
				input.next();
			}
		} else if input.tok().kind == TokenKind::Comma {
			input.next();
			fields.insert(ident, None);
			continue;
		} else {
			input.errs.push(input.error(
				format!(
					"unexpected token `{}` in struct expression",
					tok_val(&input.program, input.tok())
				),
				PIErrorCode::ParseUnexpectedTokenStructExpr,
				vec![],
			));
			break;
		}
	}

	if input.tok().kind == TokenKind::RBrace {
		input.next();
	}
	let fields_end = input.tok().span.start;
	Spanned::new(
		Expr::StructExpr(StructExpr::new(
			name.clone(),
			Spanned::new(fields, Span::new(fields_start..fields_end, input.file_id)),
		)),
		Span::new(fields_start..fields_end, input.file_id),
	)
}

fn operand(input: &mut ParseInput) -> Spanned<Expr> {
	match input.tok().kind {
		TokenKind::Ident => {
			let e = ident(input);
			Spanned::new(
				Expr::Ident(e.node),
				Span::new(e.span.range.start..e.span.range.end, input.file_id),
			)
		}
		TokenKind::Minus
		| TokenKind::Int
		| TokenKind::Float
		| TokenKind::CharLit
		| TokenKind::StringLit => basic_lit(input),
		// TokenKind::Nil => input.nil_expr(),
		_ => {
			input.errs.push(input.error(
				"unexpected expression operand".to_owned(),
				PIErrorCode::ParseUnexpectedExprOperand,
				vec![(
					format!(
						"unexpected expression operand `{}`",
						tok_val(&input.program, input.tok())
					),
					Span::new(input.tok().span.clone(), input.file_id),
				)],
			));
			Spanned::new(Expr::Error, Span::new(0..0, input.file_id))
		}
	}
}

fn basic_lit(input: &mut ParseInput) -> Spanned<Expr> {
	let begin_pos = input.tok().span.start;
	let mut val_start = input.tok().span.start;
	let sign_span = begin_pos..begin_pos + 1;
	let mut signed = false;
	if input.tok().kind == TokenKind::Minus {
		signed = true;
		input.next();
		val_start += 1;
	}

	input.expect_range(
		TokenKind::BasicLitBegin,
		TokenKind::BasicLitEnd,
		input.error(
			"expected a basic literal expression".to_owned(),
			PIErrorCode::ParseExpectedBasicLit,
			vec![(
				format!(
					"expected a basic literal expression, instead got `{}`",
					tok_val(&input.program, input.tok())
				),
				Span::new(input.tok().span.clone(), input.file_id),
			)],
		),
	);

	match input.tok().kind {
		TokenKind::Int => {
			let mut str_val = tok_val(&input.program, &input.tok());
			let base = if str_val.len() > 2 {
				match &str_val.as_str()[0..2] {
					"0x" => {
						str_val = str_val.as_str()[2..].to_string();
						16
					}
					"08" => {
						str_val = str_val.as_str()[2..].to_string();
						8
					}
					"0b" => {
						str_val = str_val.as_str()[2..].to_string();
						2
					}
					_ => 10,
				}
			} else {
				10
			};

			let int_witout_separators = str_val.replace("_", "");
			let x = u64::from_str_radix(int_witout_separators.as_str(), base);
			let end_pos = input.tok().span.end;
			input.next();
			match x {
				Ok(val) => Spanned::new(
					Expr::IntLit(IntLit::new(
						Spanned::new(signed, Span::new(sign_span, input.file_id)),
						true,
						32,
						Spanned::new(val, Span::new(val_start..end_pos, input.file_id)),
					)),
					Span::new(begin_pos..end_pos, input.file_id),
				),
				Err(e) => {
					input.errs.push(input.error(
						format!("could not parse integer: {}", e.to_string()),
						PIErrorCode::ParseCouldNotParseInt,
						vec![
							(
								"invalid integer".to_owned(),
								Span::new(input.tok().span.clone(), input.file_id),
							),
							(
								format!("(hint) this is a base {} integer", base).to_owned(),
								Span::new(input.tok().span.clone(), input.file_id),
							),
						],
					));
					Spanned::new(Expr::Error, Span::new(0..0, input.file_id))
				}
			}
		}
		TokenKind::Float => {
			let str_val = tok_val(&input.program, &input.tok());
			let float_witout_separators = str_val.replace("_", "");
			let x = float_witout_separators.parse::<f64>();
			let end_pos = input.tok().span.end;
			input.next();
			match x {
				Ok(val) => Spanned::new(
					Expr::FloatLit(FloatLit::new(
						Spanned::new(signed, Span::new(sign_span, input.file_id)),
						32,
						Spanned::new(val, Span::new(val_start..end_pos, input.file_id)),
					)),
					Span::new(begin_pos..end_pos, input.file_id),
				),
				_ => Spanned::new(Expr::Error, Span::new(0..0, input.file_id)),
			}
		}
		TokenKind::CharLit => {
			let start = input.tok().span.start;
			let x = tok_val(&input.program, &input.tok());
			input.next();
			match x.chars().nth(0) {
				Some(val) => Spanned::new(
					Expr::CharLit(CharLit::from(val)),
					Span::new(start..input.tok().span.start, input.file_id),
				),
				_ => Spanned::new(Expr::Error, Span::new(0..0, input.file_id)),
			}
		}
		TokenKind::StringLit => {
			let start = input.tok().span.start;
			let x = tok_val(&input.program, &input.tok());
			input.next();
			Spanned::new(
				Expr::StringLit(StringLit::from(x)),
				Span::new(start..input.tok().span.start, input.file_id),
			)
		}
		_ => {
			input.next();
			Spanned::new(Expr::Error, Span::new(0..0, input.file_id))
		}
	}
}

pub fn ident(input: &mut ParseInput) -> Spanned<Ident> {
	let start = input.tok().span.start;
	let input_program_clone = input.program.clone();
	let res = input.expect(
		TokenKind::Ident,
		input.error(
			"expected identifier".to_owned(),
			PIErrorCode::ParseExpectedIdent,
			vec![(
				"".to_owned(),
				Span::new(input.tok().span.clone(), input.file_id),
			)],
		),
	);
	let x = Ident::from(tok_val(&input_program_clone, res));
	input.next();
	return Spanned::new(x, Span::new(start..input.tok().span.start, input.file_id));
}

fn enum_type_expr(input: &mut ParseInput) -> Spanned<Expr> {
	let start = input.tok().span.start;
	input.next();

	input.expect(
		TokenKind::LBrace,
		input.error(
			"expected `{` in enum type expression".to_owned(),
			PIErrorCode::ParseExpectedLBraceInEnumTypeExpr,
			vec![],
		),
	);
	if input.tok().kind == TokenKind::LBrace {
		input.next();
	}

	let types = enum_types(input);

	input.expect(
		TokenKind::RBrace,
		input.error(
			"expected `}` in enum type expression".to_owned(),
			PIErrorCode::ParseExpectedRBraceInEnumTypeExpr,
			vec![],
		),
	);
	if input.tok().kind == TokenKind::RBrace {
		input.next();
	}

	let end = input.tok().span.start;
	return Spanned::new(Expr::EnumType(types), Span::new(start..end, input.file_id));
}

fn enum_types(input: &mut ParseInput) -> EnumType {
	let mut types = EnumType::new();
	while input.tok().kind != TokenKind::RBrace && input.tok().kind != TokenKind::EOF {
		let (k, v) = enum_type(input);
		types.insert(k, v);
	}
	types
}

fn enum_type(input: &mut ParseInput) -> (Spanned<Ident>, Spanned<Expr>) {
	let name = ident(input);

	let mut ty = Spanned::new(
		Expr::PrimitiveType(PrimitiveType::U8),
		Span::new(input.tok().span.clone(), input.file_id),
	);
	if input.tok().kind == TokenKind::Arrow {
		input.next();
		ty = type_expr(input);
	}

	if input.tok().kind == TokenKind::Comma {
		input.next();
	}

	(name, ty)
}

fn struct_type_expr(input: &mut ParseInput) -> Spanned<Expr> {
	let start = input.tok().span.start;
	input.expect(
		TokenKind::Struct,
		input.error(
			"expected `struct` in struct type expression".to_owned(),
			PIErrorCode::ParseExpectedStructInStructTypeExpr,
			vec![],
		),
	);
	if input.tok().kind == TokenKind::Struct {
		input.next();
	}

	input.expect(
		TokenKind::LBrace,
		input.error(
			"expected `{` in struct type expression".to_owned(),
			PIErrorCode::ParseExpectedLBraceInStructTypeExpr,
			vec![],
		),
	);
	if input.tok().kind == TokenKind::LBrace {
		input.next();
	}

	let fields = field_map(input);

	input.expect(
		TokenKind::RBrace,
		input.error(
			"expected `}` in struct type expression".to_owned(),
			PIErrorCode::ParseExpectedRBraceInStructTypeExpr,
			vec![],
		),
	);
	if input.tok().kind == TokenKind::RBrace {
		input.next();
	}

	let end = input.tok().span.start;
	return Spanned::new(
		Expr::StructType(fields),
		Span::new(start..end, input.file_id),
	);
}

fn field_map(input: &mut ParseInput) -> IndexMap<Spanned<Ident>, Spanned<Field>> {
	let mut fields = IndexMap::new();
	while input.tok().kind != TokenKind::RBrace && input.tok().kind != TokenKind::EOF {
		let (k, v) = field(input);
		fields.insert(k, v);
	}
	fields
}

fn field(input: &mut ParseInput) -> (Spanned<Ident>, Spanned<Field>) {
	let start = input.tok().span.start;
	let mut pub_ = false;
	let mut pub_end = input.tok().span.start;
	if input.tok().kind == TokenKind::Pub {
		pub_ = true;
		input.next();
		pub_end = input.tok().span.start;
	}
	let type_ = type_expr(input);

	input.expect(
		TokenKind::Ident,
		input.error(
			"expected identifier in field".to_owned(),
			PIErrorCode::ParseExpectedIdentInField,
			vec![(
				"(hint) give your field a name".to_owned(),
				Span::new(input.tok().span.clone(), input.file_id),
			)],
		),
	);
	let mut name = String::new();
	let ident_begin = input.tok().span.start;
	let mut ident_end = input.tok().span.start;
	if input.tok().kind == TokenKind::Ident {
		name = tok_val(&input.program, &input.tok());
		input.next();
		ident_end = input.tok().span.start;
	}

	if input.tok().kind == TokenKind::Semicolon {
		input.next();
		return (
			Spanned::new(
				Ident::from(name),
				Span::new(ident_begin..ident_end, input.file_id),
			),
			Spanned::new(
				Field::new(
					Spanned::new(pub_, Span::new(start..pub_end, input.file_id)),
					type_,
					None,
				),
				Span::new(start..input.tok().span.start, input.file_id),
			),
		);
	}

	input.expect(
		TokenKind::Eq,
		input.error(
			"expected `=` in field".to_owned(),
			PIErrorCode::ParseExpectedEqInField,
			vec![(
				"(hint) either terminate your field with a `;` or give it a default value".to_owned(),
				Span::new(input.tok().span.clone(), input.file_id),
			)],
		),
	);
	if input.tok().kind == TokenKind::Eq {
		input.next();
	}

	let val = expr(input);

	input.expect(
		TokenKind::Semicolon,
		input.error(
			"expected `;` at end of field".to_owned(),
			PIErrorCode::ParseExpectedSemicolonInField,
			vec![(
				"(hint) insert a `;` here".to_owned(),
				Span::new(input.tok().span.clone(), input.file_id),
			)],
		),
	);
	if input.tok().kind == TokenKind::Semicolon {
		input.next();
	}
	return (
		Spanned::new(
			Ident::from(name),
			Span::new(ident_begin..ident_end, input.file_id),
		),
		Spanned::new(
			Field::new(
				Spanned::new(pub_, Span::new(start..pub_end, input.file_id)),
				type_,
				Some(val),
			),
			Span::new(start..input.tok().span.start, input.file_id),
		),
	);
}

fn interface_type_expr(input: &mut ParseInput) -> Spanned<Expr> {
	let start = input.tok().span.start;
	input.expect(
		TokenKind::Interface,
		input.error(
			"expected `interface` in interface type expression".to_owned(),
			PIErrorCode::ParseExpectedInterfaceInInterfaceTypeExpr,
			vec![],
		),
	);
	if input.tok().kind == TokenKind::Interface {
		input.next();
	}

	input.expect(
		TokenKind::LBrace,
		input.error(
			"expected `{` in interface type expression".to_owned(),
			PIErrorCode::ParseExpectedLBraceInInterfaceTypeExpr,
			vec![],
		),
	);
	if input.tok().kind == TokenKind::LBrace {
		input.next();
	}

	input.inside_apply_or_interface = true;
	let methods = method_map(input);
	input.inside_apply_or_interface = false;

	input.expect(
		TokenKind::RBrace,
		input.error(
			"expected `}` after interface type expression".to_owned(),
			PIErrorCode::ParseExpectedRBraceInInterfaceTypeExpr,
			vec![],
		),
	);
	if input.tok().kind == TokenKind::RBrace {
		input.next();
	}

	let end = input.tok().span.start;
	return Spanned::new(
		Expr::InterfaceType(methods),
		Span::new(start..end, input.file_id),
	);
}

fn method_map(input: &mut ParseInput) -> InterfaceType {
	let mut methods = HashMap::new();
	while input.tok().kind != TokenKind::RBrace && input.tok().kind != TokenKind::EOF {
		let (k, v) = method(input);
		methods.insert(k, v);
	}
	return methods;
}

fn method(input: &mut ParseInput) -> (Spanned<Ident>, Spanned<Method>) {
	let mut pub_ = false;
	let pub_start = input.tok().span.start;
	let mut pub_end = input.tok().span.start;
	if input.tok().kind == TokenKind::Pub {
		pub_ = true;
		input.next();
		pub_end = input.tok().span.start;
	}

	input.expect(
		TokenKind::Fn,
		input.error(
			"expected `fn` in interface method declaration".to_owned(),
			PIErrorCode::ParseExpectedFnInInterfaceMethod,
			vec![],
		),
	);
	if input.tok().kind == TokenKind::Fn {
		input.next();
	}

	let name = ident(input);
	let params_start = input.tok().span.start;
	let params = params(input);
	let params_end = input.tok().span.end;
	let ret_ty_start = input.tok().span.start;
	let mut ret_ty = Spanned::new(
		Expr::PrimitiveType(PrimitiveType::Void),
		Span::new(ret_ty_start..ret_ty_start, input.file_id),
	);
	if input.tok().kind == TokenKind::Arrow {
		input.next();
		ret_ty = type_expr(input);
	}

	input.expect(
		TokenKind::Semicolon,
		input.error(
			"expected `;` after method in interface type expression method list".to_owned(),
			PIErrorCode::ParseExpectedSemicolonAfterMethodInInterfaceTypeMethodList,
			vec![],
		),
	);
	if input.tok().kind == TokenKind::Semicolon {
		input.next();
	}

	let end = input.tok().span.end;
	return (
		name.clone(),
		Spanned::new(
			Method::new(
				Spanned::new(pub_, Span::new(pub_start..pub_end, input.file_id)),
				name,
				Spanned::new(params, Span::new(params_start..params_end, input.file_id)),
				ret_ty,
			),
			Span::new(pub_start..end, input.file_id),
		),
	);
}

pub fn type_expr(input: &mut ParseInput) -> Spanned<Expr> {
	let mut ty = match input.tok().kind {
		TokenKind::I64
		| TokenKind::U64
		| TokenKind::I32
		| TokenKind::U32
		| TokenKind::I16
		| TokenKind::U16
		| TokenKind::I8
		| TokenKind::U8
		| TokenKind::F64
		| TokenKind::F32
		| TokenKind::Bool => {
			let start = input.tok().span.start;
			let y = token_kind_to_primitive_kind(input, input.tok().kind.clone());
			input.next();
			let end = input.tok().span.start;
			Spanned::new(Expr::PrimitiveType(y), Span::new(start..end, input.file_id))
		}
		TokenKind::Struct => struct_type_expr(input),
		TokenKind::Interface => interface_type_expr(input),
		TokenKind::Enum => enum_type_expr(input),
		TokenKind::Ident => {
			let e = ident(input);
			Spanned::new(
				Expr::Ident(e.node),
				Span::new(e.span.range.start..e.span.range.end, input.file_id),
			)
		}
		_ => {
			input.errs.push(input.error(
				"expected type expression".to_owned(),
				PIErrorCode::ParseExpectedTypeExpr,
				vec![(
					format!(
						"expected type expression, instead got `{}`",
						tok_val(&input.program, input.tok())
					),
					Span::new(input.tok().span.clone(), input.file_id),
				)],
			));
			Spanned::new(Expr::Error, Span::new(0..0, input.file_id))
		}
	};
	while input.tok().kind == TokenKind::Asterisk {
		ty = Spanned::new(
			Expr::PtrType(PtrType::from(ty.clone())),
			Span::new(ty.span.range.start..input.tok().span.end, input.file_id),
		);
		input.next();
	}
	return ty;
}

fn token_kind_to_primitive_kind(input: &mut ParseInput, tok_kind: TokenKind) -> PrimitiveType {
	match tok_kind {
		TokenKind::I64 => PrimitiveType::I64,
		TokenKind::U64 => PrimitiveType::U64,
		TokenKind::I32 => PrimitiveType::I32,
		TokenKind::U32 => PrimitiveType::U32,
		TokenKind::I16 => PrimitiveType::I16,
		TokenKind::U16 => PrimitiveType::U16,
		TokenKind::I8 => PrimitiveType::I8,
		TokenKind::U8 => PrimitiveType::U8,
		TokenKind::F64 => PrimitiveType::F64,
		TokenKind::F32 => PrimitiveType::F32,
		TokenKind::Bool => PrimitiveType::Bool,
		_ => {
			input.fatal_error(
				format!(
					"internal compiler error: could not convert token kind `{}` to a primitive type kind",
					tok_kind
				),
				PIErrorCode::ParseCouldNotConvertTokKindToPrimitiveType,
				vec![],
			);
			return PrimitiveType::Void;
		}
	}
}
