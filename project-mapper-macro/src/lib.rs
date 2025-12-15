#[path = "./tags/mod.rs"]
mod tags;

#[path = "./parser.rs"]
mod parser;

#[path = "./utils.rs"]
mod utils;

use proc_macro::TokenStream;
use project_mapper_core::runtime_config::input::InputComponentConfig;
use syn::parse_macro_input;

use crate::tags::component::CompType;

#[proc_macro_attribute]
pub fn input_component(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as parser::ImplInput);
    let args = parse_macro_input!(args as parser::ImplArgs);

    tags::component::process(CompType::Input, input, args)
}


#[proc_macro_attribute]
pub fn output_component(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as parser::ImplInput);
    let args = parse_macro_input!(args as parser::ImplArgs);

    tags::component::process(CompType::Output, input, args)
}
