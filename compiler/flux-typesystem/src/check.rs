use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

use ariadne::{Color, Label, Report, ReportKind};
use flux_error::{comma_separated_end_with_and, Error, FluxErrorCode};
use flux_span::{Span, Spanned};
use smol_str::SmolStr;

use crate::r#type::TypeKind;
use crate::{
	infer::TypeEnv,
	r#type::{ConcreteKind, TypeId},
};

pub struct TypeChecker {
	pub tenv: TypeEnv,
}

impl TypeChecker {
	pub fn new(
		implementations: HashMap<SmolStr, HashSet<SmolStr>>,
		signatures: HashMap<SmolStr, TypeId>,
	) -> Self {
		Self {
			tenv: TypeEnv::new(implementations, signatures),
		}
	}

	pub fn unify(&mut self, a: TypeId, b: TypeId, unification_span: Span) -> Result<(), TypeError> {
		use crate::r#type::TypeKind::*;
		let akind = self.tenv.vars[a].clone();
		let bkind = self.tenv.vars[b].clone();
		match (&akind.inner, &bkind.inner) {
			(Ref(a), _) => self.unify(*a, b, unification_span),
			(_, Ref(b)) => self.unify(a, *b, unification_span),
			(Unknown, _) => {
				self.tenv.set_type(
					a,
					Spanned {
						inner: TypeKind::Ref(b),
						span: bkind.span,
					},
				);
				Ok(())
			}
			(_, Unknown) => {
				self.tenv.set_type(
					b,
					Spanned {
						inner: TypeKind::Ref(a),
						span: akind.span,
					},
				);
				Ok(())
			}
			(Generic((a_name, restrictions)), Concrete(ConcreteKind::Ptr(id))) => {
				match &self.tenv.get_type(*id).inner {
					TypeKind::Generic((b_name, _)) => {
						if a_name == b_name {
							return Err(self.type_mismatch(a, b, unification_span));
						} else {
							Ok(())
						}
					}
					_ => {
						let inner_b = self.tenv.inner_type(&self.tenv.get_type(b));
						self.verify_trait_bounds(
							a,
							b,
							unification_span,
							&SmolStr::from(self.tenv.fmt_ty(&inner_b)),
							restrictions,
						)
					}
				}
			}
			(_, Generic((_, restrictions))) => {
				// Suppose T implements the trait Foo
				// *T, **T, ***T, etc. should also implement Foo without an explicit `apply`
				// So verify trait bounds on the type being pointed to
				let inner_a = self.tenv.inner_type(&self.tenv.get_type(a));
				self.verify_trait_bounds(
					a,
					b,
					unification_span,
					&SmolStr::from(self.tenv.fmt_ty(&inner_a)),
					restrictions,
				)
			}
			(Generic((_, restrictions)), _) => {
				let inner_b = self.tenv.inner_type(&self.tenv.get_type(b));
				self.verify_trait_bounds(
					a,
					b,
					unification_span,
					&SmolStr::from(self.tenv.fmt_ty(&inner_b)),
					restrictions,
				)
			}
			(Concrete(aa), Concrete(bb)) => match (aa, bb) {
				(ConcreteKind::Ident((a_name, a_params)), ConcreteKind::Ident((b_name, b_params))) => {
					if a_name == b_name {
						if a_params.len() != 0 && b_params.len() == 0 {
							Ok(())
						} else if b_params.len() != 0 && a_params.len() == 0 {
							Ok(())
						} else if a_params.len() != b_params.len() {
							Err(self.type_mismatch(a, b, unification_span))
						} else {
							a_params
								.iter()
								.zip(b_params)
								.try_for_each(|(a_param, b_param)| {
									self.unify(*a_param, *b_param, unification_span.clone())
								})
						}
					} else {
						Err(self.type_mismatch(a, b, unification_span))
					}
				}
				(ConcreteKind::Tuple(a_types), ConcreteKind::Tuple(b_types)) => {
					if a_types.len() != b_types.len() {
						Err(self.type_mismatch(a, b, unification_span))
					} else {
						for (i, a_ty) in a_types.iter().enumerate() {
							// println!("{:?} {}", self.tenv.get_type(*a_ty), i);
							self.unify(*a_ty, *b_types.get(i).unwrap(), unification_span.clone())?
						}
						Ok(())
					}
				}
				(ConcreteKind::Ptr(a_id), ConcreteKind::Ptr(b_id)) => {
					match (
						&self.tenv.get_type(*a_id).inner,
						&self.tenv.get_type(*b_id).inner,
					) {
						(_, TypeKind::Concrete(ConcreteKind::Tuple(tuple_types))) => {
							if tuple_types.len() == 0 {
								Ok(())
							} else {
								self.unify(*a_id, *b_id, unification_span)
							}
						}
						(TypeKind::Concrete(ConcreteKind::Tuple(tuple_types)), _) => {
							if tuple_types.len() == 0 {
								Ok(())
							} else {
								self.unify(*a_id, *b_id, unification_span)
							}
						}
						_ => self.unify(*a_id, *b_id, unification_span),
					}
				}
				_ => {
					if aa == bb {
						Ok(())
					} else {
						Err(self.type_mismatch(a, b, unification_span))
					}
				}
			},
			(Concrete(t), Int(id)) => {
				if let Some(id) = id {
					self.unify(a, *id, unification_span)
				} else {
					match t {
						ConcreteKind::SInt(_) | ConcreteKind::UInt(_) => (),
						_ => return Err(self.type_mismatch(a, b, unification_span)),
					}
					self.tenv.set_type(
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
						_ => return Err(self.type_mismatch(a, b, unification_span)),
					};
					self
						.tenv
						.set_type(a, Spanned::new(TypeKind::Int(Some(b)), akind.span));
					Ok(())
				}
			}
			(Concrete(t), Float(id)) => {
				if let Some(id) = id {
					self.unify(a, *id, unification_span)
				} else {
					match t {
						ConcreteKind::F32 | ConcreteKind::F64 => (),
						_ => return Err(self.type_mismatch(a, b, unification_span)),
					}
					self.tenv.set_type(
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
						_ => return Err(self.type_mismatch(a, b, unification_span)),
					}
					self.tenv.set_type(
						a,
						Spanned {
							inner: TypeKind::Float(Some(b)),
							span: akind.span,
						},
					);
					Ok(())
				}
			}
			(Int(aa), Int(bb)) => match (aa, bb) {
				(Some(aa), Some(bb)) => self.unify(*aa, *bb, unification_span),
				(Some(_), None) => {
					self.tenv.set_type(
						b,
						Spanned {
							inner: TypeKind::Int(Some(a)),
							span: akind.span,
						},
					);
					Ok(())
				}
				(None, Some(_)) => {
					self.tenv.set_type(
						a,
						Spanned {
							inner: TypeKind::Int(Some(b)),
							span: akind.span,
						},
					);
					Ok(())
				}
				(None, None) => Ok(()),
			},
			_ => Err(self.type_mismatch(a, b, unification_span)),
		}
	}

