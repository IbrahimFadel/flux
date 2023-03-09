use flux_proc_macros::diagnostic;
use itertools::Itertools;

trait Plural {
    fn plural(&self, suffix: &'static str) -> &str;
    fn singular(&self, suffix: &'static str) -> &str;
}

impl Plural for usize {
    fn plural(&self, suffix: &'static str) -> &str {
        if *self == 1 {
            ""
        } else {
            suffix
        }
    }

    fn singular(&self, suffix: &'static str) -> &str {
        if *self == 1 {
            suffix
        } else {
            ""
        }
    }
}

impl<T> Plural for Vec<T> {
    fn plural(&self, suffix: &'static str) -> &str {
        if self.len() == 1 {
            ""
        } else {
            suffix
        }
    }

    fn singular(&self, suffix: &'static str) -> &str {
        if self.len() == 1 {
            suffix
        } else {
            ""
        }
    }
}

#[diagnostic]
pub(crate) enum LowerError {
    #[error(
        location = path,
        primary = "cannot access private path segment",
        label at path = "cannot access private path segment in path `{path}`",
        label at erroneous_segment = "private segment `{erroneous_segment}`",
    )]
    CannotAccessPrivatePathSegment {
        #[filespanned]
        path: String,
        #[filespanned]
        erroneous_segment: String,
    },
    #[error(
        location = path,
        primary = "could not resolve empty path",
        label at path = "could not resolve empty path",
    )]
    CouldNotResolveEmptyPath {
        #[filespanned]
        path: (),
    },
    #[error(
        location = decl,
        primary = "could not resolve module declaration",
        label at decl = "could not resolve module declaration for `{decl}`",
        help = "create the module at one of the following paths: {}"
            with (
                candidate_paths
                    .iter()
                    .map(|path| format!("`{path}`"))
                    .join(", ")
            ),
    )]
    CouldNotResolveModDecl {
        #[filespanned]
        decl: String,
        candidate_paths: Vec<String>,
    },
    #[error(
        location = path,
        primary = "could not resolve use path",
        label at path = "could not resolve path `{path}`",
        label at erroneous_segment = "unresolved path segment `{erroneous_segment}`",
    )]
    CouldNotResolveUsePath {
        #[filespanned]
        path: String,
        #[filespanned]
        erroneous_segment: String,
    },
    #[error(
        location = got_number,
        primary = "incorrect number of arguments in function call",
        label at got_number = "got {got_number} argument{}" with (got_number.plural("s")),
        label at expected_number = "expected {expected_number} arguments in function `{function}`",
    )]
    IncorrectNumArgsInCall {
        #[filespanned]
        got_number: usize,
        #[filespanned]
        expected_number: usize,
        function: String,
    },
    #[error(
        location = got_num,
        primary = "incorrect number of generic parameters in apply method",
        label at got_num = "got {got_num} generic parameter{}" with (got_num.plural("s")),
        label at expected_num = "expected {expected_num} generic parameter{}" with (expected_num.plural("s")),
    )]
    IncorrectNumGenericParamsInApplyMethod {
        #[filespanned]
        got_num: usize,
        #[filespanned]
        expected_num: usize,
    },
    // RestictionsInApplyMethodDoesntMatchTraitDecl {
    //     restriction_in_: FileSpanned<String>,
    //     restriction_in_trait_decl: FileSpanned<String>,
    // },
    #[error(
        location = methods_that_dont_belond,
        primary = "methods do not belong in apply",
        label at methods_that_dont_belond = "method{} {} do{} not belong in apply"
            with (
                methods_that_dont_belond.plural("s"),
                methods_that_dont_belond
                    .iter()
                    .map(|method| format!("`{method}`"))
                    .join(", "),
                methods_that_dont_belond.singular("es")
            ),
        label at trait_methods_declared = "trait method{} {} declared here"
            with (
                trait_methods_declared.plural("s"),
                trait_methods_declared
                    .iter()
                    .map(|method| format!("`{method}`"))
                    .join(", ")

            ),
    )]
    MethodsDontBelongInApply {
        #[filespanned]
        trait_methods_declared: Vec<String>,
        #[filespanned]
        methods_that_dont_belond: Vec<String>,
    },
    #[error(
        location = apply_decl_generic_missing_where_predicate,
        primary = "missing where predicate in apply declaration method",
        label at apply_decl_generic_missing_where_predicate = "generic parameter `{apply_decl_generic_missing_where_predicate}` missing where predicate `{trait_decl_where_predicate}`",
        label at trait_decl_where_predicate = "where predicate `{trait_decl_where_predicate}` expected due to this"
    )]
    MissingWherePredicateInApplyMethod {
        #[filespanned]
        trait_decl_where_predicate: String,
        #[filespanned]
        apply_decl_generic_missing_where_predicate: String,
    },
    #[error(
        location = following_expr,
        primary =  "statements cannot follow a terminator expression in a block",
        label at terminator = "terminator expression",
        label at following_expr = "illegal statement",
    )]
    StmtFollowingTerminatorExpr {
        #[filespanned]
        terminator: (),
        #[filespanned]
        following_expr: (),
    },
    #[error(
        location = application,
        primary = "trait is private and can't be applied here",
        label at application = "trait `{trt}` is private",
        label at declared_as_private = "declared here as private"
    )]
    TriedApplyingPrivateTrait {
        trt: String,
        #[filespanned]
        declared_as_private: (),
        #[filespanned]
        application: (),
    },
    #[error(
        location = call,
        primary = "function is private and inaccessible",
        label at call = "function `{function}` is private",
        label at declared_as_private = "declared here as private",
    )]
    TriedCallingPrivateFunction {
        function: String,
        #[filespanned]
        declared_as_private: (),
        #[filespanned]
        call: (),
    },
    #[error(
        location = unimplemented_methods,
        primary = "unimplemented trait methods in apply",
        label at unimplemented_methods = "unimeplemented trait method{} {} in apply"
            with (
                unimplemented_methods.plural("s"),
                unimplemented_methods
                    .iter()
                    .map(|method| format!("`{method}`"))
                    .join(", ")
            ),
        label at trait_methods_declared = "trait method{} {} declared here"
            with (
                trait_methods_declared.plural("s"),
                trait_methods_declared
                    .iter()
                    .map(|method| format!("`{method}`"))
                    .join(", ")
            ),
    )]
    UnimplementedTraitMethods {
        #[filespanned]
        trait_methods_declared: Vec<String>,
        #[filespanned]
        unimplemented_methods: Vec<String>,
    },
    #[error(
        location = generic,
        primary = "unknown generic used in where predicate",
        label at generic = "unknown generic `{generic}` used in where predicate",
        label at generic_params = "generic parameters {} declared here" with (
            generic_params.iter().map(|param| format!("`{param}`")).join(", ")
        )
    )]
    UnknownGeneric {
        // The unknown generic
        #[filespanned]
        generic: String,
        // Where the generics are declared
        #[filespanned]
        generic_params: Vec<String>,
    },
    #[error(
        location = function,
        primary = "unresolved function",
        label at function = "unresolved function `{function}`"
    )]
    UnresolvedFunction {
        #[filespanned]
        function: String,
    },
    #[error(
        location = trt,
        primary = "unresolved trait",
        label at trt = "unresolved trait `{trt}`"
    )]
    UnresolvedTrait {
        #[filespanned]
        trt: String,
    },
    #[error(
        location = apply_decl_where_predicate,
        primary = "where predicate in apply declaration method does not match trait declaration",
        label at apply_decl_where_predicate = "where predicate in apply declaration method `{apply_decl_where_predicate}`",
        label at trait_decl_where_predicate = "where predicate in trait declaration method `{trait_decl_where_predicate}`",
        label at apply_generic_param = "generic parameter `{apply_generic_param}` needs the trait bound `{trait_decl_where_predicate}`",
        label at trait_generic_param = "generic parameter `{trait_generic_param}` with trait bound `{trait_decl_where_predicate}`",
    )]
    WherePredicatesDontMatchInApplyMethod {
        #[filespanned]
        trait_decl_where_predicate: String,
        #[filespanned]
        apply_decl_where_predicate: String,
        #[filespanned]
        trait_generic_param: String,
        #[filespanned]
        apply_generic_param: String,
    },
    #[error(
        location = unused_generic_params,
        primary = "unused generic parameters",
        label at unused_generic_params = "unused generic parameters {}"
            with (
                unused_generic_params.iter().map(|param| format!("`{param}`")).join(", ")
            ),
    )]
    UnusedGenericParams {
        #[filespanned]
        unused_generic_params: Vec<String>,
    },
    #[error(
        location = got_num,
        primary = "incorrect number of generic arguments in where predicate",
        label at expected_num = "expected {expected_num} arguments",
        label at got_num = "got {got_num} arguments",
    )]
    IncorrectNumGenericArgsInWherePredicate {
        #[filespanned]
        got_num: usize,
        #[filespanned]
        expected_num: usize,
    },
    #[error(
        location = generic,
        primary = "generic argument is missing required trait restriction",
        label at generic = "generic `{generic}` missing restriction `{restriction}`",
        label at restriction = "restriction `{restriction}` defined here",
    )]
    GenericArgDoesNotMatchRestriction {
        #[filespanned]
        generic: String,
        #[filespanned]
        restriction: String,
    },
}
