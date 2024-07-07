use quote::quote;
use syn::{punctuated::Punctuated, AngleBracketedGenericArguments, GenericArgument, Ident, Token};

pub(crate) mod kw {
    use syn::custom_keyword;

    custom_keyword!(With);
    custom_keyword!(Unify);

    custom_keyword!(A);
    custom_keyword!(B);
    custom_keyword!(C);
    custom_keyword!(D);

    custom_keyword!(int);
    custom_keyword!(float);
    custom_keyword!(never);
    custom_keyword!(unknown);
}

pub(crate) struct TestSuite {
    pub(crate) unification_directive: UnificationDirective,
    pub(crate) with_directive: Option<WithDirective>,
}

impl TestSuite {
    pub(crate) fn new(
        unification_directive: UnificationDirective,
        with_directive: Option<WithDirective>,
    ) -> Self {
        Self {
            unification_directive,
            with_directive,
        }
    }

    pub(crate) fn generics(&self) -> Vec<&GenericDefinition> {
        self.with_directive
            .as_ref()
            .map_or(vec![], |with_directive| {
                with_directive.generics.iter().collect()
            })
    }

    pub(crate) fn unifications(&self) -> impl Iterator<Item = &Unification> {
        self.unification_directive.unifications.iter()
    }
}

pub(crate) struct UnificationDirective {
    unifications: Punctuated<Unification, Token![;]>,
}

impl UnificationDirective {
    pub(crate) fn new(unifications: Punctuated<Unification, Token![;]>) -> Self {
        Self { unifications }
    }
}

pub(crate) struct Unification {
    a: Type,
    kind: UnificationKind,
    b: Type,
}

impl Unification {
    pub(crate) fn new(a: Type, kind: UnificationKind, b: Type) -> Self {
        Self { a, kind, b }
    }

    pub(crate) fn as_token_stream(
        &self,
        generics: &[&GenericDefinition],
    ) -> proc_macro2::TokenStream {
        let a = self.a.as_tid_token_stream(generics);
        let b = self.b.as_tid_token_stream(generics);
        let unification_assertian = match self.kind {
            UnificationKind::Success => quote!(.is_ok()),
            UnificationKind::Failure => quote!(.is_err()),
        };
        quote! {
            let a = #a;
            let b = #b;
            assert!(tchk.unify(a, b, file_span.clone())#unification_assertian);
        }
    }
}

pub(crate) enum UnificationKind {
    Success,
    Failure,
}

pub(crate) struct GenericDefinition {
    name: GenericName,
    restrictions: Punctuated<Path, Token![,]>,
}

impl GenericDefinition {
    pub(crate) fn new(name: GenericName, restrictions: Punctuated<Path, Token![,]>) -> Self {
        Self { name, restrictions }
    }
}

pub(crate) struct WithDirective {
    generics: Punctuated<GenericDefinition, Token![,]>,
}

impl WithDirective {
    pub(crate) fn new(generics: Punctuated<GenericDefinition, Token![,]>) -> Self {
        Self { generics }
    }
}

// pub(crate) struct WithDirective {
//     name: GenericName,
//     restrictions: Punctuated<Path, Token![,]>,
// }

// impl WithDirective {
//     pub(crate) fn new(name: GenericName, restrictions: Punctuated<Path, Token![,]>) -> Self {
//         Self { name, restrictions }
//     }
// }

pub(crate) enum Type {
    Path(Path),
    Generic(Generic),
    Int(Int),
    Float(Float),
    Never(kw::never),
    Unknown(kw::unknown),
}

impl Type {
    fn as_tid_token_stream(&self, generics: &[&GenericDefinition]) -> proc_macro2::TokenStream {
        match self {
            Type::Path(path) => path.to_tid_token_stream(generics),
            Type::Generic(_) => todo!(),
            Type::Int(int) => int.to_tid_token_stream(),
            Type::Float(float) => float.to_tid_token_stream(),
            Type::Never(never) => never.to_tid_token_stream(),
            Type::Unknown(unknown) => unknown.to_tid_token_stream(),
        }
    }
}

pub(crate) struct Int {
    ref_to: Option<Path>,
}

impl Int {
    pub(crate) fn new(ref_to: Option<Path>) -> Self {
        Self { ref_to }
    }

