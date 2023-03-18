use flux_diagnostics::{quote_and_listify, Plural};
use flux_proc_macros::diagnostic;
use itertools::Itertools;

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
                quote_and_listify(candidate_paths.iter())
            ),
    )]
    CouldNotResolveModDecl {
        #[filespanned]
        decl: String,
        candidate_paths: Vec<String>,
    },
    #[error(
        location = path,
        primary = "could not resolve path",
        label at path = "could not resolve path `{path}`",
        label at erroneous_segment = "unresolved path segment `{erroneous_segment}`",
    )]
    CouldNotResolvePath {
        #[filespanned]
        path: String,
        #[filespanned]
        erroneous_segment: String,
    },
    #[error(
        location = generics_that_caused_duplication,
        primary = "duplicate generics",
        label at generics_that_caused_duplication = "duplicate generics {}" with (
            quote_and_listify(generics_that_caused_duplication.iter().sorted())
        ),
        label at generics_that_were_chilling = "previously defined here"
    )]
    DuplicateGenerics {
        #[filespanned]
        generics_that_were_chilling: (),
        #[filespanned]
        generics_that_caused_duplication: Vec<String>,
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
                quote_and_listify(methods_that_dont_belond.iter().sorted()),
                methods_that_dont_belond.singular("es")
            ),
        label at trait_methods_declared = "trait method{} {} declared here"
            with (
                trait_methods_declared.plural("s"),
                quote_and_listify(trait_methods_declared.iter().sorted())
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
                quote_and_listify(unimplemented_methods.iter().sorted())
            ),
        label at trait_methods_declared = "trait method{} {} declared here"
            with (
                trait_methods_declared.plural("s"),
                quote_and_listify(trait_methods_declared.iter().sorted())
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
            quote_and_listify(generic_params.iter())
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
                quote_and_listify(unused_generic_params.iter())
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
    #[error(
        location = struct_path,
        primary = "could not resolve struct",
        label at struct_path = "could not resolve struct `{struct_path}`",
        help = "there exists a {other_item_kind} with the same name, did you mean that?"
    )]
    CouldNotFindStructButFoundAnotherItem {
        #[filespanned]
        struct_path: String,
        other_item_kind: String,
    },
    #[error(
        location = function_path,
        primary = "could not resolve function",
        label at function_path = "could not resolve function `{function_path}`",
        help = "there exists a {other_item_kind} with the same name, did you mean that?"
    )]
    CouldNotFindFunctionButFoundAnotherItem {
        #[filespanned]
        function_path: String,
        other_item_kind: String,
    },
    #[error(
        location = uninitialized_fields,
        primary = "uninitialized fields in struct expression",
        label at uninitialized_fields = "uninitialized fields {} for `{struct_name}` struct initialization" with (
            quote_and_listify(uninitialized_fields.iter())
        )
    )]
    UninitializedFieldsInStructExpr {
        #[filespanned]
        uninitialized_fields: Vec<String>,
        struct_name: String,
    },
    #[error(
        location = unknown_fields,
        primary = "unknown fields in struct expression",
        label at unknown_fields = "unknown fields {} for `{struct_name}` struct initialization" with (
            quote_and_listify(unknown_fields.iter())
        )
    )]
    UnknownFieldsInStructExpr {
        #[filespanned]
        unknown_fields: Vec<String>,
        struct_name: String,
    },
    #[error(
        location = method,
        primary = "could not find method or field",
        label at method = "could not find method called `{method}`{}" with (
            if let Some(for_type) = for_type {
                format!(" for type `{for_type}`")
            } else {
                "".to_string()
            }
        ),
        help = "either create a method, or use a trait that defines it"
    )]
    CouldNotFindMethodReferenced {
        #[filespanned]
        method: String,
        for_type: Option<String>,
    },
    #[error(
        location = field,
        primary = "could not find field",
        label at field = "could not find field called `{field}`{}" with (
            if let Some(for_type) = for_type {
                format!(" for type `{for_type}`")
            } else {
                "".to_string()
            }
        ),
    )]
    CouldNotFindFieldReferenced {
        #[filespanned]
        field: String,
        for_type: Option<String>,
    },
    #[error(
        location = ty,
        primary = "associated type do not belong in apply",
        label at ty = "associated type `{ty}` do not belong in apply for trait `{trait_name}`"
    )]
    AssocTypeDoesntBelong {
        #[filespanned]
        ty: String,
        trait_name: String,
    },
    #[error(
        location = apply,
        primary = "missing associated types in apply",
        label at apply = "missing associated types {} in application of trait `{}`" with (
            quote_and_listify(types.iter().sorted()),
            trait_name
        )
    )]
    UnassignedAssocTypes {
        types: Vec<String>,
        #[filespanned]
        apply: (),
        trait_name: String,
    },
    #[error(
        location = ty,
        primary = "unresolved type",
        label at ty = "unresolved type `{ty}`",
    )]
    UnresolvedType {
        #[filespanned]
        ty: String,
    },
    #[error(
        location = strukt,
        primary = "unresolved struct",
        label at strukt = "unresolved struct `{strukt}`",
    )]
    UnresolvedStruct {
        #[filespanned]
        strukt: String,
    },
    #[error(
        location = path,
        primary = "cannot access private item",
        label at path = "cannot access private item `{path}`",
    )]
    AccessedPrivateItem {
        #[filespanned]
        path: String,
    },
    #[error(
        location = path,
        primary = "cannot access private struct",
        label at path = "cannot access private struct `{path}`",
    )]
    AccessedPrivateStruct {
        #[filespanned]
        path: String,
    },
}
