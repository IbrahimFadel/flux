extern crate flux_proc_macros;
use flux_diagnostics::ToDiagnostic;
use flux_proc_macros::diagnostic;
use flux_span::{FileId, Span};

macro_rules! assert_enum_variants_have_fields {
    ($t:ty =>
        $(
            $variant:ident {
                $($field:ident: $field_ty:ty),+
            }
        ),+
    ) => {
        const _: () = {
            fn dummy(v: Test) {
                let _: $t = match v {
                    $(Test::$variant { $($field: _),+ } => todo!(),)+
                };
            }
        };
    };
}

#[test]
fn diagnostic_enum_proc_macro() {
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
            #[filespanned]
            erroneous_segment: String,
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

    assert_enum_variants_have_fields! {

    Test =>
        CannotAccessPrivatePathSegment {
            path: String,
            path_file_span: InFile<Span>,
            erroneous_segment: String,
            erroneous_segment_file_span: InFile<Span>
        },
        IncorrectNumArgsInCall {
            a: String,
            a_file_span: InFile<Span>,
            b: String
        }

    };

    let test = Test::IncorrectNumArgsInCall {
        a: "foo".to_string(),
        a_file_span: Span::poisoned().in_file(FileId::poisoned()),
        b: "bar".to_string(),
    };

    // make sure the trait was implemented
    let _test_diagnostic = test.to_diagnostic();
}