    fn to_tid_token_stream(&self) -> proc_macro2::TokenStream {
        let ref_to = match self.ref_to.as_ref() {
            Some(ref_to) => {
                let tid = ref_to.to_tid_token_stream(&[]);
                quote! {
                    Some(#tid)
                }
            }
            None => quote! {
                None
            },
        };
        quote! {
            {
                let ref_to = #ref_to;
                flux_typesystem::Insert::insert(tchk.tenv,
                    flux_span::WithSpan::file_span(
                        flux_typesystem::TypeKind::Int(
                            ref_to
                        ),
                        file_id,
                        span
                    )
                )
            }
        }
    }
}

pub(crate) struct Float {
    ref_to: Option<Path>,
}

impl Float {
    pub(crate) fn new(ref_to: Option<Path>) -> Self {
        Self { ref_to }
    }

    fn to_tid_token_stream(&self) -> proc_macro2::TokenStream {
        let ref_to = match self.ref_to.as_ref() {
            Some(ref_to) => {
                let tid = ref_to.to_tid_token_stream(&[]);
                quote! {
                    Some(#tid)
                }
            }
            None => quote! {
                None
            },
        };
        quote! {
            {
                let ref_to = #ref_to;
                flux_typesystem::Insert::insert(tchk.tenv,
                    flux_span::WithSpan::file_span(
                        flux_typesystem::TypeKind::Float(
                            ref_to
                        ),
                        file_id,
                        span
                    )
                )
            }
        }
    }
}

impl kw::never {
    fn to_tid_token_stream(&self) -> proc_macro2::TokenStream {
        quote! {
            {
                flux_typesystem::Insert::insert(tchk.tenv,
                    flux_span::WithSpan::file_span(
                        flux_typesystem::TypeKind::Never,
                        file_id,
                        span
                    )
                )
            }
        }
    }
}

impl kw::unknown {
    fn to_tid_token_stream(&self) -> proc_macro2::TokenStream {
        quote! {
            {
                flux_typesystem::Insert::insert(tchk.tenv,
                    flux_span::WithSpan::file_span(
                        flux_typesystem::TypeKind::Unknown,
                        file_id,
                        span
                    )
                )
            }
        }
    }
}

fn syn_path_to_tid_token_stream(
    tpath: &syn::TypePath,
    generics: &[&GenericDefinition],
) -> proc_macro2::TokenStream {
    let path = &tpath.path;
    assert_eq!(path.segments.len(), 1);
    let seg = path.segments.last().unwrap();

    let name = &seg.ident;
    let name_string = name.to_string();
    let name_str = name_string.as_str();
    if matches!(name_str, "A" | "B" | "C" | "D") {
        let mut restrictions = vec![];
        for generic in generics {
            if generic.name.as_str() == name_str {
                restrictions = generic
                    .restrictions
                    .iter()
                    .map(|restriction| restriction.to_trait_restriction_token_stream(generics))
                    .collect();
            }
        }
        return quote! {
            {
                let name_key = interner.get_or_intern(stringify!(#name));
                let restrictions = vec![#(#restrictions),*];
                flux_typesystem::Insert::insert(tchk.tenv,
                    flux_span::WithSpan::file_span(
                        flux_typesystem::TypeKind::Generic(
                            flux_typesystem::Generic::new(name_key, restrictions)
                        ),
                        file_id,
                        span
                    )
                )
            }
        };
    }

    let args = match &seg.arguments {
        syn::PathArguments::None => vec![],
        syn::PathArguments::AngleBracketed(angle_bracketed_args) => angle_bracketed_args
            .args
            .iter()
            .map(|arg| match arg {
                GenericArgument::Type(ty) => syn_ty_to_tid_token_stream(ty, generics),
                _ => todo!(), // Err
            })
            .collect(),
        syn::PathArguments::Parenthesized(_) => vec![], // err
    };
    let args = if args.is_empty() {
        quote! {}
    } else {
        quote! {
            // flux_typesystem::Insert::insert(tchk.tenv);
        }
    };

    let seg = seg.ident.to_string();
    quote! {
        {
            let seg_key = interner.get_or_intern(#seg);
            flux_typesystem::Insert::insert(tchk.tenv,
                flux_span::WithSpan::file_span(
                    flux_typesystem::TypeKind::Concrete(flux_typesystem::ConcreteKind::Path(
                        flux_typesystem::Path::new(vec![seg_key], vec![#args])
                    )),
                    file_id,
                    span
                )
            )
        }
    }
    // quote! {
    //     flux_typesystem::TypeId::new(0)
    // }
}

fn syn_ty_to_tid_token_stream(
    ty: &syn::Type,
    generics: &[&GenericDefinition],
) -> proc_macro2::TokenStream {
    match ty {
        syn::Type::Array(_) => todo!(),
        syn::Type::BareFn(_) => todo!(),
        syn::Type::Group(_) => todo!(),
        syn::Type::ImplTrait(_) => todo!(),
        syn::Type::Infer(_) => todo!(),
        syn::Type::Macro(_) => todo!(),
        syn::Type::Never(_) => todo!(),
        syn::Type::Paren(_) => todo!(),
        syn::Type::Path(path) => syn_path_to_tid_token_stream(path, generics),
        syn::Type::Ptr(_) => todo!(),
        syn::Type::Reference(_) => todo!(),
        syn::Type::Slice(_) => todo!(),
        syn::Type::TraitObject(_) => todo!(),
        syn::Type::Tuple(_) => todo!(),
        syn::Type::Verbatim(_) => todo!(),
        _ => todo!(),
    }
}

pub(crate) struct Path {
    name: Ident,
    args: Option<AngleBracketedGenericArguments>,
}

impl Path {
    pub(crate) fn new(name: Ident, args: Option<AngleBracketedGenericArguments>) -> Self {
        Self { name, args }
    }

    fn to_trait_restriction_token_stream(
        &self,
        generic_defs: &[&GenericDefinition],
    ) -> proc_macro2::TokenStream {
        let name = &self.name;
        let args = self.args.as_ref().map_or(vec![], |generics| {
            generics
                .args
                .iter()
                .map(|arg| match arg {
                    GenericArgument::Type(ty) => syn_ty_to_tid_token_stream(ty, generic_defs),
                    _ => todo!(),
                })
                .collect()
        });

        let name_str = name.to_string();
        quote! {
            {
                let trait_restriction_name = interner.get_or_intern(#name_str);
                flux_typesystem::TraitRestriction::new(
                    vec![trait_restriction_name],
                    vec![#(#args),*],
                )
            }
        }
    }

    fn to_tid_token_stream(&self, generic_defs: &[&GenericDefinition]) -> proc_macro2::TokenStream {
        let name = &self.name;
        let args = self.args.as_ref().map_or(vec![], |generics| {
            generics
                .args
                .iter()
                .map(|arg| match arg {
                    GenericArgument::Type(ty) => syn_ty_to_tid_token_stream(ty, generic_defs),
                    _ => todo!(),
                })
                .collect()
        });
        quote! {
            {
                let seg_key = interner.get_or_intern(stringify!(#name));
                let args = vec![#(#args),*];
                flux_typesystem::Insert::insert(tchk.tenv,
                    flux_span::WithSpan::file_span(
                        flux_typesystem::TypeKind::Concrete(flux_typesystem::ConcreteKind::Path(
                            flux_typesystem::Path::new(vec![seg_key], args)
                        )),
                        file_id,
                        span
                    )
                )
            }
        }
    }
}

pub(crate) struct Generic {
    name: Ident,
}

impl Generic {
    pub(crate) fn new(name: Ident) -> Self {
        Self { name }
    }
}

pub(crate) enum GenericName {
    A(kw::A),
    B(kw::B),
    C(kw::C),
    D(kw::D),
}

impl GenericName {
    fn as_str(&self) -> &'static str {
        match self {
            GenericName::A(_) => "A",
            GenericName::B(_) => "B",
            GenericName::C(_) => "C",
            GenericName::D(_) => "D",
        }
    }
}
