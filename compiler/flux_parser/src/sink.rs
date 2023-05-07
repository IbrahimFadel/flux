use cstree::{interning::TokenInterner, GreenNodeBuilder, Language, TextRange};
use flux_diagnostics::{Diagnostic, ToDiagnostic};
use flux_lexer::Token;
use flux_span::{FileId, ToSpan, WithSpan};
use flux_syntax::{FluxLanguage, SyntaxKind};
use lasso::ThreadedRodeo;

use crate::{diagnostics::ParserDiagnostic, event::Event, Parse};

pub struct Sink<'t, 'src> {
    builder: GreenNodeBuilder<'static, 'static, &'static ThreadedRodeo>,
    tokens: &'t [Token<'src>],
    cursor: usize,
    events: Vec<Event>,
    diagnostics: Vec<Diagnostic>,
}

impl<'t, 'src> Sink<'t, 'src> {
    pub(crate) fn new(
        tokens: &'t [Token<'src>],
        events: Vec<Event>,
        interner: &'static ThreadedRodeo,
    ) -> Self {
        Self {
            builder: GreenNodeBuilder::from_interner(interner),
            tokens,
            cursor: 0,
            events,
            diagnostics: vec![],
        }
    }

    pub(crate) fn finish(mut self, file_id: FileId) -> Parse {
        for idx in 0..self.events.len() {
            match std::mem::replace(&mut self.events[idx], Event::Placeholder) {
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
                        } =
                            std::mem::replace(&mut self.events[idx], Event::Placeholder)
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
                Event::Error(msg) => {
                    let range = self
                        .tokens
                        .get(self.cursor)
                        .map(|token| token.range)
                        .unwrap_or_else(|| {
                            self.tokens
                                .last()
                                .map_or(TextRange::new(0.into(), 0.into()), |token| token.range)
                        });
                    self.diagnostics.push(
                        ParserDiagnostic::Unxpected {
                            expected: msg.file_span(file_id, range.to_span()),
                        }
                        .to_diagnostic(),
                    );
                    self.builder
                        .start_node(FluxLanguage::kind_to_raw(SyntaxKind::Poisoned));
                    self.builder.finish_node();
                }
                Event::Placeholder => {}
            }

            self.eat_trivia();
        }

        let (tree, cache) = self.builder.finish();

        Parse {
            green_node: tree,
            diagnostics: self.diagnostics,
        }
    }

    fn token(&mut self) {
        let Token { kind, text, .. } = self.tokens[self.cursor];

        self.builder
            .token(FluxLanguage::kind_to_raw(kind.into()), text);

        self.cursor += 1;
    }

    fn eat_trivia(&mut self) {
        while let Some(token) = self.tokens.get(self.cursor) {
            if !token.kind.is_trivia() {
                break;
            }

            self.token();
        }
    }
}
