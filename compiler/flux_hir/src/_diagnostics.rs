// use flux_diagnostics::{Diagnostic, DiagnosticCode, ToDiagnostic};
// use flux_span::{FileSpanned, InFile, Span, Spanned, WithSpan};
// use itertools::Itertools;

// #[derive(Debug, Clone)]
// pub(crate) enum LowerError {
//     CannotAccessPrivatePathSegment {
//         path: FileSpanned<String>,
//         erroneous_segment: FileSpanned<String>,
//     },
//     CouldNotResolveEmptyPath {
//         path_span: InFile<Span>,
//     },
//     CouldNotResolveModDecl {
//         decl: FileSpanned<String>,
//         candidate_paths: Vec<String>,
//     },
//     CouldNotResolveUsePath {
//         path: FileSpanned<String>,
//         erroneous_segment: FileSpanned<String>,
//     },
//     IncorrectNumArgsInCall {
//         expected_number: FileSpanned<usize>,
//         got_number: FileSpanned<usize>,
//         function: String,
//     },
//     IncorrectNumGenericParamsInApplyMethod {
//         got_num: FileSpanned<usize>,
//         expected_num: FileSpanned<usize>,
//     },
//     // RestictionsInApplyMethodDoesntMatchTraitDecl {
//     //     restriction_in_: FileSpanned<String>,
//     //     restriction_in_trait_decl: FileSpanned<String>,
//     // },
//     MethodsDontBelongInApply {
//         trait_methods_declared: InFile<Spanned<Vec<String>>>,
//         methods_that_dont_belond: InFile<Spanned<Vec<String>>>,
//     },
//     StmtFollowingTerminatorExpr {
//         terminator: InFile<Span>,
//         following_expr: InFile<Span>,
//     },
//     TriedApplyingPrivateTrait {
//         trt: String,
//         declared_as_private: InFile<Span>,
//         application: InFile<Span>,
//     },
//     TriedCallingPrivateFunction {
//         function: String,
//         declared_as_private: InFile<Span>,
//         call: InFile<Span>,
//     },
//     UnimplementedTraitMethods {
//         trait_methods_declared: InFile<Spanned<Vec<String>>>,
//         unimplemented_methods: InFile<Spanned<Vec<String>>>,
//     },
//     UnknownGenericInWherePredicate {
//         /// The unknown generic
//         generic: FileSpanned<String>,
//         /// Where the generics are declared
//         generic_params: InFile<Span>,
//     },
//     UnresolvedFunction {
//         function: FileSpanned<String>,
//     },
//     UnresolvedTrait {
//         trt: FileSpanned<String>,
//     },
// }

