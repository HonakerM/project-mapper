#[path = "./tags/mod.rs"]
mod tags;

#[path = "./parser.rs"]
mod parser;

#[path = "./utils.rs"]
mod utils;

use proc_macro::TokenStream;
use project_mapper_core::runtime_config::input::{InputComponentConfig, test::TestConfig};
use syn::parse_macro_input;

use crate::tags::component::CompType;

#[proc_macro_attribute]
pub fn input_component(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as parser::ImplInput);
    let args = parse_macro_input!(args as parser::ImplArgs);

    tags::component::process(
        CompType::Input,
        InputComponentConfig::default(Box::new(TestConfig { fps: 30 })),
        input,
        args,
    )
}
