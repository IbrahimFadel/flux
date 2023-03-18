use flux_diagnostics::{quote_and_listify, Plural};
use flux_proc_macros::diagnostic;
use flux_span::FileSpanned;
use itertools::Itertools;

#[diagnostic]
pub enum TypeError {
    #[error(
        location = implementation_a,
        primary = "conflicting trait implementations",
        label at implementation_a = "type `{impltor}` implements trait `{trait_name}` here",
        label at implementation_b = "and type `{impltor}` also implements trait `{trait_name}` here"
    )]
    ConflictingTraitImplementations {
        trait_name: String,
        impltor: String,
        #[filespanned]
        implementation_a: (),
        #[filespanned]
        implementation_b: (),
    },
    #[error(
        location = trait_name,
        primary = "trait does not exist",
        label at trait_name = "trait `{trait_name}` does not exist"
    )]
    TraitDoesNotExist {
        #[filespanned]
        trait_name: String,
    },
    #[error(
        location = type_supposed_to_implement_trait,
        primary = "type does not implement trait",
        label at type_supposed_to_implement_trait = "trait `{}` is not implemented for `{type_supposed_to_implement_trait}`" with (
            restriction
        ),
        label at restriction = "trait restriction occurs here"
    )]
    TraitNotImplementedForType {
        #[filespanned]
        restriction: String,
        #[filespanned]
        type_supposed_to_implement_trait: String,
    },
    #[error(
        location = ty,
        primary = "trait restrictions not met",
        label at ty = "trait restriction{} {} not met for type `{ty}`" with (
            unmet_restrictions.plural("s"),
            quote_and_listify(unmet_restrictions.iter().map(|restriction| restriction.inner.inner.clone()).sorted())
        ),
        labels for unmet_restrictions = "restriction `{looped_value}` defined here"
    )]
    TraitRestrictionsNotMet {
        #[filespanned]
        ty: String,
        unmet_restrictions: Vec<FileSpanned<String>>,
    },
    #[error(
        location = span,
        primary =  "type mismatch",
        label at span =  "type mismatch between `{a}` and `{b}`",
        label at a = "`{a}`",
        label at b = "`{b}`",
    )]
    TypeMismatch {
        #[filespanned]
        a: String,
        #[filespanned]
        b: String,
        #[filespanned]
        span: (),
    },
    #[error(
        location = path,
        primary = "unknown function referenced",
        label at path = "unknown function `{path}` referenced",
    )]
    UnknownFunction {
        #[filespanned]
        path: String,
    },
    #[error(
        location = path,
        primary = "unknown struct referenced",
        label at path = "unknown struct `{path}` referenced",
    )]
    UnknownStruct {
        #[filespanned]
        path: String,
    },
    #[error(
        location = path,
        primary = "unknown type referenced",
        label at path = "unknown type `{path}` referenced",
    )]
    UnknownType {
        #[filespanned]
        path: String,
    },
    #[error(
        location = name,
        primary = "unknown variable referenced",
        label at name = "unknown variable `{name}` referenced",
    )]
    UnknownVariable {
        #[filespanned]
        name: String,
    },
    #[error(
        location = int_types,
        primary = "the type of this int is ambiguous",
        label at int_types = "the type of this int is ambiguous between the following types: {}" with (
            quote_and_listify(int_types.iter().sorted())
        ),
        labels for applications = "application of trait `{trt}` to type `{looped_value}`",
        label at trt = "this trait restriction requires the type to be known",
        help = "try specifying the type of int by adding the int type to the end of the literal"
    )]
    MultiplePossibleIntSpecializations {
        #[filespanned]
        int_types: Vec<String>,
        #[filespanned]
        trt: String,
        // The name of specific int types at the span of the application
        applications: Vec<FileSpanned<String>>,
    },
}
