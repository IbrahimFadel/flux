use flux_error::Span;

use super::*;

impl<'a> TypeEnv<'a> {
	pub(super) fn unify(
		&mut self,
		a: TypeId,
		b: TypeId,
		span: Span,
	) -> Result<Spanned<Type>, FluxError> {
		use Type::*;
		let ty = match (
			self.get_type_with_id(a).node.clone(),
			self.get_type_with_id(b).node.clone(),
		) {
			(Float, Float) => Float,
			(Int, Int) => Int,
			(Int, SInt(n)) => {
				let span = self.get_type_with_id(a).span.clone();
				self.set_id_type(a, Spanned::new(Ref(b), span));
				SInt(n)
			}
			(Int, UInt(n)) => {
				let span = self.get_type_with_id(a).span.clone();
				self.set_id_type(a, Spanned::new(Ref(b), span));
				UInt(n)
			}
			(SInt(n), Int) => {
				let span = self.get_type_with_id(b).span.clone();
				self.set_id_type(b, Spanned::new(Ref(a), span));
				SInt(n)
			}
			(UInt(n), Int) => {
				let span = self.get_type_with_id(b).span.clone();
				self.set_id_type(b, Spanned::new(Ref(a), span));
				UInt(n)
			}
			(F32, Float) | (Float, F32) | (F32, F32) => F32,
			(F64, Float) | (Float, F64) | (F64, F64) => F64,
			(Ref(a), Ref(b)) => {
				// this pattern isn't necessary, but it saves one function call if both sides are references. Since comparing variables is quite common, i think it makes sense to have
				return self.unify(a, b, span);
			}
			(Ref(a), _) => return self.unify(a, b, span),
			(_, Ref(b)) => return self.unify(a, b, span),
			(SInt(n1), SInt(n2)) => {
				if n1 == n2 {
					SInt(n1)
				} else {
					return Err(self.unification_err(a, b, span));
				}
			}
			(UInt(n1), UInt(n2)) => {
				if n1 == n2 {
					UInt(n1)
				} else {
					return Err(self.unification_err(a, b, span));
				}
			}
			(Ident(x), Ident(y)) => {
				if x == y {
					Ident(x)
				} else {
					return Err(self.unification_err(a, b, span));
				}
			}
			_ => return Err(self.unification_err(a, b, span)),
		};
		Ok(Spanned::new(ty, self.get_type_with_id(a).span.clone()))
	}

	fn unification_err(&self, a: TypeId, b: TypeId, span: Span) -> FluxError {
		let mut a_info = self.get_type_with_id(a).clone();
		let mut a_i = 0;
		while let Type::Ref(id) = &a_info.node {
			a_info = self.get_type_with_id(*id).clone();
			a_i += 1;
		}
		let mut b_info = self.get_type_with_id(b).clone();
		let mut b_i = 0;
		while let Type::Ref(id) = &b_info.node {
			b_info = self.get_type_with_id(*id).clone();
			b_i += 1;
		}
		let mut err = FluxError::default()
			.with_msg(format!(
				"could not unify `{}` and `{}`",
				a_info.node, b_info.node
			))
			.with_primary(
				format!("could not unify `{}` and `{}`", a_info.node, b_info.node),
				Some(span),
			)
			.with_label(
				format!("`{}` type", a_info.node),
				Some(self.get_type_with_id(a).span.clone()),
			)
			.with_label(
				format!("`{}` type", b_info.node),
				Some(self.get_type_with_id(b).span.clone()),
			);

		if a_i > 0 {
			err = err.with_label(
				format!("type `{}` inferred from here", a_info.node),
				Some(a_info.span),
			);
		}
		if b_i > 0 {
			err = err.with_label(
				format!("type `{}` inferred from here", b_info.node),
				Some(b_info.span),
			);
		}
		err
	}
}
