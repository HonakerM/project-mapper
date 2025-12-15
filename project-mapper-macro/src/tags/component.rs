use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::{
    parser::{ImplArgs, ImplInput},
    utils::type_name,
};
use anyhow::Result;
use inventory;
use proc_macro::TokenStream;
use project_mapper_core::runtime_config::{
    effect::common::EffectSrcConfigTrait, shared::ComponentConfig,
};
use quote::quote;
use syn::parse_macro_input;
use syn::{Error, ItemImpl, Type, TypePath, parse_quote};

#[derive(Clone)]
pub enum CompType {
    Input,
    Effect(Box<dyn EffectSrcConfigTrait>),
    Output,
}

pub fn process(comp_type: CompType, input: ImplInput, args: ImplArgs) -> TokenStream {
    let mut item_impl = input.implementation.clone();
    let mut expanded = quote! {
        #item_impl
    };
    let mut type_name = type_name(&input.implementation.self_ty).unwrap();

    let mut config_expr = args.config_expr.clone();

    let mut config_val = match comp_type {
        CompType::Input => quote! {
            Result::Ok(
                Box::new(
                    project_mapper_runtime::project_mapper_core::runtime_config::input::InputComponentConfig::default(
                        Box::new(
                            #config_expr
                        )
                    )
                )
            )
        },
        CompType::Output => quote! {
            Result::Ok(
                Box::new(
                    project_mapper_runtime::project_mapper_core::runtime_config::output::OutputComponentConfig::default(
                        Box::new(
                            #config_expr
                        )
                    )
                )
            )
        },
        CompType::Effect(_) => quote! {},
    };

    //config_expr
    // Access the self_ty field, which is a Box<Type>
    let ident = match &*item_impl.self_ty {
        Type::Path(type_path) => {
            // Extract the first segment of the path and get its identifier
            type_path.path.segments.first().unwrap().ident.clone()
        }
        // Handle other possible types if necessary (e.g., slices, references, etc.)
        _ => panic!("Unsupported self type for impl block"),
    };

    let type_id = quote! {
        ||{project_mapper_runtime::components::marker::type_id_of(#config_expr)}
    };

    expanded.extend(
        quote! {
            project_mapper_runtime::components::factory::inventory::submit! {
                project_mapper_runtime::components::marker::ComponentMarker::<project_mapper_runtime::components::marker::DefaultConfig, project_mapper_runtime::components::marker::ConstructComponent>::new(
                    #type_name,
                    #type_id,
                    ||{#config_val},
                    |cfg|{Result::Ok(Box::new(#ident::new(cfg)?))},
                )
            }
        }
    );
    expanded.into()
}
