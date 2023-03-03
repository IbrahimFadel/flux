use proc_macro2::Ident;
use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, LitStr, Result, Token, Type, Visibility,
};

mod kw {
    use syn::custom_keyword;

    custom_keyword!(error);
    custom_keyword!(location);
    custom_keyword!(primary);
    custom_keyword!(label);
    custom_keyword!(help);

    custom_keyword!(with);

    custom_keyword!(normal);
    custom_keyword!(map);
    custom_keyword!(map_inner);
    custom_keyword!(from_file_span);
}

pub(super) struct DiagnosticEnum {
    pub(super) name: Ident,
    pub(super) variants: Punctuated<Variant, Token![,]>,
}

pub(super) enum AccessKind {
    Normal(Ident),
    Map(Ident),
    MapInner(Ident),
    FromFileSpan(Ident),
}

pub(super) struct Label {
    pub(super) access_kind: AccessKind,
    pub(super) msg: LitStr,
    pub(super) exprs: Option<Punctuated<Expr, Token![,]>>,
}

pub(super) struct Help {
    pub(super) msg: LitStr,
    pub(super) exprs: Option<Punctuated<Expr, Token![,]>>,
}

pub(super) struct Variant {
    pub(super) error_attributes: Punctuated<ErrorAttribute, Token![,]>,
    pub(super) name: Ident,
    pub(super) fields: Punctuated<(Ident, Type), Token![,]>,
}

pub(super) enum ErrorAttribute {
    Location(Location),
    Primary(LitStr),
    Label(Label),
    Help(Help),
}

pub(super) struct Location {
    pub(super) access_kind: AccessKind,
}

impl Parse for DiagnosticEnum {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let content;
        let _vis: Visibility = input.parse()?;
        let _enum_token: Token![enum] = input.parse()?;
        let name = input.parse()?;
        let _brace_token = braced!(content in input);
        let variants: Punctuated<Variant, Token![,]> = content.parse_terminated(Variant::parse)?;
        Ok(Self { name, variants })
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
        let fields: Punctuated<(Ident, Type), Token![,]> = content.parse_terminated(|input| {
            let name = input.parse()?;
            let _colon: Token![:] = input.parse()?;
            let ty = input.parse()?;
            Ok((name, ty))
        })?;
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
        let access_kind = input.parse()?;
        Ok(Location { access_kind })
    }
}

// impl Parse for AccessKind {
//     fn parse(input: ParseStream) -> Result<Self> {
//         let content;
//         let _bracket = bracketed!(content in input);
//         if content.peek(kw::map_inner) {
//             let _map_inner: kw::map_inner = content.parse()?;
//             Ok(AccessKind::MapInner)
//         } else if content.peek(kw::map) {
//             let _map_inner: kw::map = content.parse()?;
//             Ok(AccessKind::Map)
//         } else {
//             Err(content.error("invalid location access kind"))
//         }
//     }
// }

impl Parse for Label {
    fn parse(input: ParseStream) -> Result<Self> {
        let _label: kw::label = input.parse()?;
        let _eq: Token![=] = input.parse()?;
        let access_kind = input.parse()?;
        let msg: LitStr = input.parse()?;
        let exprs = if input.peek(kw::with) {
            let _with: kw::with = input.parse()?;
            let content;
            let _paren = parenthesized!(content in input);
            Some(content.parse_terminated(Expr::parse)?)
        } else {
            None
        };
        Ok(Label {
            access_kind,
            msg,
            exprs,
        })
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

impl Parse for AccessKind {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let _bracket = bracketed!(content in input);
        if content.peek(kw::map_inner) {
            let content1;
            let _map_inner: kw::map_inner = content.parse()?;
            let _parens = parenthesized!(content1 in content);
            let field: Ident = content1.parse()?;
            Ok(AccessKind::MapInner(field))
        } else if content.peek(kw::map) {
            let content1;
            let _map_inner: kw::map = content.parse()?;
            let _parens = parenthesized!(content1 in content);
            let field: Ident = content1.parse()?;
            Ok(AccessKind::Map(field))
        } else if content.peek(kw::normal) {
            let content1;
            let _map_inner: kw::normal = content.parse()?;
            let _parens = parenthesized!(content1 in content);
            let field: Ident = content1.parse()?;
            Ok(AccessKind::Normal(field))
        } else if content.peek(kw::from_file_span) {
            let content1;
            let _map_inner: kw::from_file_span = content.parse()?;
            let _parens = parenthesized!(content1 in content);
            let field: Ident = content1.parse()?;
            Ok(AccessKind::FromFileSpan(field))
        } else {
            Err(input.error("invalid location access kind"))
        }
    }
}