	fn verify_trait_bounds(
		&self,
		a: TypeId,
		b: TypeId,
		unification_span: Span,
		name: &SmolStr,
		restrictions: &HashSet<SmolStr>,
	) -> Result<(), TypeError> {
		if restrictions.len() == 0 {
			return Ok(());
		}
		if let Some(implementations) = self.tenv.get_trait_implementations(name) {
			let mut missing_implementations = HashSet::new();
			for restriction in restrictions {
				if implementations.get(restriction).is_none() {
					missing_implementations.insert(restriction.clone());
				}
			}

			if missing_implementations.len() == 0 {
				return Ok(());
			} else {
				return Err(self.trait_bounds_unsatisfied(a, b, unification_span, missing_implementations));
			}
		} else {
			return Err(self.trait_bounds_unsatisfied(a, b, unification_span, restrictions.clone()));
		}
	}

	fn type_mismatch(&self, a: TypeId, b: TypeId, span: Span) -> TypeError {
		let aa = self.tenv.get_type(a);
		let bb = self.tenv.get_type(b);
		TypeError::TypeMismatch {
			a: aa.map(|ty_kind| self.tenv.fmt_ty(&ty_kind)),
			b: bb.map(|ty_kind| self.tenv.fmt_ty(&ty_kind)),
			span,
		}
	}

