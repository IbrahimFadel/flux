use std::{
	fmt,
	fmt::{Display, Formatter},
};

use crate::{Block, FnDecl, FnParam, Stmt, Type};

impl Display for Type {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Type::SInt(n) => write!(f, "i{}", *n),
			Type::UInt(n) => write!(f, "u{}", *n),
			Type::Void => write!(f, "void"),
			Type::Ident(name) => write!(f, "{}", name),
			_ => write!(f, "{:?}", self),
		}
	}
}

impl Display for Block {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{{\n{}\n}}",
			self
				.0
				.iter()
				.map(|stmt| if let Some(stmt) = &stmt {
					format!("{}", stmt.node)
				} else {
					"<empty>".to_string()
				})
				.collect::<Vec<_>>()
				.join("\n")
		)
	}
}

impl Display for Stmt {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Stmt::VarDecl(var) => write!(f, "{} {}", var.ty.node, var.name),
			Stmt::If(if_) => write!(f, "if <expr> {}", if_.then),
			_ => write!(f, ""),
		}
	}
}

impl fmt::Display for FnDecl {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}fn {}({}) -> {} {}",
			if self.public.node { "pub " } else { "" },
			if let Some(name) = &self.name {
				name.node.as_str()
			} else {
				"<unknown>"
			},
			self
				.params
				.node
				.iter()
				.map(|p| format!("{}", p.node))
				.collect::<Vec<_>>()
				.join(", "),
			self.return_type.node,
			self.block
		)
	}
}

impl Display for FnParam {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}{} {}",
			if self.mutable { "mut " } else { "" },
			self.ty.node,
			if let Some(name) = &self.name {
				name
			} else {
				"<unknown>"
			}
		)
	}
}
