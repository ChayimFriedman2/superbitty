use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::*;

mod kw {
    syn::custom_keyword!(construct);
}

pub(super) type Constructor = Option<(Visibility, Ident)>;

pub(super) struct BitfieldsStruct {
    pub(super) attrs: Vec<Attribute>,
    pub(super) vis: Visibility,
    pub(super) struct_token: Token![struct],
    pub(super) ident: Ident,
    pub(super) base_ty: syn::Type,
    pub(super) generics: Generics,
    pub(super) fields: BitfieldsStructFields,
    pub(super) constructor: Constructor,
}

impl Parse for BitfieldsStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        let struct_token = input.parse()?;
        let ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let base_ty = input.parse()?;
        let generics = input.parse()?;
        let where_clause = if input.peek(Token![where]) { Some(input.parse()?) } else { None };
        let fields = input.parse()?;
        let constructor = if input.peek(kw::construct) {
            input.parse::<kw::construct>()?;
            input.parse::<Token![=]>()?;
            let vis = input.parse()?;
            input.parse::<Token![fn]>()?;
            let name = input.parse()?;
            let args;
            let parens = parenthesized!(args in input);
            if !args.is_empty() {
                return Err(syn::Error::new(
                    parens.span,
                    "the syntax is `construct = <visibility> fn <name>()`, with no arguments",
                ));
            }
            input.parse::<Token![;]>()?;
            Some((vis, name))
        } else {
            None
        };
        Ok(Self {
            attrs,
            vis,
            struct_token,
            ident,
            base_ty,
            generics: Generics { where_clause, ..generics },
            fields,
            constructor,
        })
    }
}

pub(super) struct BitfieldsStructFields {
    pub(super) fields: Punctuated<BitfieldsStructField, Token![,]>,
}

impl Parse for BitfieldsStructFields {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        braced!(content in input);
        Ok(Self { fields: content.parse_terminated(BitfieldsStructField::parse)? })
    }
}

pub(super) struct BitfieldsStructField {
    pub(super) attrs: Vec<Attribute>,
    pub(super) vis: Visibility,
    pub(super) ident: Ident,
    pub(super) ty: syn::Type,
}

impl Parse for BitfieldsStructField {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        let ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        Ok(Self { attrs, vis, ident, ty })
    }
}
