use crate::{errors::ParseError, Parse};

use super::event::Event;
use cstree::{
	interning::{IntoResolver, Resolver},
	GreenNodeBuilder, Language,
};
use flux_lexer::{Token, TokenKind};
use flux_syntax::syntax_kind::FluxLanguage;
use std::mem;

pub struct Sink<'t, 'src> {
	builder: GreenNodeBuilder<'static, 'static>,
	tokens: &'t [Token<'src>],
	cursor: usize,
	events: Vec<Event>,
	errors: Vec<ParseError>,
}

impl<'t, 'src> Sink<'t, 'src> {
	pub fn new(tokens: &'t [Token<'src>], events: Vec<Event>) -> Self {
		Self {
			builder: GreenNodeBuilder::new(),
			tokens,
			cursor: 0,
			events,
			errors: vec![],
		}
	}

	pub fn finish(mut self) -> Parse<impl Resolver> {
		for idx in 0..self.events.len() {
			match mem::replace(&mut self.events[idx], Event::Placeholder) {
				Event::StartNode {
					kind,
					forward_parent,
				} => {
					let mut kinds = vec![kind];

					let mut idx = idx;
					let mut forward_parent = forward_parent;

					while let Some(fp) = forward_parent {
						idx += fp;

						forward_parent = if let Event::StartNode {
							kind,
							forward_parent,
						} = mem::replace(&mut self.events[idx], Event::Placeholder)
						{
							kinds.push(kind);
							forward_parent
						} else {
							unreachable!()
						};
					}

					for kind in kinds.into_iter().rev() {
						self.builder.start_node(FluxLanguage::kind_to_raw(kind));
					}
				}
				Event::AddToken => self.token(),
				Event::FinishNode => self.builder.finish_node(),
				Event::Error(error) => self.errors.push(error),
				Event::Placeholder => {}
			}

			self.eat_trivia();
		}

		let (tree, cache) = self.builder.finish();

		Parse {
			green_node: tree,
			resolver: cache.unwrap().into_interner().unwrap().into_resolver(),
			errors: self.errors,
		}
	}

	fn token(&mut self) {
		let Token { kind, text, .. } = self.tokens[self.cursor];

		self
			.builder
			.token(FluxLanguage::kind_to_raw(kind.into()), text);

		self.cursor += 1;
	}

	fn eat_trivia(&mut self) {
		while let Some(token) = self.tokens.get(self.cursor) {
			if !TokenKind::from(token.kind).is_trivia() {
				break;
			}

			self.token();
		}
	}
}
