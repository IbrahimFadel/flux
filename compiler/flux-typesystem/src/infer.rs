use std::{
	collections::{HashMap, HashSet},
	fmt,
};

use flux_span::{Span, Spanned};
use itertools::Itertools;
use smol_str::SmolStr;

use crate::{
	check::TypeError,
	r#type::{ConcreteKind, TypeId, TypeKind},
};

pub enum InferenceError {
	CouldNotInfer { ty_span: Span },
}

pub struct TypeEnv {
	/// A map from type name to the traits it implements
	pub implementations: HashMap<SmolStr, HashSet<SmolStr>>, // TODO: how can we store spans for trait bounds (error messages)
	pub vars: Vec<Spanned<TypeKind>>,
	pub var_ids: HashMap<SmolStr, TypeId>,
	pub return_type_id: TypeId,
}

impl TypeEnv {
	pub fn new(implementations: HashMap<SmolStr, HashSet<SmolStr>>) -> Self {
		Self {
			implementations,
			vars: vec![],
			var_ids: HashMap::new(),
			return_type_id: 0,
		}
	}

	pub fn insert(&mut self, ty: Spanned<TypeKind>) -> TypeId {
		let id = self.vars.len();
		self.vars.push(ty);
		id
	}

	pub fn get_type(&self, id: TypeId) -> Spanned<TypeKind> {
		self.vars.get(id).cloned().unwrap()
	}

	pub fn set_type(&mut self, id: TypeId, ty: Spanned<TypeKind>) {
		self.vars[id] = ty;
	}

	pub fn get_path_id(&self, path: &Vec<Spanned<SmolStr>>) -> Result<TypeId, TypeError> {
		let as_str = SmolStr::from(path.iter().map(|s| s.to_string()).join("::"));
		match self.var_ids.get(&as_str) {
			Some(id) => Ok(*id),
			None => Err(TypeError::UnknownPath {
				path: Spanned::new(as_str, Spanned::vec_span(&path).unwrap()),
			}),
		}
	}

	pub fn set_path_id(&mut self, path: &[SmolStr], id: TypeId) {
		self.var_ids.insert(SmolStr::from(path.join("::")), id);
	}

	pub fn set_trait_implementation(&mut self, type_name: SmolStr, trt: SmolStr) {
		let trts = self
			.implementations
			.entry(type_name)
			.or_insert(HashSet::new());
		trts.insert(trt);
	}

	pub fn get_trait_implementations(&self, type_name: &SmolStr) -> Option<&HashSet<SmolStr>> {
		self.implementations.get(type_name)
	}

	pub fn reconstruct(&self, id: TypeId) -> Result<Spanned<TypeKind>, InferenceError> {
		todo!()
		// use TypeKind::*;
		// let span = self.vars[&id].span.clone();
		// match &self.vars[&id].inner {
		// 	Unknown => Err(InferenceError::CouldNotInfer { ty_span: span }),
		// 	Ref(id) => self.reconstruct(*id),
		// 	Int(id) => {
		// 		if let Some(id) = id {
		// 			self.reconstruct(*id)
		// 		} else {
		// 			Ok(Spanned {
		// 				inner: TypeKind::Int(None),
		// 				span,
		// 			})
		// 		}
		// 	}
		// 	Float(id) => {
		// 		if let Some(id) = id {
		// 			self.reconstruct(*id)
		// 		} else {
		// 			Ok(Spanned {
		// 				inner: TypeKind::Float(None),
		// 				span,
		// 			})
		// 		}
		// 	}
		// 	Concrete(t) => Ok(Spanned {
		// 		inner: TypeKind::Concrete(t.clone()),
		// 		span,
		// 	}),
		// }
	}

	pub fn fmt_ty(&self, ty: &TypeKind) -> String {
		match ty {
			TypeKind::Concrete(t) => self.fmt_concrete_ty(t),
			TypeKind::Ref(id) => format!("{}", self.fmt_ty(&self.get_type(*id))),
			TypeKind::Int(id) => {
				if let Some(id) = id {
					format!("{}", self.fmt_ty(&self.get_type(*id)))
				} else {
					format!("Int")
				}
			}
			TypeKind::Float(id) => {
				if let Some(id) = id {
					format!("Float({})", id)
				} else {
					format!("Float")
				}
			}
			TypeKind::Generic((name, restrictions)) => format!(
				"{}{}",
				name,
				if restrictions.len() == 0 {
					format!("")
				} else {
					format!(": {}", restrictions.iter().join(", "))
				}
			),
			TypeKind::Unknown => format!("Unknown"),
		}
	}

	pub fn fmt_concrete_ty(&self, ty: &ConcreteKind) -> String {
		match ty {
			ConcreteKind::SInt(n) => format!("i{}", n),
			ConcreteKind::UInt(n) => format!("u{}", n),
			ConcreteKind::F64 => format!("f64"),
			ConcreteKind::F32 => format!("f32"),
			ConcreteKind::Ptr(id) => format!("*{}", self.fmt_ty(&self.get_type(*id))),
			ConcreteKind::Ident((name, type_params)) => format!(
				"{}{}",
				name,
				if type_params.len() == 0 {
					format!("")
				} else {
					format!(
						"<{}>",
						type_params
							.iter()
							.map(|id| self.fmt_ty(&self.get_type(*id)))
							.join(", ")
					)
				}
			),
			ConcreteKind::Tuple(types) => format!(
				"({})",
				types
					.iter()
					.map(|id| self.fmt_ty(&self.get_type(*id).inner))
					.reduce(|s, ty| format!("{}, {}", s, ty))
					.map_or(String::new(), |s| s)
			),
			ConcreteKind::Func(i, o) => format!(
				"{} -> {}",
				if let Some(s) = i
					.iter()
					.map(|ty| self.fmt_ty(&self.get_type(*ty).inner))
					.reduce(|s, ty| format!("{}, {}", s, ty))
				{
					s
				} else {
					String::from("()")
				},
				self.fmt_ty(&self.get_type(**o))
			),
		}
	}
}

impl fmt::Display for TypeEnv {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = String::new();
		for var in &self.var_ids {
			s += &format!("{} -> {:?}\n", var.0, var.1);
		}
		s += &format!("\n-------------\n");
		// for var in &self.vars {
		// 	s += &format!("{} -> {}\n", var.0, self.fmt_ty(&var.1.inner));
		// }
		write!(f, "{}", s)
	}
}
