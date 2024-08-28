use flux_diagnostics::fmt::quote_and_listify;
use flux_proc_macros::diagnostic;

#[diagnostic]
pub enum TypeError {
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
        location = ty,
        primary = "could not infer type",
        label at ty = "could not infer type",
        help = "consider adding an explicit type annotation"
    )]
    CouldNotInfer {
        #[filespanned]
        ty: (),
    },
    #[error(
        location = ty,
        primary = "could not infer type",
        label at ty = "could not infer type",
        label at ty = "could be any of {}" with (quote_and_listify(potential_types.iter())),
    )]
    CouldBeMultipleTypes {
        #[filespanned]
        ty: (),
        potential_types: Vec<String>,
    },
}
