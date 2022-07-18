use ariadne::{Color, Label, Report, ReportKind};
use flux_error::{Error, FluxErrorCode};
use flux_span::{Span, Spanned};

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
		};
		report.finish()
	}
}

impl TypeChecker {
	pub fn new() -> Self {
		Self {
			tenv: TypeEnv::new(),
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
			(Concrete(aa), Concrete(bb)) => match (aa, bb) {
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
						_ => return Err(self.type_mismatch(a, b, unification_span)),
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
						_ => return Err(self.type_mismatch(a, b, unification_span)),
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
						_ => return Err(self.type_mismatch(a, b, unification_span)),
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
			(Int(aa), Int(bb)) => match (aa, bb) {
				(Some(aa), Some(bb)) => self.unify(*aa, *bb, unification_span),
				(Some(_), None) => {
					self.tenv.vars.insert(
						b,
						Spanned {
							inner: TypeKind::Int(Some(a)),
							span: akind.span,
						},
					);
					Ok(())
				}
				(None, Some(_)) => {
					self.tenv.vars.insert(
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
}

#[derive(Debug)]
pub enum TypeError {
	TypeMismatch {
		/// Since we can't format TypeKind without a typeenv, we need to pass them after they've been formatted
		a: Spanned<String>,
		b: Spanned<String>,
		span: Span,
	},
}
