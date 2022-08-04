use std::{collections::HashSet, fmt::Debug};

use ariadne::{Color, Label, Report, ReportKind};
use flux_error::{Error, FluxErrorCode};
use flux_span::{Span, Spanned};
use lasso::{Rodeo, Spur};
use tracing::{debug, info};

use crate::{
	infer::TypeEnv,
	r#type::{ConcreteKind, TypeId, TypeKind},
};

#[derive(Debug)]
pub struct TypeChecker<'a> {
	pub tenv: TypeEnv<'a>,
}

impl<'a> TypeChecker<'a> {
	pub fn new(resolver: &'a Rodeo) -> Self {
		Self {
			tenv: TypeEnv::new(resolver),
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
					_ => self.verify_trait_bounds(b, a, unification_span, restrictions),
				}
			}
			(Concrete(ConcreteKind::Ptr(id)), Generic((b_name, restrictions))) => {
				match &self.tenv.get_type(*id).inner {
					TypeKind::Generic((a_name, _)) => {
						if b_name == a_name {
							return Err(self.type_mismatch(a, b, unification_span));
						} else {
							Ok(())
						}
					}
					_ => self.verify_trait_bounds(a, b, unification_span, restrictions),
				}
			}
			(Generic((a_name, a_restrictions)), Generic((b_name, b_restrictions))) => {
				self.verify_trait_bounds(b, a, unification_span, a_restrictions)
			}
			(_, Generic((_, restrictions))) => {
				self.verify_trait_bounds(a, b, unification_span, restrictions)
			}
			(Generic((_, restrictions)), _) => {
				self.verify_trait_bounds(b, a, unification_span, restrictions)
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
			(Float(aa), Float(bb)) => match (aa, bb) {
				(Some(aa), Some(bb)) => self.unify(*aa, *bb, unification_span),
				(Some(_), None) => {
					self.tenv.set_type(
						b,
						Spanned {
							inner: TypeKind::Float(Some(a)),
							span: akind.span,
						},
					);
					Ok(())
				}
				(None, Some(_)) => {
					self.tenv.set_type(
						a,
						Spanned {
							inner: TypeKind::Float(Some(b)),
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
		&mut self,
		ty: TypeId,
		generic: TypeId,
		unification_span: Span,
		traits: &HashSet<(Spur, Vec<TypeId>)>,
	) -> Result<(), TypeError> {
		if traits.len() == 0 {
			return Ok(());
		}
		let inner_ty = self.tenv.inner_type(&self.tenv.get_type(ty));
		if let TypeKind::Generic((_, actual_restrictions)) = inner_ty {
			let missing_implementations = self.get_missing_implementations(traits, &actual_restrictions);
			if missing_implementations.len() == 0 {
				return Ok(());
			} else {
				return Err(self.trait_bounds_unsatisfied(
					ty,
					generic,
					unification_span,
					missing_implementations,
				));
			}
		}
		// let inner_ty = self.tenv.inner_type(&self.tenv.get_type(generic));
		// if let TypeKind::Generic((_, actual_restrictions)) = inner_ty {
		// 	let missing_implementations = self.get_missing_implementations(&actual_restrictions, traits);
		// 	if missing_implementations.len() == 0 {
		// 		return Ok(());
		// 	} else {
		// 		return Err(self.trait_bounds_unsatisfied(
		// 			ty,
		// 			generic,
		// 			unification_span,
		// 			missing_implementations,
		// 		));
		// 	}
		// }

		let (type_name, type_params) = match self.tenv.get_type(ty).inner.clone() {
			TypeKind::Concrete(ConcreteKind::Ident(ident)) => ident,
			ty => (SmolStr::from(self.tenv.fmt_ty(&ty)), vec![]),
		};

		let mut missing_implementations = HashSet::new();
		for restriction in traits {
			let implements_trait_restriction =
				self.does_type_implement_trait(restriction, &type_name, &type_params);
			info!(
				"`{}` implemented for `{}`: {}",
				restriction.0, type_name, implements_trait_restriction
			);
			if !implements_trait_restriction {
				missing_implementations.insert(restriction.clone());
			}
		}
		if missing_implementations.len() > 0 {
			return Err(self.trait_bounds_unsatisfied(
				ty,
				generic,
				unification_span,
				missing_implementations,
			));
		}
		Ok(())
	}

	fn does_type_implement_trait(
		&mut self,
		trt: &(SmolStr, Vec<TypeId>),
		type_name: &SmolStr,
		type_params: &[TypeId],
	) -> bool {
		// TODO: get trait restrictions with name
		debug!(
			"does {} implement {}",
			self.tenv.fmt_ident_w_types(type_name, type_params),
			self.tenv.fmt_ident_w_types(&trt.0, &trt.1),
		);
		if let Some(type_param_pairs) = self
			.tenv
			.trait_implementors
			.get_trait_implentations_for_type(trt, type_name)
			.cloned()
		{
			let mut acceptable_implementations = 0;
			for (trait_ty_params, impltor_ty_params) in &type_param_pairs {
				// Check if impltor type params unifies with any of the implmtors already stored in implementation table
				let impltor_type_params_unification_results: Vec<_> = impltor_ty_params
					.iter()
					.zip(type_params)
					.map(|(a, b)| {
						let span = self.tenv.get_type(*a).span.clone();
						self.unify(*a, *b, span)
					})
					.filter(|unification| unification.is_ok())
					.collect();

				// next, check if trait type params in implementation table unify with type params supplied to trait type params being applied
				let trait_type_params_unification_results: Vec<_> = trait_ty_params
					.iter()
					.zip(&trt.1)
					.map(|(a, b)| {
						let span = self.tenv.get_type(*a).span.clone();
						self.unify(*a, *b, span)
					})
					.filter(|unification| unification.is_ok())
					.collect();

				if impltor_type_params_unification_results.len() == type_params.len()
					&& trait_type_params_unification_results.len() == trait_ty_params.len()
				{
					acceptable_implementations += 1;
				}
			}

			match acceptable_implementations {
				0 => false,
				1 => true,
				_ => todo!(),
			}
		} else {
			false
		}
	}

	fn get_missing_implementations(
		&mut self,
		expected_restricions: &HashSet<(Spur, Vec<TypeId>)>,
		actual_restrictions: &HashSet<(Spur, Vec<TypeId>)>,
	) -> HashSet<(Spur, Vec<TypeId>)> {
		expected_restricions
			.iter()
			.cloned()
			.filter(
				|(expected_restriction_name, expected_restriction_type_params)| {
					let mut acceptable_implementations = 0;
					for (actual_restriction_name, actual_restriction_type_params) in actual_restrictions {
						if actual_restriction_name == expected_restriction_name {
							let results: Vec<_> = expected_restriction_type_params
								.iter()
								.zip(actual_restriction_type_params)
								.map(|(a, b)| {
									let span = self.tenv.get_type(*a).span.clone();
									self.unify(*a, *b, span)
								})
								.filter(|unification| unification.is_ok())
								.collect();
							if results.len() == expected_restriction_type_params.len() {
								acceptable_implementations += 1;
							}
						}
					}
					match acceptable_implementations {
						0 => true,
						1 => false,
						_ => todo!(),
					}
				},
			)
			.collect::<HashSet<(SmolStr, Vec<TypeId>)>>()
	}

	pub fn has_trait(&self, id: TypeId, trt: &SmolStr) -> bool {
		// let name = &SmolStr::from(self.tenv.fmt_ty(&self.tenv.get_type(id)));
		let (type_name, type_params) = match &self.tenv.get_type(id).inner {
			TypeKind::Concrete(ConcreteKind::Ident(ident)) => ident,
			_ => todo!(),
		};
		todo!()
		// match self.tenv.get_trait_implementations(type_name) {
		// 	Some(implementations) => {
		// 		todo!()
		// 	}
		// 	None => false,
		// }
		// match self.tenv.get_trait_implementations(type_name) {
		// 	Some(implementations) => match implementations.get(trt) {
		// 		Some(_) => true,
		// 		None => false,
		// 	},
		// 	None => false,
		// }
	}

	// pub fn check_type_params(
	// 	&self,
	// 	a_generics: &Spanned<IndexMap<SmolStr, HashSet<SmolStr>>>,
	// 	b_generics: &Spanned<IndexMap<SmolStr, HashSet<SmolStr>>>,
	// 	ty: Spanned<SmolStr>,
	// ) -> Result<(), TypeError> {
	// 	let a_len = a_generics.len();
	// 	let b_len = b_generics.len();

	// 	if a_len != b_len {
	// 		return Err(TypeError::IncorrectNumberOfTypeParamsSuppliedToTrait {
	// 			num_params_expected: a_generics.map(|params| params.len()),
	// 			num_params_got: b_generics.map(|params| params.len()),
	// 			ty,
	// 		});
	// 	}

	// 	Ok(())
	// }

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
		missing_implementations: HashSet<(SmolStr, Vec<TypeId>)>,
	) -> TypeError {
		let ty = self.tenv.get_type(ty);
		let generic = self.tenv.get_type(generic);
		let missing_implementations = missing_implementations
			.iter()
			.map(|(name, ids)| {
				(
					name.clone(),
					ids.iter().map(|id| self.tenv.fmt_id(*id)).collect(),
				)
			})
			.collect();
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
		missing_implementations: HashSet<(SmolStr, Vec<String>)>,
	},
	UnknownPath {
		path: Spanned<SmolStr>,
	},
	CouldNotInfer {
		ty_span: Span,
	},
	IncorrectNumberOfTypeParamsSuppliedToTrait {
		num_params_expected: Spanned<usize>,
		num_params_got: Spanned<usize>,
		ty: Spanned<SmolStr>,
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
									.0
							)
						} else {
							todo!()
							// format!(
							// 	"traits {} are",
							// 	comma_separated_end_with_and(missing_implementations.iter())
							// )
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
			TypeError::IncorrectNumberOfTypeParamsSuppliedToTrait {
				num_params_expected,
				num_params_got,
				ty,
			} => Report::build(
				ReportKind::Error,
				num_params_got.span.file_id.clone(),
				num_params_got.span.range.start().into(),
			)
			.with_code(FluxErrorCode::IncorrectNumberOfTypeParamsSuppliedToTrait)
			.with_message(format!(
				"incorrect number of type parameters supplied to trait `{}`",
				ty.inner
			))
			.with_label(
				Label::new(num_params_got.span.clone())
					.with_color(Color::Red)
					.with_message(format!(
						"incorrect number of type parameters supplied to trait `{}`",
						ty.inner
					)),
			)
			.with_label(
				Label::new(num_params_expected.span.clone())
					.with_color(Color::Blue)
					.with_message(format!(
						"expected {} type parameters",
						num_params_expected.inner
					)),
			)
			.with_label(
				Label::new(num_params_got.span.clone())
					.with_color(Color::Blue)
					.with_message(format!("got {} type parameters", num_params_got.inner)),
			)
			.with_note(if num_params_got.inner > num_params_expected.inner {
				format!(
					"try removing {} type parameters from `{}`",
					num_params_got.inner - num_params_expected.inner,
					ty.inner
				)
			} else {
				format!(
					"try adding {} type parameters to `{}`",
					num_params_expected.inner - num_params_got.inner,
					ty.inner
				)
			}),
		};
		report.finish()
	}
}

#[cfg(test)]
mod tests {
	use std::{
		collections::{HashMap, HashSet},
		io::Write,
	};

