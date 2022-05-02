use std::{
	fs, io,
	process::{Command, Stdio},
};

use anyhow::Result;
use quote::{format_ident, quote};
use std::io::Write;
use ungrammar::{Grammar, Node, Rule};

pub(crate) struct Builder<'a> {
	grammar: &'a Grammar,
	nodes_program: String,
	tokens_program: String,
}

impl<'a> Builder<'a> {
	pub fn new(grammar: &'a Grammar) -> Self {
		Self {
			grammar,
			nodes_program: String::with_capacity(1000), // We know this string will be quite long, so this should speed things up a bit
			tokens_program: String::with_capacity(1000),
		}
	}

	pub fn generate_ast(&mut self) {
		self.build_imports();
		self.build_ast_node_trait();
		self.build_root();

		for node in self.grammar.iter() {
			match self.grammar[node].rule {
				Rule::Alt(_) => self.build_enum(node),
				Rule::Token(_) | Rule::Labeled { .. } => self.build_token(node),
				_ => self.build_struct(node),
			}
		}
	}

	pub fn write_ast_to_file(&self) -> io::Result<()> {
		let nodes_program = self.format(&self.nodes_program).unwrap();
		fs::write("./pi-syntax/src/generate/nodes.rs", nodes_program)?;
		let tokens_program = self.format(&self.tokens_program).unwrap();
		fs::write("./pi-syntax/src/generate/tokens.rs", tokens_program)
	}

	fn format(&self, s: impl std::fmt::Display) -> Result<String> {
		let mut rustfmt = Command::new("rustup")
			.args(&["run", "stable", "--", "rustfmt", "--config-path"])
			.arg("./rustfmt.toml")
			.stdin(Stdio::piped())
			.stdout(Stdio::piped())
			.spawn()?;
		write!(rustfmt.stdin.take().unwrap(), "{}", s)?;
		let output = rustfmt.wait_with_output()?;
		let stdout = String::from_utf8(output.stdout)?;
		let preamble = "Generated file, do not edit by hand, see `pi-syntax/src/generate.rs`";
		Ok(format!("//! {}\n\n{}", preamble, stdout))
	}

	fn build_imports(&mut self) {
		let nodes_imports = quote! {
			use crate::{
				syntax_kind::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken},
				S,
			};

			use super::tokens::{IdentExpr, IntExpr};
		};
		let tokens_imports = quote! {
			use crate::{
				syntax_kind::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken},
				S,
			};

