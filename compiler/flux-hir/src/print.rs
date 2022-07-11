use std::{
	fmt,
	fmt::{Display, Formatter},
};

use crate::hir::{Block, FnDecl, FnParam, Stmt, Type, Visibility};

impl Display for Type {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Type::SInt(n) => write!(f, "i{}", *n),
			Type::UInt(n) => write!(f, "u{}", *n),
			Type::F32 => write!(f, "f32"),
			Type::F64 => write!(f, "f64"),
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
				.map(|stmt| format!("{}", stmt.node))
				.collect::<Vec<_>>()
				.join("\n")
		)
	}
}

impl Display for Stmt {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Stmt::VarDecl(var) => write!(f, "{} {}", var.ty.node, var.name),
			_ => write!(f, ""),
		}
	}
}

impl fmt::Display for FnDecl {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}fn {}({}) -> {}",
			if self.visibility.node == Visibility::Public {
				"pub "
			} else {
				""
			},
			self.name.node,
			self
				.params
				.node
				.iter()
				.map(|p| format!("{}", p.node))
				.collect::<Vec<_>>()
				.join(", "),
			self.return_type.node,
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
