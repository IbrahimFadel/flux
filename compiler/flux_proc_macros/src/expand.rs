use proc_macro2::TokenStream;
use syn::DeriveInput;

struct Enum {}

pub fn derive(node: &DeriveInput) -> Result<TokenStream, ()> {
    // let input = Enum::from_syn(node)?;
    // input.validate()?;
    // Ok(impl_enum(input))
    Err(())
}

// use proc_macro2::TokenStream;
// use syn::DeriveInput;

// use crate::ast::Enum;

// pub fn derive(node: &DeriveInput) -> Result<TokenStream, ()> {
//     let input = Enum::from_syn(node)?;
//     input.validate()?;
//     Ok(impl_enum(input))
// }

// fn impl_enum(input: Enum) -> TokenStream {
//     todo!()
// }
