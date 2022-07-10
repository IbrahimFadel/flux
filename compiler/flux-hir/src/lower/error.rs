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

		FluxError::build(
			format!("type mismatch"),
			span.clone(),
			FluxErrorCode::TypeMismatch,
			(
				format!(
					"type mismatch between `{}` and `{}`",
					env.fmt_ty(&env.get_type(a).inner),
					env.fmt_ty(&env.get_type(b).inner)
				),
				span,
			),
		)
		.with_label(format!("`{}`", env.fmt_ty(&aa.inner)), aa.span)
		.with_label(format!("`{}`", env.fmt_ty(&bb.inner)), bb.span)
		// .with_note(format!("consider a type cast or using a different value"))
	}
}
