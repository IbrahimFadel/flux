use super::*;

#[derive(Debug)]
pub(crate) enum DriverError {
	CouldNotOpenModule {
		parent_dir: SmolStr,
		module: Spanned<SmolStr>,
	},
}

impl Error for DriverError {
	fn to_report(&self) -> Report<Span> {
		let report = match self {
			DriverError::CouldNotOpenModule { parent_dir, module } => Report::build(
				ReportKind::Error,
				module.span.file_id.clone(),
				module.span.range.start().into(),
			)
			.with_code(FluxErrorCode::CouldNotOpenModule)
			.with_message(format!(
				"could not open module `{}`\nno such file `{}/{}.flx` or `{}/{}/{}.flx`",
				module.inner, parent_dir, module.inner, parent_dir, module.inner, module.inner
			)),
		};
		report.finish()
	}
}
