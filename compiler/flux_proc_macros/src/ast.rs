use std::collections::BTreeSet;

use proc_macro2::{Span, TokenStream};
use syn::{Attribute, DataEnum, DeriveInput, Generics, Ident, LitStr, Member, Type};

pub struct Enum<'a> {
    pub original: &'a DeriveInput,
    pub attrs: Attrs<'a>,
    pub ident: Ident,
    pub generics: &'a Generics,
    pub variants: Vec<Variant<'a>>,
}

pub struct Variant<'a> {
    pub original: &'a syn::Variant,
    pub attrs: Attrs<'a>,
    pub ident: Ident,
    pub fields: Vec<Field<'a>>,
}

pub struct Field<'a> {
    pub original: &'a syn::Field,
    pub attrs: Attrs<'a>,
    pub member: Member,
    pub ty: &'a Type,
    pub contains_generic: bool,
}

pub struct Attrs<'a> {
    pub display: Option<Display<'a>>,
    pub source: Option<&'a Attribute>,
    pub backtrace: Option<&'a Attribute>,
    pub from: Option<&'a Attribute>,
    pub transparent: Option<Transparent<'a>>,
}

#[derive(Clone)]
pub struct Display<'a> {
    pub original: &'a Attribute,
    pub fmt: LitStr,
    pub args: TokenStream,
    pub has_bonus_display: bool,
    pub implied_bounds: BTreeSet<(usize, Trait)>,
}

#[derive(Copy, Clone)]
pub struct Transparent<'a> {
    pub original: &'a Attribute,
    pub span: Span,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Trait {
    Debug,
    Display,
    Octal,
    LowerHex,
    UpperHex,
    Pointer,
    Binary,
    LowerExp,
    UpperExp,
}

impl<'a> Enum<'a> {
    pub(crate) fn from_syn(node: &'a DeriveInput, data: &'a DataEnum) -> Result<Self, ()> {
        todo!()
        // let attrs = attr::get(&node.attrs)?;
        // let scope = ParamsInScope::new(&node.generics);
        // let span = attrs.span().unwrap_or_else(Span::call_site);
        // let variants = data
        //     .variants
        //     .iter()
        //     .map(|node| {
        //         let mut variant = Variant::from_syn(node, &scope, span)?;
        //         if let display @ None = &mut variant.attrs.display {
        //             *display = attrs.display.clone();
        //         }
        //         if let Some(display) = &mut variant.attrs.display {
        //             display.expand_shorthand(&variant.fields);
        //         } else if variant.attrs.transparent.is_none() {
        //             variant.attrs.transparent = attrs.transparent;
        //         }
        //         Ok(variant)
        //     })
        //     .collect::<Result<_>>()?;
        // Ok(Enum {
        //     original: node,
        //     attrs,
        //     ident: node.ident.clone(),
        //     generics: &node.generics,
        //     variants,
        // })
    }
}
