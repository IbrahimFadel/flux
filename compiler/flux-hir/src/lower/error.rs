use flux_error::{FluxError, FluxErrorCode, Span};
use flux_typesystem::ErrorHandler;

pub(crate) struct TypeCheckErrHandler;

impl ErrorHandler for TypeCheckErrHandler {
	type Error = FluxError;

	fn type_mismatch(
		&self,
		env: &flux_typesystem::TypeEnv,
		a: flux_typesystem::TypeId,
		b: flux_typesystem::TypeId,
		span: Span,
	) -> Self::Error {
		let aa = env.get_type(a);
		let bb = env.get_type(b);

		FluxError::default()
			.with_code(FluxErrorCode::TypeMismatch)
			.with_msg(format!("type mismatch"))
			.with_primary(
				format!(
					"type mismatch between `{}` and `{}`",
					env.get_type(a).inner,
					env.get_type(b).inner
				),
				Some(span),
			)
			.with_label(format!("`{}`", aa.inner), Some(aa.span))
			.with_label(format!("`{}`", bb.inner), Some(bb.span))
	}
}
