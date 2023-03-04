extern crate flux_proc_macros;
use flux_proc_macros::diagnostic;

#[test]
fn foo() {
    #[diagnostic]
    enum Test {
        #[error(
            location = path,
            primary = "cannot access private path segment",
            label at path = "cannot access private path segment in path `{path}",
            label at erroneous_segment = "private segment `{erroneous_segment}`",
            help = "some help"
        )]
        CannotAccessPrivatePathSegment {
            #[filespanned]
            path: String,
            // path_file_span: InFile<Span>,
            #[filespanned]
            erroneous_segment: String,
            // erroneous_segment_file_span: InFile<Span>,
        },
        #[error(
            location = a,
            primary = "fooo primary",
            label at a = "ahhu path `{a}`",
        )]
        IncorrectNumArgsInCall {
            #[filespanned]
            a: String,
            b: String,
        },
    }
    // #[derive(ToDiagnostic)]
    // enum Test {
    //     #[error(
    //         location = [map_inner] path,
    //         primary = "cannot access private path segment",
    //         label = [map_inner(path)] "cannot access private path segment in path `{path}",
    //         label = [map_inner(erroneous_segment)] "private segment `{erroneous_segment}`",
    //     )]
    //     CannotAccessPrivatePathSegment {
    //         path: FileSpanned<String>,
    //         erroneous_segment: FileSpanned<String>,
    //     },
    //     // #[error(
    //     //     location = [map_inner_location] path,
    //     //     primary = "cannot access private path segment",
    //     //     label = [map_inner(path)] "cannot access private path segment in path `{path}`",
    //     //     label = [map_inner(erroneous_segment)] "test `{erroneous_segment}`"
    //     // )]
    //     // CannotAccessPrivatePathSegment {
    //     //     path: FileSpanned<String>,
    //     //     erroneous_segment: FileSpanned<String>,
    //     // },

    //     // #[error(
    //     //     location = [map_inner_location] path,
    //     //     primary = "fooo primary",
    //     //     label = [map_inner(a)] "ahhu path `{a}`",
    //     // )]
    //     // IncorrectNumArgsInCall {
    //     //     a: FileSpanned<String>,
    //     //     b: FileSpanned<String>,
    //     // },
    // }
}
