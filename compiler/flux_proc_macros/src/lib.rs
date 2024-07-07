use diagnostics::{ast::DiagnosticEnum, impl_to_diagnostic_enum};
use proc_macro::TokenStream;
use syn::parse_macro_input;
use typesystem::{ast::TestSuite, impl_to_test_suite};

extern crate proc_macro;

mod diagnostics;
mod typesystem;

#[proc_macro_attribute]
pub fn diagnostic(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DiagnosticEnum);
    impl_to_diagnostic_enum(&input)
}

#[proc_macro]
pub fn tenv(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as TestSuite);
    impl_to_test_suite(&input)
}