	use super::{TypeChecker, TypeError};
	use crate::r#type::{ConcreteKind, TypeKind};
	use ariadne::sources;
	use flux_error::Error;
	use flux_span::{FileId, Span, Spanned};
	use smol_str::SmolStr;
	use text_size::TextRange;

	macro_rules! spnd {
		($x:expr) => {
			Spanned::new($x, spn!())
		};
	}

	macro_rules! spn {
		() => {
			Span::new(
				TextRange::new(0.into(), 0.into()),
				FileId(SmolStr::from("foo.flx")),
			)
		};
	}

	macro_rules! sint {
		($n:expr) => {
			TypeKind::Concrete(ConcreteKind::SInt($n))
		};
	}
	macro_rules! uint {
		($n:expr) => {
			TypeKind::Concrete(ConcreteKind::UInt($n))
		};
	}

	macro_rules! int {
		() => {
			TypeKind::Int(None)
		};
	}

	macro_rules! f64 {
		() => {
			TypeKind::Concrete(ConcreteKind::F64)
		};
	}

	macro_rules! f32 {
		() => {
			TypeKind::Concrete(ConcreteKind::F32)
		};
	}

	macro_rules! float {
		() => {
			TypeKind::Float(None)
		};
	}

	macro_rules! ptr {
		($tchk:expr, $x:expr) => {
			TypeKind::Concrete(ConcreteKind::Ptr($tchk.tenv.insert(spnd!($x))))
		};
	}

