use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use syn::{parse::ParseStream, Attribute, Error};

use crate::ast::{Attrs, Display};

pub fn get(input: &[Attribute]) -> Result<Attrs, ()> {
    let mut attrs = Attrs {
        display: None,
        source: None,
        backtrace: None,
        from: None,
        transparent: None,
    };

    for attr in input {
        if attr.path.is_ident("error") {
            parse_error_attribute(&mut attrs, attr)?;
        }
    }

    Ok(attrs)
}

fn parse_error_attribute<'a>(attrs: &mut Attrs<'a>, attr: &'a Attribute) -> Result<(), Error> {
    attr.parse_args_with(|input: ParseStream| {
        let display = Display {
            original: attr,
            fmt: input.parse()?,
            args: parse_token_expr(input, false)?,
            has_bonus_display: false,
            implied_bounds: BTreeSet::new(),
        };
        if attrs.display.is_some() {
            return Err(Error::new_spanned(
                attr,
                "only one #[error(...)] attribute is allowed",
            ));
        }
        attrs.display = Some(display);
        Ok(())
    })
}

fn parse_token_expr(input: ParseStream, mut begin_expr: bool) -> Result<TokenStream, Error> {
    let mut tokens = Vec::new();
    while !input.is_empty() {
        if begin_expr && input.peek(Token![.]) {
            if input.peek2(Ident) {
                input.parse::<Token![.]>()?;
                begin_expr = false;
                continue;
            }
            if input.peek2(LitInt) {
                input.parse::<Token![.]>()?;
                let int: Index = input.parse()?;
                let ident = format_ident!("_{}", int.index, span = int.span);
                tokens.push(TokenTree::Ident(ident));
                begin_expr = false;
                continue;
            }
        }

        begin_expr = input.peek(Token![break])
            || input.peek(Token![continue])
            || input.peek(Token![if])
            || input.peek(Token![in])
            || input.peek(Token![match])
            || input.peek(Token![mut])
            || input.peek(Token![return])
            || input.peek(Token![while])
            || input.peek(Token![+])
            || input.peek(Token![&])
            || input.peek(Token![!])
            || input.peek(Token![^])
            || input.peek(Token![,])
            || input.peek(Token![/])
            || input.peek(Token![=])
            || input.peek(Token![>])
            || input.peek(Token![<])
            || input.peek(Token![|])
            || input.peek(Token![%])
            || input.peek(Token![;])
            || input.peek(Token![*])
            || input.peek(Token![-]);

        let token: TokenTree = if input.peek(token::Paren) {
            let content;
            let delimiter = parenthesized!(content in input);
            let nested = parse_token_expr(&content, true)?;
            let mut group = Group::new(Delimiter::Parenthesis, nested);
            group.set_span(delimiter.span);
            TokenTree::Group(group)
        } else if input.peek(token::Brace) {
            let content;
            let delimiter = braced!(content in input);
            let nested = parse_token_expr(&content, true)?;
            let mut group = Group::new(Delimiter::Brace, nested);
            group.set_span(delimiter.span);
            TokenTree::Group(group)
        } else if input.peek(token::Bracket) {
            let content;
            let delimiter = bracketed!(content in input);
            let nested = parse_token_expr(&content, true)?;
            let mut group = Group::new(Delimiter::Bracket, nested);
            group.set_span(delimiter.span);
            TokenTree::Group(group)
        } else {
            input.parse()?
        };
        tokens.push(token);
    }
    Ok(TokenStream::from_iter(tokens))
}
