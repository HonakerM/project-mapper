use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
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

use crate::parser::Input;

#[derive(Clone)]
pub enum CompType {
    Input,
    Effect(Box<dyn EffectSrcConfigTrait>),
    Output,
}

pub fn process(comp_type: CompType, config: impl ComponentConfig, input: Input) -> TokenStream {
    let mut item_impl = input.implementation.clone();
    let mut expanded = quote! {
        #item_impl
    };
    expanded.extend(
        quote! {
            const fn construct_local_default() -> project_mapper_runtime::components::marker::ComponentMarker<project_mapper_runtime::components::marker::DefaultConfig, project_mapper_runtime::components::marker::ConstructComponent> {
                project_mapper_runtime::components::marker::ComponentMarker::<project_mapper_runtime::components::marker::DefaultConfig, project_mapper_runtime::components::marker::ConstructComponent>::new(
                    "TestComponent",
                    ||{Result::Ok(Box::new(project_mapper_core::runtime_config::input::InputComponentConfig::default(Box::new(TestConfig::default()))))},
                    |cfg|{Result::Ok(Box::new(TestComponent::new(cfg)?))},
                )
            }
            inventory::submit! {
                construct_local_default()
            }
        }
    );
    expanded.into()
}