	macro_rules! generic {
		($x:expr) => {
			TypeKind::Generic(($x, HashSet::new()))
		};
		($x:expr, $restrictions:expr) => {
			TypeKind::Generic(($x, $restrictions))
		};
	}

	macro_rules! ident {
		($x:expr) => {
			TypeKind::Concrete(ConcreteKind::Ident(($x, vec![])))
		};
		($x:expr, $params:expr) => {
			TypeKind::Concrete(ConcreteKind::Ident(($x, $params)))
		};
	}

	macro_rules! tparam {
		($tchk:expr, $x:expr) => {
			$tchk.tenv.insert(spnd!($x))
		};
	}

	struct Buf(pub String);

	impl Write for &mut Buf {
		fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
			let s = String::from_utf8_lossy(buf);
			self.0 += s.into_owned().as_str();
			Ok(buf.len())
		}

		fn flush(&mut self) -> std::io::Result<()> {
			Ok(())
		}
	}

	fn format_err(err: &TypeError) -> String {
		let mut s = Buf(String::new());
		let files: Vec<(FileId, String)> = vec![(FileId(SmolStr::from("foo.flx")), format!(" "))];
		err.to_report().write(sources(files), &mut s).unwrap();
		s.0
	}

	pub fn check(tchk: &mut TypeChecker, a: TypeKind, b: TypeKind) -> String {
		let a_id = tchk.tenv.insert(spnd!(a.clone()));
		let b_id = tchk.tenv.insert(spnd!(b.clone()));

		match tchk.unify(a_id, b_id, spn!()) {
			Ok(_) => format!("{} <-> {} ✓", tchk.tenv.fmt_ty(&a), tchk.tenv.fmt_ty(&b)),
			Err(err) => format_err(&err),
		}
	}

