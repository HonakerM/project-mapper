use project_mapper_base::prelude::*;
use project_mapper_runtime::components::marker::{
    ComponentMarker, ConstructComponent, DefaultConfig,
};
//use project_mapper_base::input::test::{TestComponent, TestConfig};
pub fn main() {
    for marker in inventory::iter::<ComponentMarker<DefaultConfig, ConstructComponent>>() {
        println!("Components: {}", marker.name);
    }
    println!("Done!");
}
