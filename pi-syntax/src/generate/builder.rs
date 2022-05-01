use std::{fs, io};

use ungrammar::{Grammar, Node, Rule};

pub struct Builder<'a> {
	grammar: &'a Grammar,
	program: String,
}

impl<'a> Builder<'a> {
	pub fn new(grammar: &'a Grammar) -> Self {
		Self {
			grammar,
			program: String::with_capacity(5000), // We know this string will be quite long, so this should speed things up a bit
		}
	}

	pub fn generate_ast(&mut self) {
		self.program += "// automatically generated. do not edit manually\n\n";
		self.build_imports();
		self.build_ast_node_trait();
		self.build_root();

		for node in self.grammar.iter() {
			if let Rule::Alt(_) = self.grammar[node].rule {
				self.build_enum(node);
			} else {
				self.build_struct(node);
			}
		}
	}

	pub fn write_ast_to_file(&self) -> io::Result<()> {
		fs::write(
			"./pi-syntax/src/generate/generated_ast.rs",
			self.program.as_str(),
		)
	}

	fn build_imports(&mut self) {
		self.program += r#"use crate::{
	syntax_kind::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken},
	S,
};

"#;
	}

	fn build_ast_node_trait(&mut self) {
		self.program += r#"pub trait AstNode {
	fn cast(syntax: SyntaxNode) -> Option<Self>
	where
		Self: Sized;
	fn syntax(&self) -> &SyntaxNode;
}

"#;
	}

	fn build_root(&mut self) {
		self.program += r#"#[derive(Debug)]
pub struct Root(SyntaxNode);
		
impl AstNode for Root {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::Root => Some(Self(syntax)),
			_ => None,
		}
	}
		
	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}
}

impl Root {
	pub fn functions(&self) -> impl Iterator<Item = FnDecl> {
		self.0.children().filter_map(FnDecl::cast)
	}
}

"#;
	}

	fn build_enum(&mut self, node: Node) {
		let node = &self.grammar[node];
		self.program += &format!("#[derive(Debug)]\npub enum {} {{\n", node.name);

		let mut names = vec![];
		if let Rule::Alt(rule) = &node.rule {
			for rule in rule {
				if let Rule::Node(node) = rule {
					let node = &self.grammar[*node];
					names.push(node.name.as_str());
					self.program += &format!("\t{}({}),\n", node.name, node.name);
				} else if let Rule::Labeled { rule, .. } = rule {
					if let Rule::Node(node) = &**rule {
						let node = &self.grammar[*node];
						names.push(node.name.as_str());
						self.program += &format!("\t{}({}),\n", node.name, node.name);
					}
				}
			}
		}
		self.program += "}\n\n";

		self.program += &format!("impl AstNode for {} {{\n", node.name);
		self.program += r#"	fn cast(syntax: SyntaxNode) -> Option<Self> {
		let result = match syntax.kind() {
"#;
		for name in &names {
			self.program += &format!(
				"\t\t\tSyntaxKind::{} => Self::{}({}(syntax)),\n",
				name, name, name
			);
		}
		self.program += "\t\t\t_ => return None,";
		self.program += "\n\t\t};\n\t\treturn Some(result);\n\t}\n\n";

		self.program += &format!(
			r#"	fn syntax(&self) -> &SyntaxNode {{
		match self {{
"#
		);
		for name in &names {
			self.program += &format!("\t\t\t{}::{}(node) => &node.0,\n", node.name, name);
		}
		self.program += "\t\t}\n\t}\n";
		self.program += "}\n\n";
	}

	fn build_struct(&mut self, node: Node) {
		let node = &self.grammar[node];
		self.program += &format!(
			"#[derive(Debug)]\npub struct {}(SyntaxNode);\n\n",
			node.name
		);

		self.program += &format!(
			r#"impl AstNode for {} {{
	fn cast(syntax: SyntaxNode) -> Option<Self> {{
		match syntax.kind() {{
			SyntaxKind::{} => Some(Self(syntax)),
			_ => None,
		}}
	}}

	fn syntax(&self) -> &SyntaxNode {{
		&self.0
	}}
}}

"#,
			node.name, node.name
		);

		self.program += &format!("impl {} {{\n", node.name);
		if let Rule::Seq(seq) = &node.rule {
			for node in seq {
				if let Rule::Labeled { label, rule } = node {
					let name = label;
					if let Rule::Node(node) = &**rule {
						let node = &self.grammar[*node];
						self.program += &format!(
							r#"	pub fn {}(&self) -> Option<{}> {{
		self.0.children().find_map({}::cast)
	}}
"#,
							name, node.name, node.name
						);
					} else if let Rule::Token(tok) = &**rule {
						let tok = &self.grammar[*tok];
						self.program += &format!(
							r#"	pub fn {}(&self) -> Option<SyntaxToken> {{
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| token.kind() == S!({}))
	}}
"#,
							name, tok.name
						);
					} else if let Rule::Alt(alt) = &**rule {
						let mut toks: Vec<String> = vec![];
						for tok in alt {
							if let Rule::Token(tok) = tok {
								toks.push(format!("S!({})", self.grammar[*tok].name));
							}
						}
						self.program += &format!(
							r#"	pub fn {}(&self) -> Option<SyntaxToken> {{
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| {{
				matches!(
					token.kind(),
					{},
				)
			}})
	}}
"#,
							name,
							toks.join(" | ")
						);
					} else if let Rule::Opt(opt) = &**rule {
						if let Rule::Seq(seq) = &**opt {
							for rule in seq {
								if let Rule::Node(node) = rule {
									let node = &self.grammar[*node];
									self.program += &format!(
										r#"	pub fn {}(&self) -> Option<{}> {{
		self.0.children().find_map({}::cast)
	}}
"#,
										name, node.name, node.name
									);
								}
							}
						}
					} else if let Rule::Rep(rep) = &**rule {
						if let Rule::Node(node) = &**rep {
							let node = &self.grammar[*node];
							self.program += &format!(
								r#"	pub fn {}(&self) -> Vec<{}> {{
		self.0.children().filter_map({}::cast).collect()
	}}
"#,
								name, node.name, node.name
							);
						}
					}
				} else if let Rule::Node(node) = node {
					let node = &self.grammar[*node];
					self.program += &format!("\tpub fn {}(&self) {{\n\t}}\n", node.name);
				}
			}
		} else if let Rule::Labeled { label, rule } = &node.rule {
			if let Rule::Token(tok) = &**rule {
				let tok = &self.grammar[*tok];
				self.program += &format!(
					r#"	pub fn {}(&self) -> Option<SyntaxToken> {{
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| token.kind() == S!({}))
	}}
"#,
					label, tok.name
				);
			}
		}
		self.program += &format!("}}\n\n");
	}
}
