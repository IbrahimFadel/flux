use flux_diagnostics::fmt::{quote_and_listify, Plural};
use flux_proc_macros::diagnostic;
use itertools::Itertools;

#[diagnostic]
pub enum LowerError {
    #[error(
        location = name,
        primary = "unknown generic referenced",
        label at name = "unknown generic `{name}` referenced",
    )]
    UnknownGeneric {
        #[filespanned]
        name: String,
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
        location = generics_that_caused_duplication,
        primary = "duplicate generic parameters",
        label at generics_that_caused_duplication = "duplicate generics {}" with (
            quote_and_listify(generics_that_caused_duplication.iter().sorted())
        ),
        label at generics_that_were_chilling = "previously defined here"
    )]
    DuplicateGenericParams {
        #[filespanned]
        generics_that_were_chilling: (),
        #[filespanned]
        generics_that_caused_duplication: Vec<String>,
    },
    #[error(
        location = val,
        primary = "integer too large",
        label at val = "integer value {val} too large",
        help = "max value is u64::MAX"
    )]
    PositiveIntegerOverflow {
        #[filespanned]
        val: String,
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
        location = path,
        primary = "path resolved to incorrect item kind",
        label at path = "expected {path} to resolve to a {expected} but it was a {got}",
    )]
    ExpectedDifferentItem {
        #[filespanned]
        path: String,
        expected: &'static str,
        got: &'static str,
    },
    #[error(
        location = struct_name,
        primary = "missing fields in struct expression",
        label at struct_name = "missing fields in `{struct_name}` initialization",
        label at missing_fields = "missing fields {}" with (
            quote_and_listify(missing_fields.iter())
        )
    )]
    MissingFieldsInStructExpr {
        #[filespanned]
        struct_name: String,
        #[filespanned]
        missing_fields: Vec<String>,
    },
    #[error(
        location = got_names,
        primary = "missing generic arguments",
        label at got_names = "got {} generic argument{}, {}" with (got_names.len(), got_names.plural("s"), quote_and_listify(got_names.iter())),
        label at expected_names = "expected {} generic argument{}, {}" with (expected_names.len(), expected_names.plural("s"), quote_and_listify(expected_names.iter()))
    )]
    MissingGenericArguments {
        #[filespanned]
        got_names: Vec<String>,
        #[filespanned]
        expected_names: Vec<String>,
    },
    #[error(
        location = item_name,
        primary = "unused generics",
        label at item_name = "unused generics in `{item_name}`",
        label at unused_generics = "{} unused" with (
            quote_and_listify(unused_generics.iter().sorted())
        ),
    )]
    UnusedGenerics {
        #[filespanned]
        item_name: String,
        #[filespanned]
        unused_generics: Vec<String>,
    },
    #[error(
        location = local,
        primary = "unknown local referenced",
        label at local = "unknown local `{local}` referenced"
    )]
    UnknownLocal {
        #[filespanned]
        local: String,
    },
    #[error(
        location = intrinsic,
        primary = "unknown intrinsic",
        label at intrinsic = "unknown intrinsic {intrinsic}"
    )]
    UnknownIntrinsic {
        #[filespanned]
        intrinsic: String,
    },
    #[error(
        location = got_num,
        primary = "incorrect number of arguments in call",
        label at got_num = "got {got_num} argument{}" with (got_num.plural("s")),
        label at expected_num = "expected {expected_num} argument{}" with (expected_num.plural("s"))
    )]
    IncorrectNumberOfArgs {
        #[filespanned]
        got_num: usize,
        #[filespanned]
        expected_num: usize,
    },
    #[error(
        location = ty,
        primary = "type does not implement trait",
        label at ty = "type `{ty}` does not implement trait `{trt}`"
    )]
    TypeDoesNotImplementTrait {
        #[filespanned]
        ty: String,
        trt: String,
    },
}
