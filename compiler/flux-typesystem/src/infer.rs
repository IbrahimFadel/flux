use flux_error::FluxErrorCode;

use super::*;

pub struct TypeEnv {
	pub id_counter: usize,
	pub vars: HashMap<TypeId, Spanned<TypeKind>>,
	pub var_ids: HashMap<SmolStr, TypeId>,
	pub return_type_id: TypeId,
}

impl TypeEnv {
	pub fn new() -> Self {
		Self {
			id_counter: 0,
			vars: HashMap::new(),
			var_ids: HashMap::new(),
			return_type_id: 0,
		}
	}

	pub fn get_type(&self, id: TypeId) -> Spanned<TypeKind> {
		self.vars[&id].clone()
	}

	pub fn get_path_id(&self, path: &[SmolStr]) -> TypeId {
		self.var_ids[&SmolStr::from(path.join("::"))]
	}

	pub fn set_path_id(&mut self, path: &[SmolStr], id: TypeId) {
		self.var_ids.insert(SmolStr::from(path.join("::")), id);
	}

	pub fn reconstruct(&self, id: TypeId) -> Result<Spanned<TypeKind>, FluxError> {
		use TypeKind::*;
		let span = self.vars[&id].span.clone();
		match &self.vars[&id].inner {
			Unknown => Err(FluxError::build(
				format!("could not infer type"),
				span.clone(),
				FluxErrorCode::CouldNotInferType,
				(format!("could not infer type"), span.clone()),
			)),
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
		}
	}

	pub fn fmt_ty(&self, ty: &TypeKind) -> String {
		match ty {
			TypeKind::Concrete(t) => match t {
				ConcreteKind::SInt(n) => format!("i{}", n),
				ConcreteKind::UInt(n) => format!("u{}", n),
				ConcreteKind::F64 => format!("f64"),
				ConcreteKind::F32 => format!("f32"),
				ConcreteKind::Ident(name) => format!("{}", name),
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
						.map(|ty| self.fmt_ty(&ty))
						.reduce(|s, ty| format!("{}, {}", s, ty))
					{
						s
					} else {
						String::from("()")
					},
					self.fmt_ty(o)
				),
			},
			TypeKind::Ref(id) => format!("Ref({})", id),
			TypeKind::Int(id) => {
				if let Some(id) = id {
					format!("Int({})", id)
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
			TypeKind::Unknown => format!("Unknown"),
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
		for var in &self.vars {
			s += &format!("{} -> {}\n", var.0, self.fmt_ty(&var.1.inner));
		}
		write!(f, "{}", s)
	}
}
