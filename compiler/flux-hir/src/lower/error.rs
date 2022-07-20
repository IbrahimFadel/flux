use ariadne::{Color, Label, Report, ReportKind};
use flux_error::{Error, FluxErrorCode};
use flux_span::{Span, Spanned};
use flux_typesystem::check::TypeError;
use itertools::Itertools;
use smol_str::SmolStr;

use crate::hir::{FnParam, Type};

#[derive(Debug)]
pub enum LowerError {
	TypeError(TypeError),
	Missing {
		missing: Spanned<String>,
	},
	AppliedUnknownTrait {
		trt: Spanned<SmolStr>,
		ty: Spanned<String>,
	},
	AppliedUnknownMethodToTrait {
		method: Spanned<SmolStr>,
		trt: Spanned<SmolStr>,
		trt_methods: Vec<SmolStr>,
	},
	UnimplementedTraitMethods {
		trt: Spanned<SmolStr>,
		ty: Spanned<String>,
		unimplemented_methods: Vec<SmolStr>,
	},
	IncorrectNumberOfParamsInTraitMethodDefinition {
		method_name: String,
		implementation_params: Spanned<Vec<Spanned<FnParam>>>,
		declaration_params: Spanned<Vec<Spanned<FnParam>>>,
	},
	UnknownStruct {
		name: Spanned<SmolStr>,
	},
	NoSuchStructField {
		struct_name: Spanned<SmolStr>,
		field_name: Spanned<SmolStr>,
	},
	UnknownIntrinsic {
		intrinsic: Spanned<SmolStr>,
	},
	IncorrectNumberOfArgsInCall {},
}

impl Error for LowerError {
	fn to_report(&self) -> Report<Span> {
		let report = match self {
			LowerError::TypeError(err) => return err.to_report(),
			LowerError::Missing { missing } => Report::build(
				ReportKind::Error,
				missing.span.file_id.clone(),
				missing.span.range.start().into(),
			)
			.with_code(FluxErrorCode::MissingDataInLowering)
			.with_message(missing.inner.clone()),
			LowerError::AppliedUnknownTrait { trt, ty } => Report::build(
				ReportKind::Error,
				trt.span.file_id.clone(),
				trt.span.range.start().into(),
			)
			.with_code(FluxErrorCode::AppliedUnknownTrait)
			.with_message(format!("unknown trait `{}`", trt.inner))
			.with_label(
				Label::new(Span::combine(&trt.span, &ty.span))
					.with_color(Color::Red)
					.with_message(format!(
						"cannot apply trait `{}` to `{}`: the trait does not exist",
						trt.inner, ty.inner
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
				ty,
				unimplemented_methods,
			} => Report::build(
				ReportKind::Error,
				ty.span.file_id.clone(),
				ty.span.range.start().into(),
			)
			.with_code(FluxErrorCode::UnimplementedTraitMethods)
			.with_message(format!("unimplemented trait methods"))
			.with_label(
				Label::new(ty.span.clone())
					.with_color(Color::Red)
					.with_message(format!(
						"missing implementation for the methods {} in application of trait `{}` to type `{}`",
						unimplemented_methods
							.iter()
							.map(|s| format!("`{s}`"))
							.join(", "),
						trt.inner,
						ty.inner
					)),
			),
			LowerError::IncorrectNumberOfParamsInTraitMethodDefinition {
				method_name,
				implementation_params,
				declaration_params,
			} => Report::build(
				ReportKind::Error,
				implementation_params.span.file_id.clone(),
				implementation_params.span.range.start().into(),
			)
			.with_code(FluxErrorCode::IncorrectNumberOfParamsInTraitMethodDefinition)
			.with_message(format!(
				"incorrect number of parameters in trait method definition"
			))
			.with_label(
				Label::new(implementation_params.span.clone())
					.with_color(Color::Red)
					.with_message(format!(
						"incorrect number of parameters in trait method definition"
					)),
			)
			.with_label(
				Label::new(implementation_params.span.clone())
					.with_color(Color::Red)
					.with_message(format!(
						"the method `{}` is defined with {} parameters",
						method_name,
						declaration_params.len()
					)),
			),
			LowerError::UnknownStruct { name } => Report::build(
				ReportKind::Error,
				name.span.file_id.clone(),
				name.span.range.start().into(),
			)
			.with_code(FluxErrorCode::UnknownStruct)
			.with_message(format!("unknown struct referenced"))
			.with_label(
				Label::new(name.span.clone())
					.with_color(Color::Red)
					.with_message(format!("unknown struct `{}` referenced", name.inner)),
			),
			LowerError::NoSuchStructField {
				struct_name,
				field_name,
			} => Report::build(
				ReportKind::Error,
				field_name.span.file_id.clone(),
				field_name.span.range.start().into(),
			)
			.with_code(FluxErrorCode::NoSuchStructField)
			.with_message(format!("no such struct field"))
			.with_label(
				Label::new(field_name.span.clone())
					.with_color(Color::Red)
					.with_message(format!(
						"no such field `{}` on struct `{}`",
						field_name.inner, struct_name.inner
					)),
			),
			LowerError::UnknownIntrinsic { intrinsic } => Report::build(
				ReportKind::Error,
				intrinsic.span.file_id.clone(),
				intrinsic.span.range.start().into(),
			)
			.with_code(FluxErrorCode::NoSuchIntrinsic)
			.with_message(format!("no such intrinsic `{}`", intrinsic.inner))
			.with_label(
				Label::new(intrinsic.span.clone())
					.with_color(Color::Red)
					.with_message(format!("no such intrinsic `{}`", intrinsic.inner)),
			),
			LowerError::IncorrectNumberOfArgsInCall {} => todo!(),
		};
		report.finish()
	}
}
