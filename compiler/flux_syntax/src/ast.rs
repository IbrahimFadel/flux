use cstree::{SyntaxElementRef, TextRange};

use crate::{SyntaxKind, SyntaxNode, SyntaxToken};

mod def;
pub use def::*;

macro_rules! getter {
    ($name:ident -> tok($tok_kind:ident); $($rest:tt)*) => {
        pub fn $name(&self) -> Option<&SyntaxToken> {
            self.syntax()
                .children_with_tokens()
                .filter_map(SyntaxElementRef::into_token)
                .find(|token| token.kind() == SyntaxKind::$tok_kind)
        }
        getter! {
            $($rest)*
        }
    };
    ($name:ident -> toks($tok_kind:ident); $($rest:tt)*) => {
        pub fn $name(&self) -> impl Iterator<Item = &SyntaxToken> {
            self.syntax()
                .children_with_tokens()
                .filter_map(SyntaxElementRef::into_token)
                .filter(|token| token.kind() == SyntaxKind::$tok_kind)
        }
        getter! {
            $($rest)*
        }
    };
    ($name:ident -> node($node_kind:ident); $($rest:tt)*) => {
        pub fn $name(&self) -> Option<$node_kind> {
            self.syntax().children().cloned().find_map($node_kind::cast)
        }
        getter! {
            $($rest)*
        }
    };
    ($name:ident -> nodes($node_kind:ident); $($rest:tt)*) => {
        pub fn $name(&self) -> impl Iterator<Item = $node_kind> + '_ {
            self.syntax().children().cloned().filter_map($node_kind::cast)
        }
        getter! {
            $($rest)*
        }
    };
    ($name:ident -> nth_node($node_kind:ident, $n:expr); $($rest:tt)*) => {
        pub fn $name(&self) -> Option<$node_kind> {
            self.syntax().children().cloned().filter_map($node_kind::cast).nth($n)
        }
        getter! {
            $($rest)*
        }
    };
    ($name:ident -> tok_matches(
        $($tok_kind:ident),*
    ); $($rest:tt)*) => {
        pub fn $name(&self) -> Option<&SyntaxToken> {
            self.syntax()
                .children_with_tokens()
                .filter_map(SyntaxElementRef::into_token)
                .find(|token| matches!(token.kind(), $(SyntaxKind::$tok_kind)|*))
        }
        getter! {
            $($rest)*
        }
    };
    () => {};
}

/// Generates the getter methods used in the AST layer
#[macro_export]
macro_rules! getters {
    (
        $struct_name:ident {
            $($getting:tt)*
        }
        $($rest:tt)*
    ) => {
        impl $struct_name {
            getter! {
                $($getting)*
            }
        }
        getters! {
            $($rest)*
        }
    };
    () => {};
}

#[macro_use]
mod getters;

#[macro_export]
macro_rules! basic_node {
    ($name:ident) => {
        #[derive(Debug, PartialEq, Eq, Clone)]
        pub struct $name(SyntaxNode);

        impl AstNode for $name {
            fn can_cast(kind: SyntaxKind) -> bool {
                match kind {
                    SyntaxKind::$name => true,
                    _ => false,
                }
            }

            fn cast(node: SyntaxNode) -> Option<Self> {
                match node.kind() {
                    SyntaxKind::$name => Some(Self(node)),
                    _ => None,
                }
            }

            fn syntax(&self) -> &SyntaxNode {
                &self.0
            }

            fn range(&self) -> TextRange {
                trim_trailing_whitesapce(&self.0)
            }

            fn is_poisoned(&self) -> bool {
                !self
                    .syntax()
                    .children()
                    .cloned()
                    .filter_map(Poisoned::cast)
                    .collect::<Vec<_>>()
                    .is_empty()
            }
        }
    };
}

fn trim_trailing_whitesapce(node: &SyntaxNode) -> TextRange {
    let len = node.children_with_tokens().len();
    if len == 0 {
        return node.text_range();
    }
    let mut i = len - 1;
    let start = node.text_range().start();
    let original_end = node.text_range().end();
    let mut end = original_end;
    loop {
        let child = node.children_with_tokens().nth(i);
        if let Some(child) = child {
            match child.as_node() {
                Some(node) => {
                    end = trim_trailing_whitesapce(node).end();
                    return TextRange::new(start, end);
                }
                None => {
                    let tok = child.as_token().unwrap();
                    if matches!(tok.kind(), SyntaxKind::Whitespace | SyntaxKind::Comment) {
                        end = tok.text_range().start();
                    } else {
                        end = tok.text_range().end();
                        return TextRange::new(start, end);
                    }
                }
            };
            if i == 0 {
                return TextRange::new(start, end);
            }
            i -= 1;
        } else {
            return TextRange::new(start, end);
        }
    }
}

#[macro_export]
macro_rules! enum_node {
	($name:ident: $($x:ident),*) => {
            #[derive(Debug)]
			pub enum $name {
				$($x($x)),+
			}

			impl AstNode for $name {
                fn can_cast(kind: SyntaxKind) -> bool {
					match kind {
						$(SyntaxKind::$x => true),+,
						_ => false,
					}
				}

				fn cast(syntax: SyntaxNode) -> Option<Self> {
					match syntax.kind() {
						$(SyntaxKind::$x => Some(Self::$x($x(syntax.clone())))),+,
						_ => None,
					}
				}

                fn syntax(&self) -> &SyntaxNode {
                    match self {
                        $($name::$x(node) => &node.0),+
                    }
                }

				fn range(&self) -> TextRange {
					self.syntax().text_range()
				}

                fn is_poisoned(&self) -> bool {
                    !self
                        .syntax()
                        .children()
                        .cloned()
                        .filter_map(Poisoned::cast)
                        .collect::<Vec<_>>()
                        .is_empty()
                }
			}

            $(
                impl From<$x> for $name {
                    fn from(value: $x) -> Self {
                        $name::$x(value)
                    }
                }
            )*
	};
}

pub trait AstNode {
    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxNode;

    fn range(&self) -> TextRange;

    fn is_poisoned(&self) -> bool;
}
