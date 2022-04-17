use std::vec;

use pi_ast::{
	BinOp, CallExpr, CharLit, Expr, Field, FloatLit, Ident, IntLit, Method, PrimitiveKind,
	PrimitiveType, PtrType, StringLit, Unary,
};
use pi_error::PIErrorCode;
use pi_lexer::token::TokenKind;
use smol_str::SmolStr;

use crate::{decl::params, get_tokprec, tok_val, token_kind_to_op_kind, ParseInput};

pub fn expr(input: &mut ParseInput) -> Expr {
	binop_expr(input, 1)
}

fn binop_expr(input: &mut ParseInput, prec1: i8) -> Expr {
	let mut x = unary_expr(input);
	loop {
		let oprec = get_tokprec(&input.tok().kind);
		let op = input.tok().kind.clone();
		if oprec < prec1 {
			return x;
		}

		input.next();

		let y = binop_expr(input, oprec + 1);

		let binop = Expr::BinOp(BinOp::new(
			Box::from(x.clone()),
			token_kind_to_op_kind(&op),
			Box::from(y),
		));
		let post = postfix(input, binop);
		x = post;
	}
}

fn postfix(input: &mut ParseInput, x: Expr) -> Expr {
	match input.tok().kind {
		TokenKind::LParen => call(input, x),
		_ => x,
	}
}

fn call(input: &mut ParseInput, x: Expr) -> Expr {
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

	let mut args = vec![];
	while input.tok().kind != TokenKind::RParen {
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
	return Expr::CallExpr(CallExpr::new(Box::from(x), args));
}

fn unary_expr(input: &mut ParseInput) -> Expr {
	let kind = input.tok().kind.clone();
	match kind {
		TokenKind::Ampersand => Expr::Unary(Unary::new(kind, Box::from(expr(input)))),
		_ => primary_expr(input),
	}
}

fn primary_expr(input: &mut ParseInput) -> Expr {
	let x = operand(input);
	return x;
	// loop {
	// match input.tok().kind {
	// TokenKind::Period =>

	// }
	// }
}

fn operand(input: &mut ParseInput) -> Expr {
	match input.tok().kind {
		TokenKind::Ident => Expr::Ident(ident(input)),
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
					input.tok().span.clone(),
				)],
			));
			Expr::Error
		}
	}
}

fn basic_lit(input: &mut ParseInput) -> Expr {
	let mut begin_pos = input.tok().span.start;
	let sign_span = begin_pos..begin_pos + 1;
	let mut signed = false;
	if input.tok().kind == TokenKind::Minus {
		signed = true;
		input.next();
		begin_pos += 1;
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
				input.tok().span.clone(),
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

			let x = u64::from_str_radix(str_val.as_str(), base);
			input.next();
			match x {
				Ok(val) => Expr::IntLit(IntLit::new(
					sign_span,
					begin_pos..input.tok().span.start,
					signed,
					32,
					val,
				)),
				Err(e) => {
					input.errs.push(input.error(
						format!("could not parse integer: {}", e.to_string()),
						PIErrorCode::ParseCouldNotParseInt,
						vec![
							("invalid integer".to_owned(), input.tok().span.clone()),
							(
								format!("(hint) this is a base {} integer", base).to_owned(),
								input.tok().span.clone(),
							),
						],
					));
					Expr::Error
				}
			}
		}
		TokenKind::Float => {
			let x = tok_val(&input.program, &input.tok()).parse::<f64>();
			input.next();
			match x {
				Ok(val) => Expr::FloatLit(FloatLit::new(
					sign_span,
					begin_pos..input.tok().span.start,
					signed,
					32,
					val,
				)),
				_ => Expr::Error,
			}
		}
		TokenKind::CharLit => {
			let x = tok_val(&input.program, &input.tok());
			input.next();
			match x.chars().nth(0) {
				Some(val) => Expr::CharLit(CharLit::from(val)),
				_ => Expr::Error,
			}
		}
		TokenKind::StringLit => {
			let x = tok_val(&input.program, &input.tok());
			input.next();
			Expr::StringLit(StringLit::from(x))
		}
		_ => {
			input.next();
			Expr::Error
		}
	}
}

pub fn ident(input: &mut ParseInput) -> Ident {
	let program_clone = input.program.clone(); // dirty hack because im too dumb to use rust
	let x = SmolStr::from(tok_val(
		&program_clone,
		input.expect(
			TokenKind::Ident,
			input.error(
				"expected identifier".to_owned(),
				PIErrorCode::ParseExpectedIdent,
				vec![("".to_owned(), input.tok().span.clone())],
			),
		),
	));
	input.next();
	return x;
}

fn struct_type_expr(input: &mut ParseInput) -> Expr {
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

	let fields = field_list(input);

	input.expect(
		TokenKind::RBrace,
		input.error(
			"expected `{` in struct type expression".to_owned(),
			PIErrorCode::ParseExpectedRBraceInStructTypeExpr,
			vec![],
		),
	);
	if input.tok().kind == TokenKind::RBrace {
		input.next();
	}

	return Expr::StructType(fields);
}

