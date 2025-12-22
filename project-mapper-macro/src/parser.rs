use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{Attribute, Error, ImplItem, ItemImpl, Token, Visibility};

mod kw {
    syn::custom_keyword!(config);
    syn::custom_keyword!(available);
    syn::custom_keyword!(requires_refresh);
    syn::custom_keyword!(schema);
    syn::custom_keyword!(src);
    syn::custom_keyword!(src_schema);
}

pub struct ImplInput {
    pub implementation: ItemImpl,
}

impl Parse for ImplInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attrs = Attribute::parse_outer(input)?;

        let ahead = input.fork();
        ahead.parse::<Visibility>()?;
        ahead.parse::<Option<Token![unsafe]>>()?;

        if ahead.peek(Token![impl]) {
            let mut item: ItemImpl = input.parse()?;
            if item.trait_.is_none() {
                let impl_token = item.impl_token;
                let ty = item.self_ty;
                let span = quote!(#impl_token #ty);
                let msg = "expected impl Trait for Type";
                return Err(Error::new_spanned(span, msg));
            }
            for assoc in &item.items {
                if let ImplItem::Const(assoc) = assoc {
                    let const_token = assoc.const_token;
                    let semi_token = assoc.semi_token;
                    let span = quote!(#const_token #semi_token);
                    let msg = "typetag trait with associated const is not supported yet";
                    return Err(Error::new_spanned(span, msg));
                }
            }
            attrs.extend(item.attrs);
            item.attrs = attrs;
            Ok(Self {
                implementation: item,
            })
        } else {
            Err(input.error("expected trait or impl block"))
        }
    }
}

pub struct ImplArgs {
    pub config_expr: syn::Expr,
    pub available_expr: Option<syn::Expr>,
    pub schema_expr: Option<syn::Expr>,
    pub src_expr: Option<syn::Expr>,
    pub src_schema_expr: Option<syn::Expr>,
    pub requires_refresh_expr: Option<syn::Expr>,
}
impl Parse for ImplArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut config_expr = None;
        let mut available_expr = None;
        let mut schema_expr = None;
        let mut requires_refresh_expr = None;
        let mut src_expr = None;
        let mut src_schema_expr = None;

        while !input.is_empty() {
            let lookahead = input.lookahead1();

            if lookahead.peek(kw::config) {
                let _: kw::config = input.parse()?;
                input.parse::<Token![=]>()?;

                // Always expect braces
                let content;
                syn::braced!(content in input);
                let value: syn::Expr = content.parse()?;

                if config_expr.is_some() {
                    return Err(syn::Error::new(input.span(), "duplicate argument: config"));
                }
                config_expr = Some(value);
            } else if lookahead.peek(kw::available) {
                let _: kw::available = input.parse()?;
                input.parse::<Token![=]>()?;

                // Always expect braces
                let content;
                syn::braced!(content in input);
                let value: syn::Expr = content.parse()?;

                if available_expr.is_some() {
                    return Err(syn::Error::new(
                        input.span(),
                        "duplicate argument: available",
                    ));
                }
                available_expr = Some(value);
            } else if lookahead.peek(kw::schema) {
                let _: kw::schema = input.parse()?;
                input.parse::<Token![=]>()?;

                // Always expect braces
                let content;
                syn::braced!(content in input);
                let value: syn::Expr = content.parse()?;

                if schema_expr.is_some() {
                    return Err(syn::Error::new(input.span(), "duplicate argument: schema"));
                }
                schema_expr = Some(value);
            } else if lookahead.peek(kw::src) {
                let _: kw::src = input.parse()?;
                input.parse::<Token![=]>()?;

                // Always expect braces
                let content;
                syn::braced!(content in input);
                let value: syn::Expr = content.parse()?;

                if src_expr.is_some() {
                    return Err(syn::Error::new(input.span(), "duplicate argument: src"));
                }
                src_expr = Some(value);
            } else if lookahead.peek(kw::src_schema) {
                let _: kw::src_schema = input.parse()?;
                input.parse::<Token![=]>()?;

                // Always expect braces
                let content;
                syn::braced!(content in input);
                let value: syn::Expr = content.parse()?;

                if src_schema_expr.is_some() {
                    return Err(syn::Error::new(
                        input.span(),
                        "duplicate argument: src_schema",
                    ));
                }
                src_schema_expr = Some(value);
            } else if lookahead.peek(kw::requires_refresh) {
                let _: kw::requires_refresh = input.parse()?;
                input.parse::<Token![=]>()?;

                // Always expect braces
                let content;
                syn::braced!(content in input);
                let value: syn::Expr = content.parse()?;

                if requires_refresh_expr.is_some() {
                    return Err(syn::Error::new(
                        input.span(),
                        "duplicate argument: requires_refresh",
                    ));
                }
                requires_refresh_expr = Some(value);
            } else {
                return Err(lookahead.error());
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        let config_expr = config_expr
            .ok_or_else(|| syn::Error::new(input.span(), "missing required argument: config"))?;

        Ok(Self {
            config_expr,
            available_expr,
            requires_refresh_expr,
            src_expr,
            schema_expr,
            src_schema_expr,
        })
    }
}
// pub struct TraitArgs {
//     pub comp_type: CompType,
// }

// // #[typetag::serde]
// // #[typetag::serde(name = "Tag")]
// impl Parse for TraitArgs {
//     fn parse(input: ParseStream) -> Result<Self> {
//         let comp_type = if input.is_empty() {
//             Err(Error::new(input.span(), "expected comp_type argument"))
//         } else {
//             input.parse::<kw::comp_type>()?;
//             input.parse::<Token![=]>()?;
//             let comp_type_str: String = input.parse()?;
//             let comp_type = match comp_type_str.as_str() {
//                 "Input" => Ok(CompType::Input),
//                 "Output" => Ok(CompType::Output),
//                 "Effect" => Ok(CompType::Effect(Box::new(DefaultSrcConfig { uid: DEFAULT_ID }))),
//                 _ => Err(Error::new(
//                     input.span(),
//                     "comp_type must be one of: Input, Output, Effect",
//                 )),
//             }?;
//             input.parse::<Option<Token![,]>>()?;
//             Ok(comp_type)
//         }?;
//         Ok(TraitArgs { comp_type })
//     }
// }
