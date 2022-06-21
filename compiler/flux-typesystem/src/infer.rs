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
			Unknown => Err(FluxError::default().with_msg(format!("cannot infer"))),
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
}

impl fmt::Display for TypeEnv {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = String::new();
		for var in &self.var_ids {
			s += &format!("{} -> {:?}\n", var.0, var.1);
		}
		s += &format!("\n-------------\n");
		for var in &self.vars {
			s += &format!("{} -> {}\n", var.0, var.1.inner);
		}
		write!(f, "{}", s)
	}
}