fn field_list(input: &mut ParseInput) -> Vec<Field> {
	let mut fields = vec![];
	while input.tok().kind != TokenKind::RBrace {
		let field = field(input);
		fields.push(field);
	}
	fields
}

fn field(input: &mut ParseInput) -> Field {
	let mut pub_ = false;
	if input.tok().kind == TokenKind::Pub {
		pub_ = true;
		input.next();
	}
	let type_ = type_expr(input);

	input.expect(
		TokenKind::Ident,
		input.error(
			"expected identifier in field".to_owned(),
			PIErrorCode::ParseExpectedIdentInField,
			vec![(
				"(hint) give your field a name".to_owned(),
				input.tok().span.clone(),
			)],
		),
	);
	let mut name = String::new();
	if input.tok().kind == TokenKind::Ident {
		name = tok_val(&input.program, &input.tok());
		input.next();
	}

	if input.tok().kind == TokenKind::Semicolon {
		input.next();
		return Field::new(pub_, type_, Ident::from(name), None);
	}

	input.expect(
		TokenKind::Eq,
		input.error(
			"expected `=` in field".to_owned(),
			PIErrorCode::ParseExpectedEqInField,
			vec![(
				"(hint) either terminate your field with a `;` or give it a default value".to_owned(),
				input.tok().span.clone(),
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
				input.tok().span.clone(),
			)],
		),
	);
	if input.tok().kind == TokenKind::Semicolon {
		input.next();
	}
	return Field::new(pub_, type_, Ident::from(name.as_str()), Some(val));
}

fn interface_type_expr(input: &mut ParseInput) -> Expr {
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

	let methods = method_list(input);

	input.expect(
		TokenKind::RBrace,
		input.error(
			"expected `{` in interface type expression".to_owned(),
			PIErrorCode::ParseExpectedRBraceInInterfaceTypeExpr,
			vec![],
		),
	);
	if input.tok().kind == TokenKind::RBrace {
		input.next();
	}

	return Expr::InterfaceType(methods);
}

fn method_list(input: &mut ParseInput) -> Vec<Method> {
	let mut methods = vec![];
	while input.tok().kind != TokenKind::RBrace {
		let method = method(input);
		methods.push(method);
	}
	methods
}

fn method(input: &mut ParseInput) -> Method {
	let mut pub_ = false;
	if input.tok().kind == TokenKind::Pub {
		pub_ = true;
		input.next();
	}

	let name = ident(input);
	let params = params(input);
	let mut ret_ty = Expr::PrimitiveType(PrimitiveType::new(PrimitiveKind::Void));
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

	return Method::new(pub_, name, params, ret_ty);
}

pub fn type_expr(input: &mut ParseInput) -> Expr {
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
			let y = token_kind_to_primitive_kind(input, input.tok().kind.clone());
			input.next();
			let x = Expr::PrimitiveType(PrimitiveType::new(y));
			return x;
		}
		TokenKind::Struct => struct_type_expr(input),
		TokenKind::Interface => interface_type_expr(input),
		TokenKind::Ident => Expr::Ident(ident(input)),
		_ => {
			input.errs.push(input.error(
				"expected type expression".to_owned(),
				PIErrorCode::ParseExpectedTypeExpr,
				vec![(
					format!(
						"expected type expression, instead got `{}`",
						tok_val(&input.program, input.tok())
					),
					input.tok().span.clone(),
				)],
			));
			Expr::Error
		}
	};
	while input.tok().kind == TokenKind::Asterisk {
		ty = Expr::PtrType(PtrType::from(ty.clone()));
		input.next();
	}
	return ty;
}

fn token_kind_to_primitive_kind(input: &mut ParseInput, tok_kind: TokenKind) -> PrimitiveKind {
	match tok_kind {
		TokenKind::I64 => PrimitiveKind::I64,
		TokenKind::U64 => PrimitiveKind::U64,
		TokenKind::I32 => PrimitiveKind::I32,
		TokenKind::U32 => PrimitiveKind::U32,
		TokenKind::I16 => PrimitiveKind::I16,
		TokenKind::U16 => PrimitiveKind::U16,
		TokenKind::I8 => PrimitiveKind::I8,
		TokenKind::U8 => PrimitiveKind::U8,
		TokenKind::F64 => PrimitiveKind::F64,
		TokenKind::F32 => PrimitiveKind::F32,
		TokenKind::Bool => PrimitiveKind::Bool,
		_ => {
			input.fatal_error(
				format!(
					"internal compiler error: could not convert token kind `{}` to a primitive type kind",
					tok_kind
				),
				PIErrorCode::ParseCouldNotConvertTokKindToPrimitiveKind,
				vec![],
			);
			return PrimitiveKind::Void;
		}
	}
}
