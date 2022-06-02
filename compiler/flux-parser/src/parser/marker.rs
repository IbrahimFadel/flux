use drop_bomb::DropBomb;

use crate::{event::Event, parser::Parser};
use flux_syntax::syntax_kind::SyntaxKind;

pub(crate) struct Marker {
	pos: usize,
	completed: DropBomb,
}

impl Marker {
	pub(crate) fn new(pos: usize) -> Self {
		Self {
			pos,
			completed: DropBomb::new("Marker not completed"),
		}
	}

	pub(crate) fn complete(mut self, p: &mut Parser, kind: SyntaxKind) -> CompletedMarker {
		self.completed.defuse();

		let event_at_pos = &mut p.events[self.pos];
		assert_eq!(*event_at_pos, Event::Placeholder);

		*event_at_pos = Event::StartNode {
			kind,
			forward_parent: None,
		};

		p.events.push(Event::FinishNode);

		CompletedMarker { pos: self.pos }
	}
}

#[derive(Debug)]
pub(crate) struct CompletedMarker {
	pos: usize,
}

impl CompletedMarker {
	pub(crate) fn precede(self, p: &mut Parser) -> Marker {
		let new_m = p.start();

		if let Event::StartNode {
			ref mut forward_parent,
			..
		} = p.events[self.pos]
		{
			*forward_parent = Some(new_m.pos - self.pos);
		} else {
			unreachable!();
		}

		new_m
	}
}
