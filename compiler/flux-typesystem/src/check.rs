use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};

use ariadne::{Color, Label, Report, ReportKind};
use flux_error::{Error, FluxErrorCode};
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
								comma_separated_end_with_and(missing_implementations)
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
		};
		report.finish()
	}
}

impl TypeChecker {
	pub fn new(implementations: HashMap<SmolStr, HashSet<SmolStr>>) -> Self {
		Self {
			tenv: TypeEnv::new(implementations),
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
			(Concrete(aa), Generic((_, restrictions))) => {
				match aa {
					ConcreteKind::Ident((name, _)) => {
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
								return Err(self.trait_bounds_unsatisfied(
									a,
									b,
									unification_span,
									missing_implementations,
								));
							}
						} else {
							return Err(self.trait_bounds_unsatisfied(
								a,
								b,
								unification_span,
								restrictions.clone(),
							));
						}
					}
					_ => {
						return Err(self.trait_bounds_unsatisfied(a, b, unification_span, restrictions.clone()))
					}
				};
			}
			(Concrete(aa), Concrete(bb)) => match (aa, bb) {
				(ConcreteKind::Ident((a_name, a_params)), ConcreteKind::Ident((b_name, b_params))) => {
					if a_name == b_name {
						if a_params.len() != 0 && b_params.len() == 0 {
							Ok(())
						} else if b_params.len() != 0 && a_params.len() == 0 {
							Ok(())
						} else {
							let result: Result<Vec<_>, _> = a_params
								.iter()
								.zip(b_params)
								.map(|(a_param, b_param)| self.unify(*a_param, *b_param, unification_span.clone()))
								.collect();
							if let Some(err) = result.err() {
								return Err(err);
							} else {
								Ok(())
							}
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
}

fn comma_separated_end_with_and<T: Display>(els: &HashSet<T>) -> String {
	let mut els: Vec<String> = els.iter().map(|el| format!("`{}`", el)).collect();
	let len = els.len();
	if len > 1 {
		els[len - 1] = format!("and {}", els[len - 1]);
	}
	els.join(", ")
}
