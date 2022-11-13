use proc_macro::TokenStream;
use quote::quote;

extern crate proc_macro;

#[proc_macro_derive(Locatable)]
pub fn locatable(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_locatable_macro(&ast)
}

fn impl_locatable_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl WithSpan for #name {}
    };
    gen.into()
}