	fn trait_bounds_unsatisfied(
		&self,
		ty: TypeId,
		generic: TypeId,
		span: Span,
		missing_implementations: HashSet<SmolStr>,
	) -> TypeError {
		let ty = self.tenv.get_type(ty);
		let generic = self.tenv.get_type(generic);
		TypeError::TraitBoundsUnsatisfied {
			ty: ty.map(|ty_kind| self.tenv.fmt_ty(&ty_kind)),
			generic: generic.map(|ty_kind| self.tenv.fmt_ty(&ty_kind)),
			span: span,
			missing_implementations,
		}
	}
}

#[derive(Debug)]
pub enum TypeError {
	TypeMismatch {
		/// Since we can't format TypeKind without a typeenv, we need to pass them after they've been formatted
		a: Spanned<String>,
		b: Spanned<String>,
		span: Span,
	},
	TraitBoundsUnsatisfied {
		ty: Spanned<String>,
		generic: Spanned<String>,
		span: Span,
		missing_implementations: HashSet<SmolStr>,
	},
	UnknownPath {
		path: Spanned<SmolStr>,
	},
	CouldNotInfer {
		ty_span: Span,
	},
}

impl Error for TypeError {
	fn to_report(&self) -> Report<Span> {
		let report = match self {
			TypeError::TypeMismatch { a, b, span } => Report::build(
				ReportKind::Error,
				span.file_id.clone(),
				span.range.start().into(),
			)
			.with_code(FluxErrorCode::TypeMismatch)
			.with_message(format!("type mismatch"))
			.with_label(
				Label::new(span.clone())
					.with_color(Color::Red)
					.with_message(format!(
						"type mismatch between `{}` and `{}`",
						a.inner, b.inner
					)),
			)
			.with_label(
				Label::new(a.span.clone())
					.with_color(Color::Blue)
					.with_message(format!("`{}`", a.inner)),
			)
			.with_label(
				Label::new(b.span.clone())
					.with_color(Color::Blue)
					.with_message(format!("`{}`", b.inner)),
			),
			TypeError::TraitBoundsUnsatisfied {
				ty,
				generic,
				span,
				missing_implementations,
			} => Report::build(
				ReportKind::Error,
				span.file_id.clone(),
				span.range.start().into(),
			)
			.with_code(FluxErrorCode::TraitBoundsUnsatisfied)
			.with_message(format!("trait bounds unsatisfied"))
			.with_label(
				Label::new(span.clone())
					.with_color(Color::Red)
					.with_message(format!(
						"the {} not implemented for `{}`",
						if missing_implementations.len() == 1 {
							format!(
								"trait `{}` is",
								missing_implementations
									.iter()
									.collect::<Vec<_>>()
									.first()
									.unwrap()
							)
						} else {
							format!(
								"traits {} are",
								comma_separated_end_with_and(missing_implementations.iter())
							)
						},
						ty.inner
					)),
			)
			.with_label(
				Label::new(generic.span.clone())
					.with_color(Color::Blue)
					.with_message(format!("{}", generic.inner)),
			),
			TypeError::UnknownPath { path } => Report::build(
				ReportKind::Error,
				path.span.file_id.clone(),
				path.span.range.start().into(),
			)
			.with_code(FluxErrorCode::TypeMismatch)
			.with_message(format!("unknown path"))
			.with_label(
				Label::new(path.span.clone())
					.with_color(Color::Red)
					.with_message(format!("unknown path `{}`", path.inner)),
			),
			TypeError::CouldNotInfer { ty_span } => Report::build(
				ReportKind::Error,
				ty_span.file_id.clone(),
				ty_span.range.start().into(),
			)
			.with_code(FluxErrorCode::CouldNotInfer)
			.with_message(format!("could not infer type"))
			.with_label(
				Label::new(ty_span.clone())
					.with_color(Color::Red)
					.with_message(format!("could not infer type")),
			)
			.with_note(format!("add type annotations")),
		};
		report.finish()
	}
}
