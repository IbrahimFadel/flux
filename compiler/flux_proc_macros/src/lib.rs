use parse::{AccessKind, ErrorAttribute};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use crate::parse::DiagnosticEnum;

extern crate proc_macro;

mod parse;

#[proc_macro_derive(Locatable)]
pub fn locatable(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_locatable_macro(&ast)
}

fn impl_locatable_macro(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let type_params = ast.generics.type_params();
    let (_impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let gen = quote! {
        impl <#(#type_params),*> WithSpan for #name #ty_generics #where_clause {}
    };
    gen.into()
}

#[proc_macro_derive(ToDiagnostic, attributes(error))]
pub fn into_diagnotic(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DiagnosticEnum);
    impl_to_diagnostic_enum(&input)
}

fn impl_to_diagnostic_enum(input: &DiagnosticEnum) -> TokenStream {
    let enum_name = &input.name;

    let mut variant_names = vec![];
    let mut variant_field_names = vec![];
    let mut locations = vec![];
    let mut diagnostic_codes = vec![];
    let mut primaries = vec![];
    let mut labels = vec![];
    let mut helps = Vec::with_capacity(input.variants.len());

    let mut i = 0;
    input.variants.iter().for_each(|variant| {
        let name = &variant.name;
        variant_names.push(quote!(#name));
        diagnostic_codes.push(quote! {
            DiagnosticCode::#name
        });
        let field_names = variant.fields.iter().map(|(name, _)| name);
        variant_field_names.push(quote! {
            #(#field_names),*
        });
        let mut this_labels = vec![];
        helps.push(quote!(None));
        variant
            .error_attributes
            .iter()
            .for_each(|error_attribute| match error_attribute {
                ErrorAttribute::Location(location) => {
                    let token_stream = match &location.access_kind {
                        AccessKind::Normal(field) => {
                            quote! {
                                #field
                            }
                        }
                        AccessKind::Map(field) => {
                            quote! {
                                #field.map_ref(|s| s.span.range.start().into())
                            }
                        }
                        AccessKind::MapInner(field) => {
                            quote! {
                                #field.map_inner_ref(|s| s.range.start().into())
                            }
                        }
                        AccessKind::FromFileSpan(field) => {
                            quote! {
                                InFile::new(#field.inner.range.start().into(), #field.file_id)
                            }
                        }
                    };
                    locations.push(token_stream);
                }
                ErrorAttribute::Primary(primary) => primaries.push(quote!(#primary)),
                ErrorAttribute::Label(label) => {
                    let msg = &label.msg;
                    match &label.access_kind {
                        parse::AccessKind::Normal(_) => {
                            todo!()
                        }
                        parse::AccessKind::Map(field) => {
                            if let Some(exprs) = &label.exprs {
                                let exprs = exprs.iter();
                                this_labels.push(quote! {
                                    #field.map_ref(|#field| format!(#msg, #(#exprs),*))
                                });
                            } else {
                                this_labels.push(quote! {
                                    #field.map_ref(|#field| format!(#msg))
                                });
                            }
                        }
                        parse::AccessKind::MapInner(field) => {
                            if let Some(exprs) = &label.exprs {
                                let exprs = exprs.iter();
                                this_labels.push(quote! {
                                    #field.map_inner_ref(|#field| format!(#msg, #(#exprs),*))
                                });
                            } else {
                                this_labels.push(quote! {
                                    #field.map_inner_ref(|#field| format!(#msg))
                                });
                            }
                        }
                        parse::AccessKind::FromFileSpan(field) => {
                            this_labels.push(quote! {
                                #field.to_file_spanned(#msg.to_string())
                            });
                        }
                    }
                }
                ErrorAttribute::Help(label) => {
                    let msg = &label.msg;
                    if let Some(exprs) = &label.exprs {
                        let exprs = exprs.iter();
                        helps[i] = quote! {
                            Some(format!(#msg, #(#exprs),*))
                        };
                    } else {
                        helps[i] = quote! {
                            Some(format!(#msg))
                        };
                    }
                }
            });
        labels.push(quote! {
            #(#this_labels),*
        });

        i += 1;
    });

    let gen = quote! {
        use flux_diagnostics::{ToDiagnostic, Diagnostic, DiagnosticCode};
        use flux_span::{FileId, WithSpan};
        use itertools::Itertools;

        impl ToDiagnostic for #enum_name {
            fn to_diagnostic(&self) -> Diagnostic {
                match self {
                    #(
                        Self::#variant_names { #variant_field_names } => Diagnostic::error(
                                #locations,
                                DiagnosticCode::#variant_names,
                                #primaries.to_string(),
                                vec![
                                    #labels
                                ]
                            ).opt_with_help(#helps),
                    )*
                }
            }
        }
    };
    gen.into()
}
