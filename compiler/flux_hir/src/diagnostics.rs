use flux_diagnostics::fmt::quote_and_listify;
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
}
