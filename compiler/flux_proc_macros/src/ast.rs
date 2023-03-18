use proc_macro2::Ident;
use syn::{punctuated::Punctuated, Expr, LitStr, Token, Type, Visibility};

pub(super) struct DiagnosticEnum {
    pub(super) visibility: Visibility,
    pub(super) name: Ident,
    pub(super) variants: Punctuated<Variant, Token![,]>,
}

pub(super) struct Label {
    pub(super) field: Ident,
    pub(super) msg: LitStr,
    pub(super) exprs: Option<Punctuated<Expr, Token![,]>>,
}

pub(super) struct Labels {
    pub(super) field: Ident,
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
    pub(super) fields: Punctuated<Field, Token![,]>,
}

pub(super) struct Field {
    pub(super) attr: FieldAttribute,
    pub(super) name: Ident,
    pub(super) ty: Type,
}

pub(super) enum FieldAttribute {
    None,
    FileSpanned,
}

pub(super) enum ErrorAttribute {
    Location(Location),
    Primary(LitStr),
    Label(Label),
    Labels(Labels),
    Help(Help),
}

pub(super) struct Location {
    pub(super) field: Ident,
}