// impl ToDiagnostic for LowerError {
//     fn to_diagnostic(&self) -> flux_diagnostics::Diagnostic {
//         match self {
//             Self::CannotAccessPrivatePathSegment {
//                 path,
//                 erroneous_segment,
//             } => Diagnostic::error(
//                 path.map_ref(|span| span.span.range.start().into()),
//                 DiagnosticCode::CannotAccessPrivatePathSegment,
//                 "cannot access private path segment".to_string(),
//                 vec![
//                     path.map_inner_ref(|path| {
//                         format!("cannot access private path segment in path `{path}`")
//                     }),
//                     erroneous_segment.map_inner_ref(|erroneous_segment| {
//                         format!("private segment `{erroneous_segment}`")
//                     }),
//                 ],
//             ),
//             Self::CouldNotResolveEmptyPath { path_span } => Diagnostic::error(
//                 path_span.map_ref(|span| span.range.start().into()),
//                 DiagnosticCode::CouldNotResolveEmptyPath,
//                 "could not resolve empty path".to_string(),
//                 vec!["could not resolve empty path"
//                     .to_string()
//                     .file_span(path_span.file_id, path_span.inner)],
//             ),
//             Self::CouldNotResolveModDecl {
//                 decl,
//                 candidate_paths,
//             } => Diagnostic::error(
//                 decl.map_ref(|span| span.span.range.start().into()),
//                 DiagnosticCode::CouldNotResolveModDecl,
//                 "could not resolve module declaration".to_string(),
//                 vec![decl.map_inner_ref(|name| {
//                     format!("could not resolve module declaration for `{name}`")
//                 })],
//             )
//             .with_help(format!(
//                 "create the module at one of the following paths: {}",
//                 candidate_paths
//                     .iter()
//                     .map(|path| format!("`{path}`"))
//                     .join(", ")
//             )),
//             Self::CouldNotResolveUsePath {
//                 path,
//                 erroneous_segment,
//             } => Diagnostic::error(
//                 path.map_ref(|span| span.span.range.start().into()),
//                 DiagnosticCode::CouldNotResolveUsePath,
//                 "could not resolve use path".to_string(),
//                 vec![
//                     path.map_inner_ref(|path| format!("could not resolve path `{path}`")),
//                     erroneous_segment.map_inner_ref(|erroneous_segment| {
//                         format!("unresolved path segment `{erroneous_segment}`")
//                     }),
//                 ],
//             ),
//             Self::IncorrectNumArgsInCall {
//                 expected_number,
//                 got_number,
//                 function,
//             } => Diagnostic::error(
//                 got_number.map_ref(|span| span.span.range.start().into()),
//                 DiagnosticCode::IncorrectNumArgsInCall,
//                 "incorrect number of arguments in function call".to_string(),
//                 vec![
//                     got_number.map_inner_ref(|num| format!("got {num} arguments")),
//                     expected_number.map_inner_ref(|num| {
//                         format!("expected {num} arguments in function `{function}`")
//                     }),
//                 ],
//             ),
//             Self::IncorrectNumGenericParamsInApplyMethod {
//                 got_num,
//                 expected_num,
//             } => Diagnostic::error(
//                 got_num.map_ref(|span| span.span.range.start().into()),
//                 DiagnosticCode::IncorrectNumGenericParamsInApplyMethod,
//                 "incorrect number of generic parameters in apply_method".to_string(),
//                 vec![
//                     got_num.map_inner_ref(|_| {
//                         format!("incorrect number of generic parameters in apply_method")
//                     }),
//                     expected_num.map_inner_ref(|num| {
//                         format!(
//                             "expected {num} generic parameter{}",
//                             if *num == 1 { "" } else { "s" }
//                         )
//                     }),
//                     got_num.map_inner_ref(|num| {
//                         format!(
//                             "got {num} generic parameter{}",
//                             if *num == 1 { "" } else { "s" }
//                         )
//                     }),
//                 ],
//             ),
//             Self::MethodsDontBelongInApply {
//                 trait_methods_declared,
//                 methods_that_dont_belond,
//             } => Diagnostic::error(
//                 methods_that_dont_belond.map_ref(|span| span.span.range.start().into()),
//                 DiagnosticCode::MethodsDontBelongInApply,
//                 "methods do not belong in apply".to_string(),
//                 vec![
//                     methods_that_dont_belond.map_inner_ref(|methods| {
//                         format!(
//                             "method{} {} do{} not belong in apply",
//                             if methods.len() == 1 { "" } else { "s" },
//                             methods
//                                 .iter()
//                                 .map(|method| format!("`{method}`"))
//                                 .join(", "),
//                             if methods.len() == 1 { "es" } else { "" },
//                         )
//                     }),
//                     trait_methods_declared.map_inner_ref(|methods| {
//                         format!(
//                             "trait method{} {} declared here",
//                             if methods.len() == 1 { "" } else { "s" },
//                             methods
//                                 .iter()
//                                 .map(|method| format!("`{method}`"))
//                                 .join(", ")
//                         )
//                     }),
//                 ],
//             ),
//             Self::StmtFollowingTerminatorExpr {
//                 terminator,
//                 following_expr,
//             } => Diagnostic::error(
//                 following_expr.map_ref(|span| span.range.start().into()),
//                 DiagnosticCode::StmtFollowingTerminatorExpr,
//                 "statements cannot follow a terminator expression in a block".to_string(),
//                 vec![
//                     FileSpanned::new(
//                         Spanned::new("terminator expression".to_string(), terminator.inner),
//                         terminator.file_id,
//                     ),
//                     FileSpanned::new(
//                         Spanned::new("illegal statement".to_string(), following_expr.inner),
//                         following_expr.file_id,
//                     ),
//                 ],
//             ),
//             Self::TriedApplyingPrivateTrait {
//                 trt,
//                 declared_as_private,
//                 application,
//             } => Diagnostic::error(
//                 application.map_ref(|span| span.range.start().into()),
//                 DiagnosticCode::TriedApplyingPrivateTrait,
//                 "trait is private and can't be applied here".to_string(),
//                 vec![
//                     format!("trait `{trt}` is private")
//                         .file_span(application.file_id, application.inner),
//                     "declared here as private"
//                         .to_string()
//                         .file_span(declared_as_private.file_id, declared_as_private.inner),
//                 ],
//             ),
//             Self::TriedCallingPrivateFunction {
//                 function,
//                 declared_as_private,
//                 call,
//             } => Diagnostic::error(
//                 call.map_ref(|span| span.range.start().into()),
//                 DiagnosticCode::TriedCallingPrivateFunction,
//                 "function is private and inaccessible".to_string(),
//                 vec![
//                     format!("function `{function}` is private").file_span(call.file_id, call.inner),
//                     "declared here as private"
//                         .to_string()
//                         .file_span(declared_as_private.file_id, declared_as_private.inner),
//                 ],
//             ),
//             Self::UnimplementedTraitMethods {
//                 trait_methods_declared,
//                 unimplemented_methods,
//             } => Diagnostic::error(
//                 unimplemented_methods.map_ref(|span| span.span.range.start().into()),
//                 DiagnosticCode::UnimplementedTraitMethods,
//                 "unimplemented trait methods in apply".to_string(),
//                 vec![
//                     unimplemented_methods.map_inner_ref(|methods| {
//                         format!(
//                             "unimeplemented trait method{} {} in apply",
//                             if methods.len() == 1 { "" } else { "s" },
//                             methods
//                                 .iter()
//                                 .map(|method| format!("`{method}`"))
//                                 .join(", ")
//                         )
//                     }),
//                     trait_methods_declared.map_inner_ref(|methods| {
//                         format!(
//                             "trait method{} {} declared here",
//                             if methods.len() == 1 { "" } else { "s" },
//                             methods
//                                 .iter()
//                                 .map(|method| format!("`{method}`"))
//                                 .join(", ")
//                         )
//                     }),
//                 ],
//             ),
//             Self::UnknownGenericInWherePredicate {
//                 generic,
//                 generic_params,
//             } => Diagnostic::error(
//                 generic.map_ref(|span| span.span.range.start().into()),
//                 DiagnosticCode::UnknownGenericInWherePredicate,
//                 "unknown generic used in where predicate".to_string(),
//                 vec![
//                     generic.map_inner_ref(|generic| {
//                         format!("unknown generic `{generic}` used in where predicate")
//                     }),
//                     generic_params
//                         .map_ref(|span| "generic parameters declared here".to_string().at(*span)),
//                 ],
//             ),
//             Self::UnresolvedFunction { function } => Diagnostic::error(
//                 function.map_ref(|span| span.span.range.start().into()),
//                 DiagnosticCode::UnresolvedFunction,
//                 "unresolved function".to_string(),
//                 vec![function.map_inner_ref(|path| format!("unresolved function `{path}`"))],
//             ),
//             Self::UnresolvedTrait { trt } => Diagnostic::error(
//                 trt.map_ref(|span| span.span.range.start().into()),
//                 DiagnosticCode::UnresolvedTrait,
//                 "unresolved trait".to_string(),
//                 vec![trt.map_inner_ref(|path| format!("unresolved trait `{path}`"))],
//             ),
//         }
//     }
// }