			use super::ast::AstNode;
		};
		self.nodes_program += &nodes_imports.to_string();
		self.tokens_program += &tokens_imports.to_string();
	}

	fn build_ast_node_trait(&mut self) {
		let ast_node_trait = quote! {
			pub trait AstNode {
				fn cast(syntax: SyntaxNode) -> Option<Self>
				where
					Self: Sized;
				fn syntax(&self) -> &SyntaxNode;
			}
		};
		self.nodes_program += &ast_node_trait.to_string();
	}

	fn build_root(&mut self) {
		let root = quote! {#[derive(Debug)]
			pub struct Root {
				syntax: SyntaxNode,
			}

			impl AstNode for Root {
				fn cast(syntax: SyntaxNode) -> Option<Self> {
					match syntax.kind() {
						SyntaxKind::Root => Some(Self { syntax }),
						_ => None,
					}
				}

				fn syntax(&self) -> &SyntaxNode {
					&self.syntax
				}
			}

			impl Root {
				pub fn functions(&self) -> impl Iterator<Item = FnDecl> {
					self.syntax.children().filter_map(FnDecl::cast)
				}
			}
		};
		self.nodes_program += &root.to_string();
	}

	fn build_token(&mut self, node: Node) {
		let node = &self.grammar[node];
		let mut method_name_opt = None;
		let (struct_name, method_name) = if let Rule::Labeled { rule, label } = &node.rule {
			method_name_opt = Some(label);
			if let Rule::Token(token) = **rule {
				let token = &self.grammar[token];
				let struct_name = node.name.to_string();
				let method_name = token.name.to_string();
				(Some(struct_name), Some(method_name))
			} else {
				(None, None)
			}
		} else if let Rule::Token(token) = node.rule {
			let token = &self.grammar[token];
			let struct_name = node.name.to_string();
			let method_name = token.name.to_string();
			(Some(struct_name), Some(method_name))
		} else {
			(None, None)
		};

		let struct_name = format_ident!("{}", struct_name.unwrap());
		let method_name_str = method_name.unwrap();
		let mut method_name_ident = format_ident!("{}", method_name_str);
		if method_name_opt.is_some() {
			method_name_ident = format_ident!("{}", method_name_opt.unwrap());
		}

		let get_method = quote! {
			pub fn #method_name_ident(&self) -> Option<SyntaxToken> {
				self
					.syntax
					.children_with_tokens()
					.filter_map(SyntaxElement::into_token)
					.find(|token| token.kind() == S!(#method_name_str))
			}
		};

		let program = quote! {
			#[derive(Debug)]
			pub struct #struct_name {
				pub(crate) syntax: SyntaxNode,
			}

			impl AstNode for #struct_name {
				fn cast(syntax: SyntaxNode) -> Option<Self> {
					match syntax.kind() {
						SyntaxKind::#struct_name => Some(Self { syntax }),
						_ => None,
					}
				}

				fn syntax(&self) -> &SyntaxNode {
					&self.syntax
				}
			}

			impl #struct_name {
				#get_method
			}
		};

		self.tokens_program += &program.to_string();
	}

	fn build_enum(&mut self, node: Node) {
		let node = &self.grammar[node];
		let enum_name = format_ident!("{}", node.name);

		let mut fields = vec![];
		let mut names = vec![];
		if let Rule::Alt(rule) = &node.rule {
			for rule in rule {
				if let Rule::Node(node) = rule {
					let node = &self.grammar[*node];
					let field_name = format_ident!("{}", node.name);
					names.push(quote! {
						#field_name
					});
					fields.push(quote! {
						#field_name(#field_name),
					});
				} else if let Rule::Labeled { rule, .. } = rule {
					if let Rule::Node(node) = **rule {
						let node = &self.grammar[node];
						let field_name = format_ident!("{}", node.name);
						names.push(quote! {
							#field_name
						});
						fields.push(quote! {
							#field_name(#field_name),
						});
					}
				}
			}
		}

		let enum_decl = quote! {
			#[derive(Debug)]
			pub enum #enum_name {
				#(#fields)*
			}
		};

		let ast_node_impl = quote! {
			impl AstNode for #enum_name {
				fn cast(syntax: SyntaxNode) -> Option<Self> {
					let result = match syntax.kind() {
						#(SyntaxKind::#names => Self::#names(#names { syntax }),)*
						_ => return None,
					};
					Some(result)
				}

				fn syntax(&self) -> &SyntaxNode {
					match self {
						#(#enum_name::#names(node) => &node.syntax,)*
					}
				}
			}
		};

		let enum_full = quote! {
			#enum_decl
			#ast_node_impl
		};

		self.nodes_program += &enum_full.to_string();
	}

	fn build_struct(&mut self, node: Node) {
		let node = &self.grammar[node];

		let struct_name = format_ident!("{}", node.name);

		let struct_decl = quote! {
			#[derive(Debug)]
			pub struct #struct_name {
				syntax: SyntaxNode,
			}
		};

		let ast_node_impl = quote! {
			impl AstNode for #struct_name {
				fn cast(syntax: SyntaxNode) -> Option<Self> {
					match syntax.kind() {
						SyntaxKind::#struct_name => Some(Self { syntax }),
						_ => None,
					}
				}

				fn syntax(&self) -> &SyntaxNode {
					&self.syntax
				}
			}
		};

		let mut getters = vec![];
		match &node.rule {
			Rule::Seq(seq) => {
				for node in seq {
					if let Rule::Labeled { label, rule } = node {
						let name = label;
						let getter_name = format_ident!("{}", name);
						match &**rule {
							Rule::Node(node) => {
								let node = &self.grammar[*node];
								let ret_ty = format_ident!("{}", node.name);
								let getter = quote! {
									pub fn #getter_name(&self) -> Option<#ret_ty> {
										self.syntax.children().find_map(#ret_ty::cast)
									}
								};
								getters.push(getter);
							}
							Rule::Token(tok) => {
								let tok = &self.grammar[*tok];
								let token_name = tok.name.to_string();
								let getter = quote! {
									pub fn #getter_name(&self) -> Option<SyntaxToken> {
										self
											.syntax
											.children_with_tokens()
											.filter_map(SyntaxElement::into_token)
											.find(|token| token.kind() == S!(#token_name))
									}
								};
								getters.push(getter);
							}
							Rule::Alt(alt) => {
								let mut toks = vec![];
								for tok in alt {
									if let Rule::Token(tok) = tok {
										let ident = self.grammar[*tok].name.to_string();
										toks.push(quote! {
											S!(#ident)
										});
									}
								}
								let getter = quote! {
									pub fn #getter_name(&self) -> Option<SyntaxToken> {
										self
											.syntax
											.children_with_tokens()
											.filter_map(SyntaxElement::into_token)
											.find(|token| {
												matches!(
													token.kind(),
													#(#toks)|*,
												)
											})
									}
								};
								getters.push(getter);
							}
							Rule::Opt(opt) => {
								if let Rule::Seq(seq) = &**opt {
									for rule in seq {
										if let Rule::Node(node) = rule {
											let node = &self.grammar[*node];
											let ret_ty = format_ident!("{}", node.name);
											let getter = quote! {
												pub fn #getter_name(&self) -> Option<#ret_ty> {{
													self.syntax.children().find_map(#ret_ty::cast)
												}}
											};
											getters.push(getter);
										}
									}
								}
							}
							Rule::Rep(rep) => {
								if let Rule::Node(node) = &**rep {
									let node = &self.grammar[*node];
									let ret_ty = format_ident!("{}", node.name);
									let getter = quote! {
										pub fn #getter_name(&self) -> Vec<#ret_ty> {{
											self.syntax.children().filter_map(#ret_ty::cast).collect()
										}}
									};
									getters.push(getter);
								}
							}
							_ => {}
						};
					} else if let Rule::Node(_) = node {
						panic!("oops")
					}
				}
			}
			_ => panic!(),
		};

		let node_impl = quote! {
			#struct_decl

			#ast_node_impl

			impl #struct_name {
				#(#getters)*
			}
		};

		self.nodes_program += &node_impl.to_string();
	}
}
