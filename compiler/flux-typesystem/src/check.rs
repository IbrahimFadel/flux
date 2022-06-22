use crate::infer::TypeEnv;

use super::*;

pub struct TypeChecker<E: ErrorHandler> {
	pub tenv: TypeEnv,
	err_handler: E,
}

impl<E: ErrorHandler> TypeChecker<E> {
	pub fn new(err_handler: E) -> Self {
		Self {
			tenv: TypeEnv::new(),
			err_handler,
		}
	}

	pub fn unify(&mut self, a: TypeId, b: TypeId, unification_span: Span) -> Result<(), E::Error> {
		use TypeKind::*;
		let akind = self.tenv.vars[&a].clone();
		let bkind = self.tenv.vars[&b].clone();
		match (&akind.inner, &bkind.inner) {
			(Ref(a), _) => self.unify(*a, b, unification_span),
			(_, Ref(b)) => self.unify(a, *b, unification_span),
			(Unknown, _) => {
				self.tenv.vars.insert(
					a,
					Spanned {
						inner: TypeKind::Ref(b),
						span: bkind.span,
					},
				);
				Ok(())
			}
			(_, Unknown) => {
				self.tenv.vars.insert(
					b,
					Spanned {
						inner: TypeKind::Ref(a),
						span: akind.span,
					},
				);
				Ok(())
			}
			(Concrete(aa), Concrete(bb)) => {
				if aa == bb {
					Ok(())
				} else {
					Err(
						self
							.err_handler
							.type_mismatch(&self.tenv, a, b, unification_span),
					)
				}
			}
			(Concrete(t), Int(id)) => {
				if let Some(id) = id {
					self.unify(a, *id, unification_span)
				} else {
					match t {
						ConcreteKind::SInt(_) | ConcreteKind::UInt(_) => (),
						_ => {
							return Err(
								self
									.err_handler
									.type_mismatch(&self.tenv, a, b, unification_span),
							)
						}
					}
					self.tenv.vars.insert(
						b,
						Spanned {
							inner: TypeKind::Int(Some(a)),
							span: akind.span,
						},
					);
					Ok(())
				}
			}
			(Int(id), Concrete(t)) => {
				if let Some(id) = id {
					self.unify(*id, a, unification_span)
				} else {
					match t {
						ConcreteKind::SInt(_) | ConcreteKind::UInt(_) => (),
						_ => {
							return Err(
								self
									.err_handler
									.type_mismatch(&self.tenv, a, b, unification_span),
							)
						}
					}
					self.tenv.vars.insert(
						a,
						Spanned {
							inner: TypeKind::Int(Some(b)),
							span: akind.span,
						},
					);
					Ok(())
				}
			}
			(Concrete(t), Float(id)) => {
				if let Some(id) = id {
					self.unify(a, *id, unification_span)
				} else {
					match t {
						ConcreteKind::F32 | ConcreteKind::F64 => (),
						_ => {
							return Err(
								self
									.err_handler
									.type_mismatch(&self.tenv, a, b, unification_span),
							)
						}
					}
					self.tenv.vars.insert(
						b,
						Spanned {
							inner: TypeKind::Float(Some(a)),
							span: akind.span,
						},
					);
					Ok(())
				}
			}
			(Float(id), Concrete(t)) => {
				if let Some(id) = id {
					self.unify(*id, a, unification_span)
				} else {
					match t {
						ConcreteKind::F32 | ConcreteKind::F64 => (),
						_ => {
							return Err(
								self
									.err_handler
									.type_mismatch(&self.tenv, a, b, unification_span),
							)
						}
					}
					self.tenv.vars.insert(
						a,
						Spanned {
							inner: TypeKind::Float(Some(b)),
							span: akind.span,
						},
					);
					Ok(())
				}
			}
			_ => Err(
				self
					.err_handler
					.type_mismatch(&self.tenv, a, b, unification_span),
			),
		}
	}
}

pub trait ErrorHandler {
	type Error;

	fn type_mismatch(&self, env: &TypeEnv, a: TypeId, b: TypeId, span: Span) -> Self::Error;
}
