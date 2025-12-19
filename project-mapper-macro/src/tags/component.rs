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
use project_mapper_core::{
    available_config,
    runtime_config::{effect::common::EffectSrcConfigTrait, shared::ComponentConfig},
};
use quote::quote;
use syn::parse_macro_input;
use syn::{Error, ItemImpl, Type, TypePath, parse_quote};

#[derive(Clone)]
pub enum CompType {
    Input,
    Effect,
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
        CompType::Effect => quote! {},
    };

    let mut available_config_expr = if let Some(expr) = args.available_expr {
        expr.clone()
    } else {
        config_expr.clone()
    };
    let mut requires_refresh_expr = if let Some(expr) = args.requires_refresh_expr {
        quote! { #expr}
    } else {
        quote! { false }
    };

    let mut available_config_val = match comp_type {
        CompType::Input => quote! {
             Result::Ok(
                 Box::new(
                     project_mapper_runtime::project_mapper_core::available_config::config::AvailableConfigType::Input(
                         project_mapper_runtime::project_mapper_core::available_config::config::AvailableInputConfig::new(
                             project_mapper_runtime::components::marker::name_of(#available_config_expr),
                             project_mapper_runtime::components::marker::schemars::schema_for_value!(#available_config_expr),
                             #requires_refresh_expr
                         )
                     )
                 )
             )
        },
        CompType::Output => quote! {
            Result::Ok(
                Box::new(
                    project_mapper_runtime::project_mapper_core::available_config::config::AvailableConfigType::Output(
                        project_mapper_runtime::project_mapper_core::available_config::config::AvailableOutputConfig::new(
                            project_mapper_runtime::components::marker::name_of(#available_config_expr),
                            project_mapper_runtime::components::marker::schemars::schema_for_value!(#available_config_expr),
                            #requires_refresh_expr
                        )
                    )
                )
            )
        },
        CompType::Effect => quote! {},
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
                project_mapper_runtime::components::marker::ComponentMarker::<project_mapper_runtime::components::marker::DefaultConfig, project_mapper_runtime::components::marker::ConstructComponent, project_mapper_runtime::components::marker::AvailablaeConfig>::new(
                    #type_name,
                    #type_id,
                    ||{#config_val},
                    |cfg|{Result::Ok(Box::new(#ident::new(cfg)?))},
                    ||{#available_config_val},
                )
            }
        }
    );
    expanded.into()
}
