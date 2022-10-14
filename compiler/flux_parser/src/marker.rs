use flux_syntax::SyntaxKind;

use crate::event::Event;

use super::Parser;

pub(crate) struct Marker {
    pos: usize,
    completed: bool,
}

impl Drop for Marker {
    fn drop(&mut self) {
        if !self.completed {
            panic!("dopped uncompleted marker");
        }
    }
}

impl Marker {
    pub(super) fn new(pos: usize) -> Self {
        Self {
            pos,
            completed: false,
        }
    }

    pub(crate) fn complete(mut self, p: &mut Parser, kind: SyntaxKind) -> CompletedMarker {
        self.completed = true;

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

#[derive(Debug, Clone)]
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
