use flux_diagnostics::{Diagnostic, DiagnosticCode, ToDiagnostic};
use flux_span::{FileId, FileSpanned, InFile, Span, Spanned};

pub enum TypeError {
    ConflictingTraitImplementations {
        implementation_a_file_id: FileId,
        implementation_b_file_id: FileId,
        impl_a_trt: String,
        impl_a_ty: Spanned<String>,
        impl_b_trt: String,
        impl_b_ty: Spanned<String>,
    },
    TraitInTraitRestrictionDoesNotExist {
        trait_name: FileSpanned<String>,
    },
    TraitNotImplementedForType {
        restriction: FileSpanned<String>,
        type_supposed_to_implement_trait: FileSpanned<String>,
    },
    /// A type mismatch
    ///
    /// `a` and `b` are both formatted to `String`, where
    /// `a` is the type the must be matched by `b`, and
    /// `span` is the location in which the type unification was triggered.
    TypeMismatch {
        a: FileSpanned<String>,
        b: FileSpanned<String>,
        // a_got_from_list: Vec<InFile<Span>>,
        // b_got_from_list: Vec<InFile<Span>>,
        span: InFile<Span>,
    },
    UnknownFunction {
        path: FileSpanned<String>,
    },
    UnknownStruct {
        path: FileSpanned<String>,
    },
    UnknownType {
        path: FileSpanned<String>,
    },
    UnknownVariable {
        name: FileSpanned<String>,
    },
}

impl ToDiagnostic for TypeError {
    fn to_diagnostic(&self) -> flux_diagnostics::Diagnostic {
        match self {
            Self::ConflictingTraitImplementations {
                implementation_a_file_id,
                implementation_b_file_id,
                impl_a_trt,
                impl_a_ty,
                impl_b_trt,
                impl_b_ty,
            } => Diagnostic::error(
                InFile::new(
                    impl_a_ty.span.range.start().into(),
                    *implementation_a_file_id,
                ),
                DiagnosticCode::ConflictingTraitImplementations,
                "conflicting trait implementations".to_string(),
                vec![
                    FileSpanned::new(
                        impl_a_ty.map_ref(|ty| {
                            format!("type `{ty}` implements trait `{impl_a_trt}` here")
                        }),
                        *implementation_a_file_id,
                    ),
                    FileSpanned::new(
                        impl_b_ty.map_ref(|ty| {
                            format!("type `{ty}` implements trait `{impl_b_trt}` here")
                        }),
                        *implementation_b_file_id,
                    ),
                ],
            ),
            Self::TraitInTraitRestrictionDoesNotExist { trait_name } => Diagnostic::error(
                trait_name.map_ref(|name| name.span.range.start().into()),
                DiagnosticCode::TraitInTraitRestrictionDoesNotExist,
                "trait does not exist".to_string(),
                vec![trait_name.map_inner_ref(|name| format!("trait `{name}` does not exist"))],
            ),
            Self::TraitNotImplementedForType {
                restriction,
                type_supposed_to_implement_trait,
            } => Diagnostic::error(
                type_supposed_to_implement_trait.map_ref(|name| name.span.range.start().into()),
                DiagnosticCode::TraitNotImplementedForType,
                "type does not implement trait".to_string(),
                vec![
                    type_supposed_to_implement_trait.map_inner_ref(|name| {
                        format!(
                            "trait `{}` is not implemented for `{name}`",
                            restriction.inner.inner
                        )
                    }),
                    restriction.map_inner_ref(|_| format!("trait restriction occurs here")),
                ],
            ),
            Self::TypeMismatch {
                a,
                b,
                span,
                // a_got_from_list,
                // b_got_from_list,
            } => {
                let labels = vec![
                    InFile::new(
                        Spanned::new(
                            format!(
                                "type mismatch between `{}` and `{}`",
                                a.inner.inner, b.inner.inner
                            ),
                            span.inner,
                        ),
                        span.file_id,
                    ),
                    a.clone(),
                    b.clone(),
                ];
                // a_got_from_list.iter().for_each(|a| {
                //     let label = FileSpanned::new("from".to_string().at(a.inner), a.file_id);
                //     labels.push(label);
                // });
                // b_got_from_list.iter().for_each(|b| {
                //     let label = FileSpanned::new("from".to_string().at(b.inner), b.file_id);
                //     labels.push(label);
                // });
                Diagnostic::error(
                    span.map_ref(|span| span.range.start().into()),
                    DiagnosticCode::TypeMismatch,
                    "type mismatch".to_string(),
                    labels,
                )
            }
            Self::UnknownVariable { name } => Diagnostic::error(
                name.map_ref(|span| span.span.range.start().into()),
                DiagnosticCode::UnknownLocal,
                "unknown variable referenced".to_string(),
                vec![FileSpanned::new(
                    Spanned::new(
                        format!("unknown variable `{}` referenced", name.inner.inner),
                        name.span,
                    ),
                    name.file_id,
                )],
            ),
            Self::UnknownFunction { path } => Diagnostic::error(
                path.map_ref(|span| span.span.range.start().into()),
                DiagnosticCode::UnknownFunction,
                "unknown function referenced".to_string(),
                vec![FileSpanned::new(
                    Spanned::new(
                        format!("unknown function `{}` referenced", path.inner.inner),
                        path.span,
                    ),
                    path.file_id,
                )],
            ),
            Self::UnknownStruct { path } => Diagnostic::error(
                path.map_ref(|span| span.span.range.start().into()),
                DiagnosticCode::UnknownStruct,
                "unknown struct referenced".to_string(),
                vec![FileSpanned::new(
                    Spanned::new(
                        format!("unknown struct `{}` referenced", path.inner.inner),
                        path.span,
                    ),
                    path.file_id,
                )],
            ),
            Self::UnknownType { path } => Diagnostic::error(
                path.map_ref(|span| span.span.range.start().into()),
                DiagnosticCode::UnknownStruct,
                "unknown type referenced".to_string(),
                vec![FileSpanned::new(
                    Spanned::new(
                        format!("unknown type `{}` referenced", path.inner.inner),
                        path.span,
                    ),
                    path.file_id,
                )],
            ),
        }
    }
}
