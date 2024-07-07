use proc_macro2::Ident;
use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, LitStr, Result, Token,
};

use crate::diagnostics::ast::{
    DiagnosticEnum, ErrorAttribute, Field, FieldAttribute, Help, Label, Labels, Location, Variant,
};

mod kw {
    use syn::custom_keyword;

    custom_keyword!(error);
    custom_keyword!(location);
    custom_keyword!(primary);
    custom_keyword!(label);
    custom_keyword!(labels);
    custom_keyword!(help);

    custom_keyword!(with);
    custom_keyword!(at);

    custom_keyword!(filespanned);
}

impl Parse for DiagnosticEnum {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let content;
        let visibility = input.parse()?;
        let _enum_token: Token![enum] = input.parse()?;
        let name = input.parse()?;
        let _brace_token = braced!(content in input);
        let variants: Punctuated<Variant, Token![,]> = content.parse_terminated(Variant::parse)?;
        Ok(Self {
            visibility,
            name,
            variants,
        })
    }
}

impl Variant {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let content1;
        let _hashtag: Token![#] = input.parse()?;
        let _lbracket = bracketed!(content in input);
        let _error: kw::error = content.parse()?;
        let _lparen = parenthesized!(content1 in content);
        let error_attributes: Punctuated<ErrorAttribute, Token![,]> =
            content1.parse_terminated(ErrorAttribute::parse)?;
        let content;
        let name: Ident = input.parse()?;
        let _brace = braced!(content in input);
        let fields: Punctuated<Field, Token![,]> = content.parse_terminated(Field::parse)?;
        Ok(Variant {
            error_attributes,
            name,
            fields,
        })
    }
}

impl ErrorAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::location) {
            input.parse().map(ErrorAttribute::Location)
        } else if lookahead.peek(kw::primary) {
            let _primary: kw::primary = input.parse()?;
            let _eq: Token![=] = input.parse()?;
            input.parse().map(ErrorAttribute::Primary)
        } else if lookahead.peek(kw::label) {
            input.parse().map(ErrorAttribute::Label)
        } else if lookahead.peek(kw::labels) {
            input.parse().map(ErrorAttribute::Labels)
        } else if lookahead.peek(kw::help) {
            input.parse().map(ErrorAttribute::Help)
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for Location {
    fn parse(input: ParseStream) -> Result<Self> {
        let _loc: kw::location = input.parse()?;
        let _eq: Token![=] = input.parse()?;
        let field = input.parse()?;
        Ok(Location { field })
    }
}

impl Parse for Label {
    fn parse(input: ParseStream) -> Result<Self> {
        let _label: kw::label = input.parse()?;
        let _at: kw::at = input.parse()?;
        let field = input.parse()?;
        let _eq: Token![=] = input.parse()?;
        let msg: LitStr = input.parse()?;
        let exprs = if input.peek(kw::with) {
            let _with: kw::with = input.parse()?;
            let content;
            let _paren = parenthesized!(content in input);
            Some(content.parse_terminated(Expr::parse)?)
        } else {
            None
        };
        Ok(Label { field, msg, exprs })
    }
}

impl Parse for Labels {
    fn parse(input: ParseStream) -> Result<Self> {
        let _labels: kw::labels = input.parse()?;
        let _for: Token![for] = input.parse()?;
        let field = input.parse()?;
        let _eq: Token![=] = input.parse()?;
        let msg: LitStr = input.parse()?;
        let exprs = if input.peek(kw::with) {
            let _with: kw::with = input.parse()?;
            let content;
            let _paren = parenthesized!(content in input);
            Some(content.parse_terminated(Expr::parse)?)
        } else {
            None
        };
        Ok(Labels { field, msg, exprs })
    }
}

impl Parse for Help {
    fn parse(input: ParseStream) -> Result<Self> {
        let _label: kw::help = input.parse()?;
        let _eq: Token![=] = input.parse()?;
        let msg: LitStr = input.parse()?;
        let exprs = if input.peek(kw::with) {
            let _with: kw::with = input.parse()?;
            let content;
            let _paren = parenthesized!(content in input);
            Some(content.parse_terminated(Expr::parse)?)
        } else {
            None
        };
        Ok(Help { msg, exprs })
    }
}

impl Field {
    fn parse(input: ParseStream) -> Result<Self> {
        let attr = if input.peek(Token![#]) {
            input.parse()?
        } else {
            FieldAttribute::None
        };
        let name = input.parse()?;
        let _colon: Token![:] = input.parse()?;
        let ty = input.parse()?;
        Ok(Field { attr, name, ty })
    }
}

impl Parse for FieldAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let _hashtag: Token![#] = input.parse()?;
        let content;
        let _brace = bracketed!(content in input);
        let lookahead = content.lookahead1();
        if lookahead.peek(kw::filespanned) {
            let _filespanned: kw::filespanned = content.parse()?;
            Ok(FieldAttribute::FileSpanned)
        } else {
            Err(lookahead.error())
        }
    }
}
