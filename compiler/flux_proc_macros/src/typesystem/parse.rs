use syn::{
    braced, bracketed, parenthesized, parse::Parse, punctuated::Punctuated, token::Paren, Ident,
    Token,
};

use crate::typesystem::ast::{Generic, GenericDefinition};

use super::ast::{
    kw, Float, GenericName, Int, Path, TestSuite, Type, Unification, UnificationDirective,
    UnificationKind, WithDirective,
};

impl Parse for TestSuite {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let with_directive = if input.peek(kw::With) {
            Some(input.parse()?)
        } else {
            None
        };
        let unification = input.parse()?;
        Ok(Self::new(unification, with_directive))
    }
}

impl Parse for UnificationDirective {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _unify: kw::Unify = input.parse()?;
        let content;
        braced!(content in input);
        let unifications = content.parse_terminated(Unification::parse)?;
        Ok(Self::new(unifications))
    }
}

impl Parse for Unification {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let a = input.parse()?;
        let kind = input.parse()?;
        let b = input.parse()?;
        Ok(Self::new(a, kind, b))
    }
}

impl Parse for WithDirective {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _with: kw::With = input.parse()?;
        let content;
        braced!(content in input);
        let generics: Punctuated<GenericDefinition, Token![,]> =
            content.parse_terminated(GenericDefinition::parse)?;
        return Ok(Self::new(generics));
    }
}

impl Parse for GenericDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: GenericName = input.parse()?;
        let _colon: Token![:] = input.parse()?;
        let content;
        bracketed!(content in input);
        let restrictions: Punctuated<Path, Token![,]> = content.parse_terminated(Parse::parse)?;
        Ok(Self::new(name, restrictions))
    }
}

impl Parse for GenericName {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::A) {
            Ok(input.parse().map(Self::A)?)
        } else if lookahead.peek(kw::B) {
            Ok(input.parse().map(Self::B)?)
        } else if lookahead.peek(kw::C) {
            Ok(input.parse().map(Self::C)?)
        } else if lookahead.peek(kw::D) {
            Ok(input.parse().map(Self::D)?)
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for Path {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let args = if input.peek(Token![<]) {
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Self::new(name, args))
    }
}

impl Parse for Type {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::A)
            || lookahead.peek(kw::B)
            || lookahead.peek(kw::C)
            || lookahead.peek(kw::D)
        {
            input.parse().map(|name| Self::Generic(Generic::new(name)))
        } else if lookahead.peek(kw::int) {
            input.parse().map(|int: Int| Self::Int(int))
        } else if lookahead.peek(kw::float) {
            input.parse().map(|float: Float| Self::Float(float))
        } else if lookahead.peek(kw::never) {
            input.parse().map(Self::Never)
        } else if lookahead.peek(kw::unknown) {
            input.parse().map(Self::Unknown)
        } else if lookahead.peek(Ident) {
            Ok(Self::Path(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for Int {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _int: kw::int = input.parse()?;
        let ref_to = if input.peek(Paren) {
            let content;
            parenthesized!(content in input);
            Some(content.parse()?)
        } else {
            None
        };
        Ok(Self::new(ref_to))
    }
}

impl Parse for Float {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _float: kw::float = input.parse()?;
        let ref_to = if input.peek(Paren) {
            let content;
            parenthesized!(content in input);
            Some(content.parse()?)
        } else {
            None
        };
        Ok(Self::new(ref_to))
    }
}

impl Parse for UnificationKind {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![==]) {
            input.parse().map(|_: Token![==]| UnificationKind::Success)
        } else if lookahead.peek(Token![!=]) {
            input.parse().map(|_: Token![!=]| UnificationKind::Failure)
        } else {
            Err(lookahead.error())
        }
    }
}
