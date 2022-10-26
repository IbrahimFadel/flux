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
    let end = if let Some(last_child) = node.last_child_or_token() {
        match last_child.as_node() {
            Some(node) => trim_trailing_whitesapce(node).end(),
            None => {
                let tok = last_child.as_token().unwrap();
                if tok.kind() == SyntaxKind::Whitespace {
                    tok.text_range().start()
                } else {
                    tok.text_range().end()
                }
            }
        }
    } else {
        return node.text_range();
    };
    TextRange::new(node.text_range().start(), end)
}

// fn get_node_range(node: &SyntaxNode) -> TextRange {
//     let children = node.children_with_tokens();
//     if let Some(child) = children.last() {
//         if let Some(tok) = child.as_token() {
//             if tok.kind() == SyntaxKind::Whitespace {}
//         }
//     }

//     todo!()
// }

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
