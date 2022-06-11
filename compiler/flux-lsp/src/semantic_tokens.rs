use flux_syntax::{
	ast::{AstNode, Root},
	syntax_kind::SyntaxKind,
};
use text_size::{TextRange, TextSize};
use tower_lsp::lsp_types::{SemanticToken, SemanticTokenType};

use crate::position::flux_range_to_position;

pub const LEGEND_TYPE: &[SemanticTokenType] = &[
	SemanticTokenType::FUNCTION,
	SemanticTokenType::VARIABLE,
	SemanticTokenType::STRING,
	SemanticTokenType::COMMENT,
	SemanticTokenType::NUMBER,
	SemanticTokenType::KEYWORD,
	SemanticTokenType::OPERATOR,
	SemanticTokenType::PARAMETER,
	SemanticTokenType::ENUM_MEMBER,
	SemanticTokenType::NAMESPACE,
	SemanticTokenType::TYPE,
	SemanticTokenType::REGEXP,
];

struct SemanticTokenBuilder<'a> {
	pre_line: u32,
	pre_start: u32,
	src: &'a str,
	tokens: Vec<SemanticToken>,
}

impl<'a> SemanticTokenBuilder<'a> {
	pub fn new(src: &'a str) -> Self {
		Self {
			pre_line: 0,
			pre_start: 0,
			src,
			tokens: vec![],
		}
	}

	fn basic_tok(
		&mut self,
		tok: Option<flux_syntax::syntax_kind::SyntaxToken>,
		ty: SemanticTokenType,
	) {
		if let Some(tok) = tok {
			let range = tok.text_range();
			let pos = flux_range_to_position(range, self.src);
			if pos.end.line > pos.start.line {
				self.multiline_tok(range, ty);
				return;
			}
			let length = u32::from(range.end() - range.start());
			self.tokens.push(SemanticToken {
				delta_line: pos.start.line - self.pre_line,
				delta_start: if pos.start.character >= self.pre_start {
					pos.start.character - self.pre_start
				} else {
					pos.start.character
				},
				length,
				token_type: get_token_type(ty),
				token_modifiers_bitset: 0,
			});
			self.pre_line = pos.start.line;
			self.pre_start = pos.start.character;
		}
	}

	fn multiline_tok(&mut self, range: TextRange, ty: SemanticTokenType) {
		let initial_start = usize::from(range.start());
		let initial_end = usize::from(range.end());

		let mut total_length = 0;
		let full_str = &self.src[initial_start..initial_end];
		let lines = full_str.split('\n');
		for line in lines {
			let length = line.len();
			let start = initial_start + total_length;
			let end = start + line.len();
			let pos = flux_range_to_position(
				TextRange::new(TextSize::from(start as u32), TextSize::from(end as u32)),
				self.src,
			);
			self.tokens.push(SemanticToken {
				delta_line: pos.start.line - self.pre_line,
				delta_start: if pos.start.character >= self.pre_start {
					pos.start.character - self.pre_start
				} else {
					pos.start.character
				},
				length: length as u32,
				token_type: get_token_type(ty.clone()),
				token_modifiers_bitset: 0,
			});
			self.pre_line = pos.start.line;
			self.pre_start = pos.start.character;
			total_length += 1 + length as usize;
		}
	}

	pub fn tokens_at_level(&mut self, syntax: &flux_syntax::syntax_kind::SyntaxNode) {
		syntax.children_with_tokens().for_each(|node_or_tok| {
			if let Some(node) = node_or_tok.as_node() {
				self.tokens_at_level(node);
			} else if let Some(token) = node_or_tok.as_token() {
				if token.kind() != SyntaxKind::Whitespace {
					let ty = self.token_to_semantic_token(token);
					self.basic_tok(Some(token.clone()), ty);
				}
			}
		})
	}

	fn token_to_semantic_token(
		&mut self,
		tok: &flux_syntax::syntax_kind::SyntaxToken,
	) -> SemanticTokenType {
		match tok.kind() {
			SyntaxKind::Comment => SemanticTokenType::COMMENT,
			SyntaxKind::ModKw
			| SyntaxKind::UseKw
			| SyntaxKind::PubKw
			| SyntaxKind::FnKw
			| SyntaxKind::TypeKw
			| SyntaxKind::MutKw
			| SyntaxKind::IfKw
			| SyntaxKind::ElseKw
			| SyntaxKind::StructKw
			| SyntaxKind::InterfaceKw
			| SyntaxKind::LetKw
			| SyntaxKind::ReturnKw => SemanticTokenType::KEYWORD,
			// SyntaxKind::LParen | SyntaxKind::RParen => SemanticTokenType::REGEXP,
			// custom token type for separators
			SyntaxKind::Eq | SyntaxKind::Plus | SyntaxKind::Minus | SyntaxKind::DoubleColon => {
				SemanticTokenType::OPERATOR
			}
			SyntaxKind::IntExpr | SyntaxKind::FloatExpr => SemanticTokenType::NUMBER,
			SyntaxKind::INKw
			| SyntaxKind::UNKw
			| SyntaxKind::F64Kw
			| SyntaxKind::F32Kw
			| SyntaxKind::BoolKw => SemanticTokenType::TYPE,
			SyntaxKind::Ident => {
				if let Some(parent) = tok.parent() {
					if parent.kind() == SyntaxKind::FnDecl {
						return SemanticTokenType::FUNCTION;
					} else if parent.kind() == SyntaxKind::ModDecl {
						return SemanticTokenType::NAMESPACE;
					} else if parent.kind() == SyntaxKind::PathExpr {
						let mut lparen_found = false;
						for prev in parent.siblings_with_tokens(rowan::Direction::Prev) {
							if let Some(prev) = prev.as_token() {
								if prev.kind() == SyntaxKind::LParen {
									lparen_found = true;
								}
							}
						}
						if let Some(parent_parent) = parent.parent() {
							if parent_parent.kind() == SyntaxKind::UseDecl {
								return SemanticTokenType::NAMESPACE;
							} else if parent_parent.kind() == SyntaxKind::CallExpr {
								if tok.next_sibling_or_token().is_none() {
									if lparen_found {
										if let Some(prev) = tok.prev_sibling_or_token() {
											if let Some(prev) = prev.as_token() {
												if prev.kind() == SyntaxKind::DoubleColon {
													return SemanticTokenType::VARIABLE;
												}
											}
										}
										return SemanticTokenType::PARAMETER;
									}
									return SemanticTokenType::FUNCTION;
								} else {
									return SemanticTokenType::NAMESPACE;
								}
							} else {
								return SemanticTokenType::VARIABLE;
							}
						}
					}
				}

				return SemanticTokenType::VARIABLE;
			}
			_ => SemanticTokenType::VARIABLE,
		}
	}
}

pub fn cst_to_semantic_tokens(root: &Root, src: &str) -> Vec<SemanticToken> {
	let mut builder = SemanticTokenBuilder::new(src);
	builder.tokens_at_level(root.syntax());
	builder.tokens
}

fn get_token_type(ty: SemanticTokenType) -> u32 {
	LEGEND_TYPE.iter().position(|item| item == &ty).unwrap() as u32
}