	#[macro_export]
	#[cfg(test)]
	macro_rules! test_unify {
		($name:ident, $a:expr, $b:expr) => {
			paste::paste! {
			#[test]
			fn [<test_typecheck_ $name>]() {
				let mut tchk = TypeChecker::new();
				let result = check(&mut tchk, $a, $b);
				insta::assert_snapshot!(result);
			}}
		};
	}

	test_unify!(sint32_sint32, sint!(32), sint!(32));
	test_unify!(sint32_sint31, sint!(32), sint!(31));
	test_unify!(sint32_uint32, sint!(32), uint!(32));
	test_unify!(uint32_uint31, uint!(32), uint!(31));
	test_unify!(uint32_uint32, uint!(32), uint!(32));
	test_unify!(uint32_int, uint!(32), int!());
	test_unify!(sint32_int, sint!(32), int!());
	test_unify!(int_int, int!(), int!());
	test_unify!(f32_f64, f32!(), f64!());
	test_unify!(f32_float, f32!(), float!());
	test_unify!(f64_float, f64!(), float!());
	test_unify!(float_float, float!(), float!());

	#[test]
	fn test_typecheck_floatptr_float() {
		let mut tchk = TypeChecker::new();
		let a = ptr!(tchk, float!());
		let b = float!();
		let result = check(&mut tchk, a, b);
		insta::assert_snapshot!(result);
	}

