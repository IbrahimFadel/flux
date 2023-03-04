use ast::{DiagnosticEnum, ErrorAttribute, FieldAttribute};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

extern crate proc_macro;

mod ast;
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

#[proc_macro_attribute]
pub fn diagnostic(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DiagnosticEnum);
    impl_to_diagnostic_enum(&input)
}

fn impl_to_diagnostic_enum(input: &DiagnosticEnum) -> TokenStream {
    let enum_name = &input.name;
    let mut variants = vec![];
    let mut fields = vec![];
    let mut field_names = vec![];
    let mut locations = vec![];
    let mut primaries = vec![];
    let mut labels = vec![];
    let mut helps = vec![];

    let mut i = 0;
    input.variants.iter().for_each(|variant| {
        let variant_name = &variant.name;
        variants.push(quote! {
            #variant_name
        });

        let mut variant_fields = vec![];
        let mut variant_field_names = vec![];
        variant.fields.iter().for_each(|field| {
            let field_name = &field.name;

            variant_field_names.push(quote!(#field_name));

            let field_ty = &field.ty;
            let field_s = quote! {
                #field_name: #field_ty,
            };
            let field_s = if let FieldAttribute::FileSpanned = field.attr {
                let field_file_span = format_ident!("{}_file_span", field_name);
                variant_field_names.push(quote!(#field_file_span));
                quote! {
                    #field_s
                    #field_file_span: InFile<Span>,
                }
            } else {
                quote! {
                    #field_s
                }
            };
            variant_fields.push(field_s);
        });

        fields.push(variant_fields);
        field_names.push(variant_field_names);

        let mut variant_labels = vec![];

        helps.push(quote!(None));

        variant
            .error_attributes
            .iter()
            .for_each(|attrib| match attrib {
                ErrorAttribute::Location(location) => {
                    let field = &location.field;
                    let field_file_span = format_ident!("{}_file_span", field);
                    locations.push(quote! {
                        #field_file_span.map_ref(|span| span.range.start().into())
                    });
                }
                ErrorAttribute::Primary(primary) => {
                    primaries.push(primary);
                }
                ErrorAttribute::Label(label) => {
                    let field = &label.field;
                    let field_file_span = format_ident!("{}_file_span", field);
                    let msg = &label.msg;
                    if let Some(exprs) = &label.exprs {
                        let exprs = exprs.iter();
                        variant_labels.push(quote! {
                            #field_file_span.to_file_spanned(format!(#msg, #(#exprs),*))
                        });
                    } else {
                        variant_labels.push(quote! {
                            #field_file_span.to_file_spanned(format!(#msg))
                        });
                    }
                }
                ErrorAttribute::Help(help) => {
                    let msg = &help.msg;
                    if let Some(exprs) = &help.exprs {
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

        labels.push(variant_labels);
        i += 1;
    });

    let visibility = &input.visibility;

    let gen = quote! {
        use flux_diagnostics::{ToDiagnostic, Diagnostic, DiagnosticCode};
        use itertools::Itertools;
        use flux_span::{InFile, Span, FileId, WithSpan};


        #[derive(Debug, Clone)]
        #visibility enum #enum_name {
            #(
                #variants {
                    #(
                        #fields
                    )*
                }
            ),*
        }

        impl ToDiagnostic for #enum_name {
            fn to_diagnostic(&self) -> Diagnostic {
                match self {
                    #(
                        Self::#variants { #(#field_names),* } => Diagnostic::error(
                            #locations,
                            DiagnosticCode::#variants,
                            #primaries.to_string(),
                            vec![
                                #(#labels),*
                            ],
                        ).opt_with_help(#helps),
                    )*
                }
            }
        }
    };
    gen.into()
}

// #[proc_macro_attribute]
// pub fn filespanned(attr: TokenStream, item: TokenStream) -> TokenStream {
//     println!("attr: \"{}\"", attr.to_string());
//     println!("item: \"{}\"", item.to_string());
//     item
// }

// #[proc_macro_derive(ToDiagnostic, attributes(error, filespanned))]
// pub fn into_diagnotic(input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as DiagnosticEnum);
//     impl_to_diagnostic_enum(&input)
// }

// fn impl_to_diagnostic_enum(input: &DiagnosticEnum) -> TokenStream {
//     let enum_name = &input.name;

//     let mut variant_names = vec![];
//     let mut variant_field_names = vec![];
//     let mut locations = vec![];
//     let mut diagnostic_codes = vec![];
//     let mut primaries = vec![];
//     let mut labels = vec![];
//     let mut helps = Vec::with_capacity(input.variants.len());

//     let mut i = 0;
//     input.variants.iter().for_each(|variant| {
//         let name = &variant.name;
//         variant_names.push(quote!(#name));
//         diagnostic_codes.push(quote! {
//             DiagnosticCode::#name
//         });
//         let field_names = variant.fields.iter().map(|field| {
//             let name = &field.name;
//             let name_filespan = format_ident!("{}_file_span", name);
//             if let FieldAttribute::FileSpanned = field.attr {
//                 quote! {
//                     #name, #name_filespan
//                 }
//             } else {
//                 quote! {
//                     #name
//                 }
//             }
//         });
//         variant_field_names.push(quote! {
//             #(#field_names),*
//         });
//         let mut this_labels = vec![];
//         helps.push(quote!(None));
//         variant
//             .error_attributes
//             .iter()
//             .for_each(|error_attribute| match error_attribute {
//                 ErrorAttribute::Location(location) => {
//                     let field = &location.field;
//                     let field_in_file = format_ident!("{}_file_span", field);
//                     locations.push(quote! {
//                         #field_in_file.map_ref(|spanned| spanned.span.range.start().into())
//                     });
//                 }
//                 ErrorAttribute::Primary(primary) => primaries.push(quote!(#primary)),
//                 ErrorAttribute::Label(label) => {
//                     let field = &label.field;
//                     let msg = &label.msg;
//                     let field_file_span = format_ident!("{}_file_span", field);
//                     this_labels.push(quote! {
//                         #field_file_span.to_file_spanned(#msg)
//                     });
//                 }
//                 ErrorAttribute::Help(label) => {
//                     let msg = &label.msg;
//                     if let Some(exprs) = &label.exprs {
//                         let exprs = exprs.iter();
//                         helps[i] = quote! {
//                             Some(format!(#msg, #(#exprs),*))
//                         };
//                     } else {
//                         helps[i] = quote! {
//                             Some(format!(#msg))
//                         };
//                     }
//                 }
//             });
//         labels.push(quote! {
//             #(#this_labels),*
//         });

//         i += 1;
//     });

//     let gen = quote! {
//         use flux_diagnostics::{ToDiagnostic, Diagnostic, DiagnosticCode};
//         use flux_span::{FileId, WithSpan};
//         use itertools::Itertools;

//         impl ToDiagnostic for #enum_name {
//             fn to_diagnostic(&self) -> Diagnostic {
//                 match self {
//                     #(
//                         Self::#variant_names { #variant_field_names } => Diagnostic::error(
//                                 #locations,
//                                 DiagnosticCode::#variant_names,
//                                 #primaries.to_string(),
//                                 vec![
//                                     #labels
//                                 ]
//                             ).opt_with_help(#helps),
//                     )*
//                 }
//             }
//         }
//     };
//     gen.into()
// }
