use flux_proc_macros::ToDiagnostic;
use flux_span::{FileSpanned, InFile, Span};

#[derive(Debug, Clone, ToDiagnostic)]
pub(crate) enum LowerError {
    #[error(
        location = [map(path)],
        primary = "cannot access private path segment",
        label = [map_inner(path)] "cannot access private path segment in path `{path}`",
        label = [map_inner(erroneous_segment)] "private segment `{erroneous_segment}`",
    )]
    CannotAccessPrivatePathSegment {
        path: FileSpanned<String>,
        erroneous_segment: FileSpanned<String>,
    },
    #[error(
        location = [from_file_span(path_span)],
        primary = "could not resolve empty path",
        label = [from_file_span(path_span)] "could not resolve empty path",
    )]
    CouldNotResolveEmptyPath {
        path_span: InFile<Span>,
    },
    #[error(
        location = [map(decl)],
        primary = "could not resolve module declaration",
        label = [map_inner(decl)] "could not resolve module declaration for `{decl}`",
        help = "create the module at one of the following paths: {}"
            with (
                candidate_paths
                    .iter()
                    .map(|path| format!("`{path}`"))
                    .join(", ")
            ),
    )]
    CouldNotResolveModDecl {
        decl: FileSpanned<String>,
        candidate_paths: Vec<String>,
    },
    #[error(
        location = [map(path)],
        primary = "could not resolve use path",
        label = [map_inner(path)] "could not resolve path `{path}`",
        label = [map_inner(erroneous_segment)] "unresolved path segment `{erroneous_segment}`",
    )]
    CouldNotResolveUsePath {
        path: FileSpanned<String>,
        erroneous_segment: FileSpanned<String>,
    },
    #[error(
        location = [map(got_number)],
        primary = "incorrect number of arguments in function call",
        label = [map_inner(got_number)] "got {got_number} argument{}" with (if *got_number == 1 { "" } else { "s" }),
        label = [map_inner(expected_number)] "expected {expected_number} arguments in function `{function}`",
    )]
    IncorrectNumArgsInCall {
        got_number: FileSpanned<usize>,
        expected_number: FileSpanned<usize>,
        function: String,
    },
    #[error(
        location = [map(got_num)],
        primary = "incorrect number of generic parameters in apply_method",
        label = [map_inner(got_num)] "expected {got_num} generic parameter{}" with (if *got_num == 1 { "" } else { "s" }),
        label = [map_inner(expected_num)] "expected {expected_num} generic parameter{}" with (if *expected_num == 1 { "" } else { "s" }),
    )]
    IncorrectNumGenericParamsInApplyMethod {
        got_num: FileSpanned<usize>,
        expected_num: FileSpanned<usize>,
    },
    // RestictionsInApplyMethodDoesntMatchTraitDecl {
    //     restriction_in_: FileSpanned<String>,
    //     restriction_in_trait_decl: FileSpanned<String>,
    // },
    #[error(
        location = [map(methods_that_dont_belond)],
        primary = "methods do not belong in apply",
        label = [map_inner(methods_that_dont_belond)] "method{} {} do{} not belong in apply"
            with (
                if methods_that_dont_belond.len() == 1 { "" } else { "s" },
                methods_that_dont_belond
                    .iter()
                    .map(|method| format!("`{method}`"))
                    .join(", "),
                if methods_that_dont_belond.len() == 1 { "es" } else { "" }
            ),
        label = [map_inner(trait_methods_declared)] "trait method{} {} declared here"
            with (
                if trait_methods_declared.len() == 1 { "" } else { "s" },
                trait_methods_declared
                    .iter()
                    .map(|method| format!("`{method}`"))
                    .join(", ")
                
            ),
    )]
    MethodsDontBelongInApply {
        trait_methods_declared: FileSpanned<Vec<String>>,
        methods_that_dont_belond: FileSpanned<Vec<String>>,
    },
    #[error(
        location = [from_file_span(following_expr)],
        primary =  "statements cannot follow a terminator expression in a block",
        label = [from_file_span(terminator)] "terminator expression",
        label = [from_file_span(following_expr)] "illegal statement",
    )]
    StmtFollowingTerminatorExpr {
        terminator: InFile<Span>,
        following_expr: InFile<Span>,
    },
    #[error(
        location = [from_file_span(application)],
        primary = "trait is private and can't be applied here",
        label = [from_file_span(application)] "trait `{trt}` is private",
        label = [from_file_span(declared_as_private)] "declared here as private"
    )]
    TriedApplyingPrivateTrait {
        trt: String,
        declared_as_private: InFile<Span>,
        application: InFile<Span>,
    },
    #[error(
        location = [from_file_span(call)],
        primary = "function is private and inaccessible",
        label = [from_file_span(call)] "function `{function}` is private",
        label = [from_file_span(declared_as_private)] "declared here as private",
    )]
    TriedCallingPrivateFunction {
        function: String,
        declared_as_private: InFile<Span>,
        call: InFile<Span>,
    },
    #[error(
        location = [map(unimplemented_methods)],
        primary = "unimplemented trait methods in apply",
        label = [map_inner(unimplemented_methods)] "unimeplemented trait method{} {} in apply"
            with (
                if unimplemented_methods.len() == 1 { "" } else { "s" },
                unimplemented_methods
                    .iter()
                    .map(|method| format!("`{method}`"))
                    .join(", ")
            ),
        label = [map_inner(trait_methods_declared)] "trait method{} {} declared here"
            with (
                if trait_methods_declared.len() == 1 { "" } else { "s" },
                trait_methods_declared
                    .iter()
                    .map(|method| format!("`{method}`"))
                    .join(", ")
            ),
    )]
    UnimplementedTraitMethods {
        trait_methods_declared: FileSpanned<Vec<String>>,
        unimplemented_methods: FileSpanned<Vec<String>>,
    },
    #[error(
        location = [map(generic)],
        primary = "unknown generic used in where predicate",
        label = [map_inner(generic)] "unknown generic `{generic}` used in where predicate",
        label = [from_file_span(generic_params)] "generic parameters declared here"
    )]
    UnknownGenericInWherePredicate {
        // The unknown generic
        generic: FileSpanned<String>,
        // Where the generics are declared
        generic_params: InFile<Span>,
    },
    #[error(
        location = [map(function)],
        primary = "unresolved function",
        label = [map_inner(function)] "unresolved function `{function}`"
    )]
    UnresolvedFunction {
        function: FileSpanned<String>,
    },
    #[error(
        location = [map(trt)],
        primary = "unresolved trait",
        label = [map_inner(trt)] "unresolved trait `{trt}`"
    )]
    UnresolvedTrait {
        trt: FileSpanned<String>,
    },
}