	#[test]
	fn test_typecheck_genericptr_float() {
		let mut tchk = TypeChecker::new();
		let a = ptr!(tchk, generic!(SmolStr::from("T")));
		let b = float!();
		let result = check(&mut tchk, a, b);
		insta::assert_snapshot!(result);
	}

	#[test]
	fn test_typecheck_genericptr_generic_same_name() {
		let mut tchk = TypeChecker::new();
		let a = ptr!(tchk, generic!(SmolStr::from("T")));
		let b = generic!(SmolStr::from("T"));
		let result = check(&mut tchk, a, b);
		insta::assert_snapshot!(result);
	}

	#[test]
	fn test_typecheck_generic_generic_diff_name() {
		let mut tchk = TypeChecker::new();
		let a = generic!(SmolStr::from("T"));
		let b = generic!(SmolStr::from("E"));
		let result = check(&mut tchk, a, b);
		insta::assert_snapshot!(result);
	}

	#[test]
	fn test_typecheck_generic_ident() {
		let mut tchk = TypeChecker::new();
		let a = generic!(SmolStr::from("T"));
		let b = ident!(SmolStr::from("Foo"));
		let result = check(&mut tchk, a, b);
		insta::assert_snapshot!(result);
	}

	#[test]
	fn test_typecheck_generic_ident_with_params() {
		let mut tchk = TypeChecker::new();
		let a = generic!(SmolStr::from("T"));
		let b = ident!(SmolStr::from("Foo"), vec![tparam!(tchk, sint!(32))]);
		let result = check(&mut tchk, a, b);
		insta::assert_snapshot!(result);
	}

	#[test]
	fn test_typecheck_ident_ident() {
		let mut tchk = TypeChecker::new();
		let a = ident!(SmolStr::from("Foo"));
		let b = ident!(SmolStr::from("Bar"));
		let result = check(&mut tchk, a, b);
		insta::assert_snapshot!(result);
	}

	#[test]
	fn test_typecheck_ident_ident_with_params() {
		let mut tchk = TypeChecker::new();
		let a = ident!(SmolStr::from("Foo"));
		let b = ident!(SmolStr::from("Bar"), vec![tparam!(tchk, sint!(32))]);
		let result = check(&mut tchk, a, b);
		insta::assert_snapshot!(result);
	}

	#[test]
	fn test_typecheck_ident_ident_diff_same_params() {
		let mut tchk = TypeChecker::new();
		let a = ident!(SmolStr::from("Foo"), vec![tparam!(tchk, sint!(32))]);
		let b = ident!(SmolStr::from("Bar"), vec![tparam!(tchk, sint!(32))]);
		let result = check(&mut tchk, a, b);
		insta::assert_snapshot!(result);
	}

	#[test]
	fn test_typecheck_ident_ident_same_diff_params() {
		let mut tchk = TypeChecker::new();
		let a = ident!(SmolStr::from("Foo"), vec![tparam!(tchk, sint!(32))]);
		let b = ident!(SmolStr::from("Foo"), vec![tparam!(tchk, uint!(32))]);
		let result = check(&mut tchk, a, b);
		insta::assert_snapshot!(result);
	}
}
