use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{
    Attribute, Error, Generics, ImplItem, ItemImpl, ItemTrait, LitStr, Token, TraitItem, Type,
    TypeParamBound, Visibility, WherePredicate,
};

use crate::CompType;

mod kw {
    syn::custom_keyword!(tag);
    syn::custom_keyword!(content);
    syn::custom_keyword!(default_variant);
    syn::custom_keyword!(deny_unknown_fields);
    syn::custom_keyword!(comp_type);
}

pub struct Input {
    pub implementation: ItemImpl,
}

impl Parse for Input {
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
            Ok(Input {
                implementation: item,
            })
        } else {
            Err(input.error("expected trait or impl block"))
        }
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
