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
    let type_params = ast.generics.type_params();
    let (_impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let gen = quote! {
        impl <#(#type_params),*> WithSpan for #name #ty_generics #where_clause {}
    };
    gen.into()
}
