use std::{
	fmt,
	fmt::{Display, Formatter},
};

use itertools::Itertools;

use crate::hir::{Block, FnDecl, FnParam, Stmt, Type, Visibility};

// impl Display for Type {
// 	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
// 		match self {
// 			Type::SInt(n) => write!(f, "i{}", *n),
// 			Type::UInt(n) => write!(f, "u{}", *n),
// 			Type::F32 => write!(f, "f32"),
// 			Type::F64 => write!(f, "f64"),
// 			Type::Ident((name, type_params)) => write!(
// 				f,
// 				"{}{}",
// 				name,
// 				if type_params.len() == 0 {
// 					format!("")
// 				} else {
// 					format!("<{}>", type_params.join(", "))
// 				}
// 			),
// 			Type::Tuple(types) => write!(
// 				f,
// 				"({})",
// 				types.iter().map(|ty| format!("{}", ty)).join(", ")
// 			),
// 			_ => write!(f, "{:?}", self),
// 		}
// 	}
// }

// impl Display for Block {
// 	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
// 		write!(
// 			f,
// 			"{{\n{}\n}}",
// 			self
// 				.0
// 				.iter()
// 				.map(|stmt| format!("{}", stmt.inner))
// 				.collect::<Vec<_>>()
// 				.join("\n")
// 		)
// 	}
// }

// impl Display for Stmt {
// 	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
// 		match self {
// 			Stmt::VarDecl(var) => write!(f, "{} {}", var.ty.inner, var.name),
// 			_ => write!(f, ""),
// 		}
// 	}
// }

// impl fmt::Display for FnDecl {
// 	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
// 		write!(
// 			f,
// 			"{}fn {}({}) -> {}",
// 			if self.visibility.inner == Visibility::Public {
// 				"pub "
// 			} else {
// 				""
// 			},
// 			self.name.inner,
// 			self
// 				.params
// 				.inner
// 				.iter()
// 				.map(|p| format!("{}", p.inner))
// 				.collect::<Vec<_>>()
// 				.join(", "),
// 			self.return_type.inner,
// 		)
// 	}
// }

// impl Display for FnParam {
// 	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
// 		write!(
// 			f,
// 			"{}{} {}",
// 			if self.mutable { "mut " } else { "" },
// 			self.ty.inner,
// 			if let Some(name) = &self.name {
// 				name
// 			} else {
// 				"<unknown>"
// 			}
// 		)
// 	}
// }
