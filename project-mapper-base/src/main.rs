use project_mapper_base::prelude::*;
use project_mapper_runtime::components::marker::{
    ComponentMarker, ConstructComponent, DefaultConfig, Marker,
};
//use project_mapper_base::input::test::{TestComponent, TestConfig};
pub fn main() {
    for marker in inventory::iter::<Marker>() {
        println!("Components: {}", marker.name);
    }
    println!("Done!");

    //
    if let Err(error) = project_mapper_base::entrypoint::run_main() {
        panic!("{:#}", error);
    }
}
