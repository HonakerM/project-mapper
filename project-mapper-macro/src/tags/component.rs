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

#[derive(Clone, PartialEq)]
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
    let mut src_expr = if let Some(src) = args.src_expr {
        quote! {#src}
    } else {
        quote! {project_mapper_runtime::project_mapper_core::runtime_config::effect::common::DefaultSrcConfig::default()}
    };

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
        CompType::Effect => quote! {
            Result::Ok(
                Box::new(
                    project_mapper_runtime::project_mapper_core::runtime_config::effect::EffectComponentConfig::default(
                        Box::new(
                            #config_expr
                        ),
                        Box::new(
                            #src_expr
                        )
                    )
                )
            )
        },
    };

    let mut requires_refresh_expr = if let Some(expr) = args.requires_refresh_expr {
        quote! { #expr}
    } else {
        quote! { false }
    };

    let mut available_config_val = match comp_type {
        CompType::Input => {
            let mut available_config_expr = if let Some(expr) = args.available_expr {
                quote! {
                    #expr
                }
            } else {
                if let Some(schema) = args.schema_expr {
                    quote! {
                        project_mapper_runtime::project_mapper_core::available_config::input::AvailableInputConfig::from_input_config(
                            Box::new(#config_expr),
                            project_mapper_runtime::project_mapper_core::types::openapi::OpenAPISchema::try_from(#schema).unwrap(),
                            #requires_refresh_expr
                        )
                    }
                } else {
                    panic!(
                        "Input components must provide a schema or overal config expr for available config generation"
                    );
                }
            };

            quote! {
                Result::Ok(
                    Box::new(
                        project_mapper_runtime::project_mapper_core::available_config::config::AvailableConfigType::Input(
                            #available_config_expr,
                        )
                    )
                )
            }
        }
        CompType::Output => {
            let mut available_config_expr = if let Some(expr) = args.available_expr {
                quote! {
                    #expr
                }
            } else {
                if let Some(schema) = args.schema_expr {
                    quote! {
                        project_mapper_runtime::project_mapper_core::available_config::output::AvailableOutputConfig::from_output_config(
                            Box::new(#config_expr),
                            project_mapper_runtime::project_mapper_core::types::openapi::OpenAPISchema::try_from(#schema).unwrap(),
                            #requires_refresh_expr
                        )
                    }
                } else {
                    panic!(
                        "Output components must provide a schema or overal config expr for available config generation"
                    );
                }
            };

            quote! {
                Result::Ok(
                    Box::new(
                        project_mapper_runtime::project_mapper_core::available_config::config::AvailableConfigType::Output(
                            #available_config_expr
                        )
                    )
                )
            }
        }
        CompType::Effect => {
            let mut available_config_expr = if let Some(expr) = args.available_expr {
                quote! {
                    #expr
                }
            } else {
                if let Some(config_schema) = args.schema_expr {
                    let src = if let Some(src_schema) = args.src_schema_expr {
                        quote! {
                            #src_schema
                        }
                    } else {
                        quote! {project_mapper_runtime::project_mapper_core::runtime_config::effect::common::DefaultSrcConfig::openapi_schema().to_json_value()}
                    };

                    quote! {
                        project_mapper_runtime::project_mapper_core::available_config::effect::AvailableEffectConfig::from_effect_config(
                            Box::new(#config_expr),
                            Box::new(#src_expr),
                            project_mapper_runtime::project_mapper_core::types::openapi::OpenAPISchema::try_from(#config_schema).unwrap(),
                            project_mapper_runtime::project_mapper_core::types::openapi::OpenAPISchema::try_from(#src).unwrap(),
                            #requires_refresh_expr
                        )
                    }
                } else {
                    panic!("Effect Components must provide a schema or overall available config")
                }
            };

            quote! {
                Result::Ok(
                    Box::new(
                        project_mapper_runtime::project_mapper_core::available_config::config::AvailableConfigType::Effect(
                            #available_config_expr,
                        )
                    )
                )
            }
        }
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
