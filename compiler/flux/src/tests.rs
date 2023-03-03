extern crate flux_proc_macros;
use flux_proc_macros::ToDiagnostic;
use flux_span::FileSpanned;

#[test]
fn foo() {
    #[derive(ToDiagnostic)]
    enum Test {
        #[error(
            location = [map_inner] path,
            primary = "cannot access private path segment",
            label = [map_inner(path)] "cannot access private path segment in path `{path}",
            label = [map_inner(erroneous_segment)] "private segment `{erroneous_segment}`",
        )]
        CannotAccessPrivatePathSegment {
            path: FileSpanned<String>,
            erroneous_segment: FileSpanned<String>,
        },
        // #[error(
        //     location = [map_inner_location] path,
        //     primary = "cannot access private path segment",
        //     label = [map_inner(path)] "cannot access private path segment in path `{path}`",
        //     label = [map_inner(erroneous_segment)] "test `{erroneous_segment}`"
        // )]
        // CannotAccessPrivatePathSegment {
        //     path: FileSpanned<String>,
        //     erroneous_segment: FileSpanned<String>,
        // },

        // #[error(
        //     location = [map_inner_location] path,
        //     primary = "fooo primary",
        //     label = [map_inner(a)] "ahhu path `{a}`",
        // )]
        // IncorrectNumArgsInCall {
        //     a: FileSpanned<String>,
        //     b: FileSpanned<String>,
        // },
    }
}
