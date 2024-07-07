use proc_macro::TokenStream;
use quote::quote;

use self::ast::TestSuite;

pub(crate) mod ast;
mod parse;

pub(super) fn impl_to_test_suite(test_suite: &TestSuite) -> TokenStream {
    let generics = test_suite.generics();
    let unifications = test_suite
        .unifications()
        .map(|unification| unification.as_token_stream(&generics));
    quote! {
        let interner = Box::leak(Box::new(flux_span::Interner::new()));
        let mut source_cache = flux_diagnostics::SourceCache::new(interner);
        let mut tenv = flux_typesystem::TEnv::new(interner);
        let mut tchk = flux_typesystem::TChecker::new(&mut tenv);

        let file_id = source_cache.add_input_file("test.flx", String::new());
        let span = flux_span::Span::poisoned();
        let file_span = flux_span::WithSpan::in_file(span, file_id);

        #(#unifications)*
    }
    .into()
}
