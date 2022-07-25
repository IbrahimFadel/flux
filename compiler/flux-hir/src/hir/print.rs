use std::fmt::Display;

use itertools::Itertools;

use crate::HirModule;

use super::{FnDecl, FnParam, FnParams, TraitDecl};

impl Display for HirModule {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut s = String::from("Traits\n------\n");
		for trt in &self.traits {
			s += &format!("{trt}");
		}
		s += "Functions\n---------\n";
		for f in &self.functions {
			s += &format!("{f}");
		}
		write!(f, "{s}")
	}
}

impl Display for TraitDecl {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.name.inner)
	}
}

impl Display for FnDecl {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}{} -> {:?}",
			self.name.inner, self.params.inner, self.return_type.inner
		)
	}
}

impl Display for FnParams {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"({})",
			self
				.0
				.iter()
				.map(|param| format!("{}", param.inner))
				.join(", ")
		)
	}
}

impl Display for FnParam {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}{:?} {}",
			if self.mutable {
				format!("mut ")
			} else {
				format!("")
			},
			self.ty.inner,
			self.name
		)
	}
}
