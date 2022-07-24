use ariadne::{Color, Label, Report, ReportKind};
use flux_error::{comma_separated_end_with_and, Error, FluxErrorCode};
use flux_span::{Span, Spanned};
use flux_typesystem::check::TypeError;
use itertools::Itertools;
use smol_str::SmolStr;

use crate::hir::FnParams;

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
		implementation_params: Spanned<FnParams>,
		declaration_params: FnParams,
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
	UninitializedFieldsInStructExpr {
		struct_name: SmolStr,
		struct_type: String,
		uninitialized_fields: Vec<SmolStr>,
		span: Span,
	},
	StmtAfterTerminalStmt {
		terminal_stmt: Span,
		stmt: Span,
	},
	IndexMemAccessOnNonPtrExpr {
		span: Span,
		ty: String,
	},
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
						declaration_params.0.len()
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
			LowerError::UninitializedFieldsInStructExpr {
				struct_name,
				struct_type,
				uninitialized_fields,
				span,
			} => Report::build(
				ReportKind::Error,
				span.file_id.clone(),
				span.range.start().into(),
			)
			.with_code(FluxErrorCode::UninitializedFieldsInStructExpr)
			.with_message(format!("uninitialized fields in struct expression"))
			.with_label(
				Label::new(span.clone())
					.with_color(Color::Red)
					.with_message(format!(
						"missing fields in `{}` struct initialization",
						struct_name,
					)),
			)
			.with_label(
				Label::new(span.clone())
					.with_color(Color::Blue)
					.with_message(format!(
						"missing {}",
						comma_separated_end_with_and(uninitialized_fields.iter())
					)),
			)
			.with_note(format!("`{}` is defined as: {}", struct_name, struct_type)),
			LowerError::StmtAfterTerminalStmt {
				terminal_stmt,
				stmt,
			} => Report::build(
				ReportKind::Error,
				stmt.file_id.clone(),
				stmt.range.start().into(),
			)
			.with_code(FluxErrorCode::StmtAfterTerminalStmt)
			.with_message(format!("found statement after block's terminal statement"))
			.with_label(
				Label::new(stmt.clone())
					.with_color(Color::Red)
					.with_message(format!("statement not allowed to follow block terminal")),
			)
			.with_label(
				Label::new(terminal_stmt.clone())
					.with_color(Color::Blue)
					.with_message(format!("terminal statement")),
			),
			LowerError::IndexMemAccessOnNonPtrExpr { span, ty } => Report::build(
				ReportKind::Error,
				span.file_id.clone(),
				span.range.start().into(),
			)
			.with_code(FluxErrorCode::IndexMemAccessOnNonPtrExpr)
			.with_message(format!("cannot index memory that isn't a pointer type"))
			.with_label(
				Label::new(span.clone())
					.with_color(Color::Red)
					.with_message(format!("cannot index memory that isn't a pointer type")),
			)
			.with_label(
				Label::new(span.clone())
					.with_color(Color::Blue)
					.with_message(ty),
			),
		};
		report.finish()
	}
}
