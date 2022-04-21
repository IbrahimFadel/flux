#[cfg(test)]
#[test]
fn lexing() {
	use pi_error::{filesystem::FileId, PIErrorCode};

	use crate::{
		token::{Token, TokenKind},
		tokenize,
	};

	let tests: Vec<(&str, (Vec<Token>, Vec<PIErrorCode>))> = vec![
		(
			"0xff.0",
			(
				vec![Token::from(TokenKind::Float, 0..6)],
				vec![PIErrorCode::LexFloatInWrongBase],
			),
		),
		(
			r#""test"#,
			(
				vec![Token::from(TokenKind::StringLit, 0..5)],
				vec![PIErrorCode::LexStringLitUnterminated],
			),
		),
		(
			r#""foo\'""#,
			(
				vec![Token::from(TokenKind::StringLit, 0..7)],
				vec![PIErrorCode::LexUnknownEscapeSequence],
			),
		),
		(
			"'t",
			(
				vec![Token::from(TokenKind::CharLit, 0..2)],
				vec![PIErrorCode::LexCharLitUnterminated],
			),
		),
		(
			r#"'\x'"#,
			(
				vec![Token::from(TokenKind::CharLit, 0..4)],
				vec![
					PIErrorCode::LexUnknownEscapeSequence,
					PIErrorCode::LexInvalidCharLit,
				],
			),
		),
		(
			r#"'\"'"#,
			(
				vec![Token::from(TokenKind::CharLit, 0..4)],
				vec![
					PIErrorCode::LexUnknownEscapeSequence,
					PIErrorCode::LexInvalidCharLit,
				],
			),
		),
		(
			"'th'",
			(
				vec![Token::from(TokenKind::CharLit, 0..4)],
				vec![PIErrorCode::LexInvalidCharLit],
			),
		),
		(
			"/* hello",
			(
				vec![Token::from(TokenKind::BlockComment, 0..8)],
				vec![PIErrorCode::LexMissingEndOfBlockComment],
			),
		),
		(
			"0x",
			(
				vec![Token::from(TokenKind::Int, 0..2)],
				vec![PIErrorCode::LexExpectedDigitsFollowingIntPrefix],
			),
		),
		(
			"// foo",
			(vec![Token::from(TokenKind::LineComment, 0..6)], vec![]),
		),
		(
			"/* foo */",
			(vec![Token::from(TokenKind::BlockComment, 0..9)], vec![]),
		),
		(
			";(){}[]+-*&/,.=->:::",
			(
				vec![
					Token::from(TokenKind::Semicolon, 0..1),
					Token::from(TokenKind::LParen, 1..2),
					Token::from(TokenKind::RParen, 2..3),
					Token::from(TokenKind::LBrace, 3..4),
					Token::from(TokenKind::RBrace, 4..5),
					Token::from(TokenKind::LBracket, 5..6),
					Token::from(TokenKind::RBracket, 6..7),
					Token::from(TokenKind::Plus, 7..8),
					Token::from(TokenKind::Minus, 8..9),
					Token::from(TokenKind::Asterisk, 9..10),
					Token::from(TokenKind::Ampersand, 10..11),
					Token::from(TokenKind::Slash, 11..12),
					Token::from(TokenKind::Comma, 12..13),
					Token::from(TokenKind::Period, 13..14),
					Token::from(TokenKind::Eq, 14..15),
					Token::from(TokenKind::Arrow, 15..17),
					Token::from(TokenKind::DoubleColon, 17..19),
					Token::from(TokenKind::Colon, 19..20),
				],
				vec![],
			),
		),
		(
			"!===||&&<<=>>=",
			(
				vec![
					Token::from(TokenKind::CmpNE, 0..2),
					Token::from(TokenKind::CmpEQ, 2..4),
					Token::from(TokenKind::CmpOr, 4..6),
					Token::from(TokenKind::CmpAnd, 6..8),
					Token::from(TokenKind::CmpLT, 8..9),
					Token::from(TokenKind::CmpLTE, 9..11),
					Token::from(TokenKind::CmpGT, 11..12),
					Token::from(TokenKind::CmpGTE, 12..14),
				],
				vec![],
			),
		),
		(
			"pub fn return mut type interface struct nil if else mod apply to",
			(
				vec![
					Token::from(TokenKind::Pub, 0..3),
					Token::from(TokenKind::Fn, 4..6),
					Token::from(TokenKind::Return, 7..13),
					Token::from(TokenKind::Mut, 14..17),
					Token::from(TokenKind::Type, 18..22),
					Token::from(TokenKind::Interface, 23..32),
					Token::from(TokenKind::Struct, 33..39),
					Token::from(TokenKind::Nil, 40..43),
					Token::from(TokenKind::If, 44..46),
					Token::from(TokenKind::Else, 47..51),
					Token::from(TokenKind::Mod, 52..55),
					Token::from(TokenKind::Apply, 56..61),
					Token::from(TokenKind::To, 62..64),
				],
				vec![],
			),
		),
		(
			"i64 u64 i32 u32 i16 u16 i8 u8 f64 f32 bool",
			(
				vec![
					Token::from(TokenKind::I64, 0..3),
					Token::from(TokenKind::U64, 4..7),
					Token::from(TokenKind::I32, 8..11),
					Token::from(TokenKind::U32, 12..15),
					Token::from(TokenKind::I16, 16..19),
					Token::from(TokenKind::U16, 20..23),
					Token::from(TokenKind::I8, 24..26),
					Token::from(TokenKind::U8, 27..29),
					Token::from(TokenKind::F64, 30..33),
					Token::from(TokenKind::F32, 34..37),
					Token::from(TokenKind::Bool, 38..42),
				],
				vec![],
			),
		),
		("0xff", (vec![Token::from(TokenKind::Int, 0..4)], vec![])),
		("0810", (vec![Token::from(TokenKind::Int, 0..4)], vec![])),
		("0b10", (vec![Token::from(TokenKind::Int, 0..4)], vec![])),
		("0.12", (vec![Token::from(TokenKind::Float, 0..4)], vec![])),
		(
			r#""foo""#,
			(vec![Token::from(TokenKind::StringLit, 0..5)], vec![]),
		),
		(
			r#""fo\no""#,
			(vec![Token::from(TokenKind::StringLit, 0..7)], vec![]),
		),
		(
			r#""fo\"o""#,
			(vec![Token::from(TokenKind::StringLit, 0..7)], vec![]),
		),
		("'c'", (vec![Token::from(TokenKind::CharLit, 0..3)], vec![])),
		(
			"'\\n'",
			(vec![Token::from(TokenKind::CharLit, 0..4)], vec![]),
		),
		(
			"'\\r'",
			(vec![Token::from(TokenKind::CharLit, 0..4)], vec![]),
		),
		(
			"'\\t'",
			(vec![Token::from(TokenKind::CharLit, 0..4)], vec![]),
		),
		(
			"'\\''",
			(vec![Token::from(TokenKind::CharLit, 0..4)], vec![]),
		),
		(
			"f0o_bazz",
			(vec![Token::from(TokenKind::Ident, 0..8)], vec![]),
		),
	];

	for (input, (expected_toks, expected_errs)) in tests {
		let result = tokenize(input, FileId(0));
		let result_errcodes: Vec<PIErrorCode> = result.1.into_iter().map(|err| err.code).collect();
		assert_eq!(expected_toks, result.0[0..result.0.len() - 1]); // ignore EOF token
		assert_eq!(expected_errs, result_errcodes);
	}
}
