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

use self::trait_implementors::TraitImplementorTable;

mod trait_implementors;

pub enum InferenceError {
	CouldNotInfer { ty_span: Span },
}

#[derive(Debug)]
pub struct TypeEnv {
	/// A map from type name to the traits it implements
	// pub implementations: HashMap<SmolStr, HashSet<SmolStr>>, // TODO: how can we store spans for trait bounds (error messages)
	// pub implementations: HashMap<(SmolStr, Vec<TypeId>), HashSet<SmolStr>>,
	// pub implementations: HashMap<SmolStr, Vec<(Vec<TypeId>, HashSet<SmolStr>)>>,

	// trait name -> ( impltor name -> ([trait_type_params], [impltor_type_params]) )
	pub trait_implementors: TraitImplementorTable,

	pub signatures: HashMap<SmolStr, TypeId>,
	pub enums: HashMap<SmolStr, HashMap<SmolStr, Option<TypeId>>>,
	pub vars: Vec<Spanned<TypeKind>>,
	pub var_ids: HashMap<SmolStr, TypeId>,
	pub return_type_id: TypeId,
}

impl TypeEnv {
	pub fn new() -> Self {
		Self {
			trait_implementors: TraitImplementorTable::new(),
			signatures: HashMap::new(),
			enums: HashMap::new(),
			vars: vec![],
			var_ids: HashMap::new(),
			return_type_id: 0,
		}
	}

	pub fn reset_symbol_table(&mut self) {
		// self.vars.clear(); // Because we're doing indexes, it might be safe to just not clear this. but this will probably become a huge vector so maybe we go back to hashmap? benchmark?
		self.var_ids.clear();
		self.return_type_id = 0;
	}

	pub fn insert(&mut self, ty: Spanned<TypeKind>) -> TypeId {
		let id = self.vars.len();
		self.vars.push(ty);
		id
	}

	#[inline]
	pub fn get_type(&self, id: TypeId) -> &Spanned<TypeKind> {
		self.vars.get(id).unwrap()
	}

	pub fn get_deref_type(&self, id: TypeId) -> &Spanned<TypeKind> {
		let mut true_id = id;
		while let TypeKind::Ref(id) = &self.get_type(true_id).inner {
			true_id = *id;
		}
		self.get_type(true_id)
	}

	#[inline]
	pub fn set_type(&mut self, id: TypeId, ty: Spanned<TypeKind>) {
		self.vars[id] = ty;
	}

	pub fn get_path_id(&self, path: &Vec<Spanned<SmolStr>>) -> Result<TypeId, TypeError> {
		let as_str = SmolStr::from(path.iter().map(|s| s.to_string()).join("::"));
		match self.var_ids.get(&as_str) {
			Some(id) => Ok(*id),
			None => match self.signatures.get(&as_str) {
				Some(id) => Ok(*id),
				None => Err(TypeError::UnknownPath {
					path: Spanned::new(as_str, Spanned::vec_span(&path).unwrap()),
				}),
			},
		}
	}

	#[inline]
	pub fn set_path_id(&mut self, path: &[SmolStr], id: TypeId) {
		self.var_ids.insert(SmolStr::from(path.join("::")), id);
	}

	pub fn set_trait_implementation(&mut self, type_name: (SmolStr, Vec<TypeId>), trt: SmolStr) {
		todo!()
		// let trts = self
		// 	.implementations
		// 	.entry(type_name)
		// 	.or_insert(HashSet::new());
		// trts.insert(trt);
	}

	// pub fn get_trait_implementations(
	// 	&self,
	// 	type_name: &SmolStr,
	// ) -> Option<&Vec<(Vec<TypeId>, HashSet<SmolStr>)>> {
	// 	self.implementations.get(type_name)
	// }

	pub fn reconstruct(&self, id: TypeId) -> Result<Spanned<TypeKind>, TypeError> {
		use TypeKind::*;
		let span = self.vars[id].span.clone();
		match &self.vars[id].inner {
			Ref(id) => self.reconstruct(*id),
			Int(id) => {
				if let Some(id) = id {
					self.reconstruct(*id)
				} else {
					Ok(Spanned {
						inner: TypeKind::Int(None),
						span,
					})
				}
			}
			Float(id) => {
				if let Some(id) = id {
					self.reconstruct(*id)
				} else {
					Ok(Spanned {
						inner: TypeKind::Float(None),
						span,
					})
				}
			}
			Concrete(t) => Ok(Spanned {
				inner: TypeKind::Concrete(t.clone()),
				span,
			}),
			_ => Err(TypeError::CouldNotInfer { ty_span: span }),
		}
	}

	pub fn fmt_ty(&self, ty: &TypeKind) -> String {
		match ty {
			TypeKind::Concrete(t) => self.fmt_concrete_ty(t),
			TypeKind::Ref(id) => format!("{}", self.fmt_ty(&self.get_type(*id).inner)),
			TypeKind::Int(id) => {
				if let Some(id) = id {
					format!("{}", self.fmt_ty(&self.get_type(*id).inner))
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
			ConcreteKind::Ptr(id) => format!("*{}", self.fmt_ty(&self.get_type(*id).inner)),
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
							.map(|id| self.fmt_ty(&self.get_type(*id).inner))
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
				self.fmt_ty(&self.get_type(*o).inner)
			),
		}
	}

	pub fn inner_type(&self, ty: &TypeKind) -> TypeKind {
		if let TypeKind::Concrete(concrete) = ty {
			if let ConcreteKind::Ptr(id) = concrete {
				return self.inner_type(&self.get_type(*id).inner);
			}
		}
		ty.clone()
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
