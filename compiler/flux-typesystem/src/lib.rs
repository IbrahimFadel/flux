use std::{collections::HashMap, fmt};

use flux_error::FluxError;
use smol_str::SmolStr;

pub type TypeId = usize;

pub trait TypeData: Clone + PartialEq + fmt::Display {}

pub struct Type<D: TypeData>(pub TypeKind<D>);

#[derive(Debug, Clone)]
pub enum TypeKind<D: TypeData> {
	Concrete(D),
	Int(Option<TypeId>),
	Float(Option<TypeId>),
	Ref(TypeId),
	Unknown,
}

impl<D: TypeData> fmt::Display for TypeKind<D> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Concrete(t) => write!(f, "{}", t),
			Self::Ref(id) => write!(f, "Ref({})", id),
			Self::Int(id) => if let Some(id) = id {
				write!(f, "Int({})", id)
			} else {
				write!(f, "Int")
			}
			Self::Float(id) => if let Some(id) = id {
				write!(f, "Float({})", id)
			} else {
				write!(f, "Float")
			}
			Self::Unknown => write!(f, "Unknown"),
		}
	}
}

pub struct TypeEnv<D: TypeData> {
	id_counter: usize,
	vars: HashMap<TypeId, TypeKind<D>>,
	var_ids: HashMap<SmolStr, TypeId>,
}

impl<D: TypeData> Default for TypeEnv<D> {
	fn default() -> Self {
		Self {
			id_counter: 0,
			vars: HashMap::new(),
			var_ids: HashMap::new(),
		}
	}
}

impl<D: TypeData> TypeEnv<D> {
	pub fn insert(&mut self, info: TypeKind<D>) -> TypeId {
		let id = self.id_counter;
		self.id_counter += 1;
		self.vars.insert(id, info);
		id
	}

	pub fn get_type(&self, id: TypeId) -> TypeKind<D> {
		self.vars[&id].clone()
	}

	pub fn get_path_id(&self, path: &[SmolStr]) -> TypeId {
		self.var_ids[&SmolStr::from(path.join("::"))]
	}

	pub fn set_path_id(&mut self, path: &[SmolStr], id: TypeId) {
		self.var_ids.insert(SmolStr::from(path.join("::")), id);
	}

	pub fn unify(&mut self, a: TypeId, b: TypeId) -> Result<(), FluxError> {
		use TypeKind::*;
		match (self.vars[&a].clone(), self.vars[&b].clone()) {
			(Ref(a), _) => self.unify(a, b),
			(_, Ref(b)) => self.unify(a, b),
			(Unknown, _) => {
				self.vars.insert(a, TypeKind::Ref(b));
				Ok(())
			}
			(_, Unknown) => {
				self.vars.insert(b, TypeKind::Ref(a));
				Ok(())
			}
			(Concrete(a), Concrete(b)) => {
				if a == b {
					Ok(())
				} else {
					Err(FluxError::default().with_msg(format!("type mismatch between {} and {}", a, b)))
				}
			}
			(Concrete(_), Int(id)) => {
				if let Some(id) = id {
					self.unify(a, id)
				}  else {
					self.vars.insert(b, TypeKind::Int(Some(a)));
					Ok(())
				}
			}
			(a, b) => {
				Err(FluxError::default().with_msg(format!("type mismatch between {} and {}", a, b)))
			}
		}
	}

	pub fn reconstruct(&self, id: TypeId) -> Result<Type<D>, FluxError> {
		use TypeKind::*;
		match &self.vars[&id] {
			Unknown => Err(FluxError::default().with_msg(format!("cannot infer"))),
			Ref(id) => self.reconstruct(*id),
			Int(id) => {
				if let Some(id) = id {
					self.reconstruct(*id)
				} else {
					// Ok(Type::Int)
					Ok(Type(TypeKind::Int(None)))
				}
			}
			Float(id) => {
				if let Some(id) = id {
					self.reconstruct(*id)
				} else {
					// Ok(Type::Float)
					Ok(Type(TypeKind::Float(None)))
				}
			}
			Concrete(t) => {
				Ok(Type(TypeKind::Concrete(t.clone())))
			}
			
			// SInt(n) => Ok(Type::SInt(n)),
			// UInt(n) => Ok(Type::UInt(n)),
			// Float => Ok(Type::F32),
			// F32 => Ok(Type::F32),
			// F64 => Ok(Type::F64),
			// Bool => Ok(Type::Bool),
			// Unit => Ok(Type::Unit),
			// List(item) => Ok(Type::List(Box::new(self.reconstruct(item)?))),
			// Func(i, o) => Ok(Type::Func(
				// Box::new(self.reconstruct(i)?),
				// Box::new(self.reconstruct(o)?),
			// )),
		}
	}
}

impl<D: TypeData> fmt::Display for TypeEnv<D> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = String::new();
		for var in &self.var_ids {
			s += &format!("{} -> {:?}\n", var.0, var.1);
		}
		s += &format!("\n-------------\n");
		for var in &self.vars {
			s += &format!("{} -> {}\n", var.0, var.1);
		}
		write!(f, "{}", s)
	}
}
