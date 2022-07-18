use ariadne::{Color, Label, Report, ReportKind};
use flux_error::{Error, FluxErrorCode};
use flux_span::{Span, Spanned};
use flux_typesystem::check::TypeError;
use itertools::Itertools;
use smol_str::SmolStr;

#[derive(Debug)]
pub enum LowerError {
	TypeError(TypeError),
	AppliedUnknownTrait {
		trt: Spanned<SmolStr>,
		struct_: Spanned<SmolStr>,
	},
	AppliedUnknownMethodToTrait {
		method: Spanned<SmolStr>,
		trt: Spanned<SmolStr>,
		trt_methods: Vec<SmolStr>,
	},
	UnimplementedTraitMethods {
		trt: Spanned<SmolStr>,
		struct_: Spanned<SmolStr>,
		unimplemented_methods: Vec<SmolStr>,
	},
}

impl Error for LowerError {
	fn to_report(&self) -> Report<Span> {
		let report = match self {
			LowerError::TypeError(err) => match err {
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
			},
			LowerError::AppliedUnknownTrait { trt, struct_ } => Report::build(
				ReportKind::Error,
				trt.span.file_id.clone(),
				trt.span.range.start().into(),
			)
			.with_code(FluxErrorCode::AppliedUnknownTrait)
			.with_message(format!("unknown trait `{}`", trt.inner))
			.with_label(
				Label::new(Span::combine(&trt.span, &struct_.span))
					.with_color(Color::Red)
					.with_message(format!(
						"cannot apply trait `{}` to `{}`: the trait does not exist",
						trt.inner, struct_.inner
					)),
			)
			.with_note(format!("declare it with `trait {} {{}}`", trt.inner)),
			LowerError::AppliedUnknownMethodToTrait {
				method,
				trt,
				trt_methods,
			} => Report::build(
				ReportKind::Error,
				method.span.file_id.clone(),
				method.span.range.start().into(),
			)
			.with_code(FluxErrorCode::AppliedUnknownMethodToTrait)
			.with_message(format!(
				"trait `{}` does not have method `{}`",
				trt.inner, method.inner
			))
			.with_label(
				Label::new(method.span.clone())
					.with_color(Color::Red)
					.with_message(format!(
						"no such method `{}` on trait `{}`",
						method.inner, trt.inner
					)),
			)
			.with_label(
				Label::new(trt.span.clone())
					.with_color(Color::Blue)
					.with_message(format!("`{}` defined here", trt.inner)),
			)
			.with_note(if trt_methods.len() > 0 {
				format!(
					"the trait `{}` has the following methods: {}",
					trt.inner,
					trt_methods.join(",")
				)
			} else {
				format!("the trait `{}` has no methods", trt.inner)
			}),
			LowerError::UnimplementedTraitMethods {
				trt,
				struct_,
				unimplemented_methods,
			} => Report::build(
				ReportKind::Error,
				struct_.span.file_id.clone(),
				struct_.span.range.start().into(),
			)
			.with_code(FluxErrorCode::UnimplementedTraitMethods)
			.with_message(format!("unimplemented trait methods"))
			.with_label(
				Label::new(struct_.span.clone())
					.with_color(Color::Red)
					.with_message(format!(
						"missing implementation for the methods {} in application of trait `{}` to struct `{}`",
						unimplemented_methods
							.iter()
							.map(|s| format!("`{s}`"))
							.join(", "),
						trt.inner,
						struct_.inner
					)),
			),
		};
		report.finish()
	}
}
