use std::any::type_name_of_val;

use project_mapper_base::{effect::balance::BalanceConfig, prelude::*};
use project_mapper_core::runtime_config::effect::common::EffectConfigTrait;
use project_mapper_runtime::components::marker::{
    ComponentMarker, ConstructComponent, DefaultConfig, Marker,
};
use serde_json::Value;
use utoipa::{
    OpenApi, ToSchema,
    openapi::{ComponentsBuilder, InfoBuilder, OpenApiBuilder},
};

/// Get just the schema of a struct as a serde_json::Value
fn get_schema_as_value<T: ToSchema>(
    val: T,
) -> (String, utoipa::openapi::RefOr<utoipa::openapi::Schema>) {
    let schema: utoipa::openapi::RefOr<utoipa::openapi::Schema> = T::schema();
    let schema_value = serde_json::to_value(&schema).expect("Failed to serialize schema");
    (type_name_of_val(&val).to_string(), schema)
}

fn main() {
    // Generate the OpenAPI specification as JSON
    let mut openapi = OpenApiBuilder::new()
        .info(
            InfoBuilder::new()
                .title("Project Mapper Runtime")
                .version("0.0.1")
                .build(),
        )
        .build();

    let (balance_name, balance_schema) = get_schema_as_value(BalanceConfig::default());

    let balance_json =
        serde_json::to_value(&BalanceConfig::default() as &dyn EffectConfigTrait).unwrap();
    let balance_name = balance_json
        .get("type")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    println!("BalanceConfig Schema JSON: {}\n", balance_json);

    // Ensure components exist
    if openapi.components.is_none() {
        openapi.components = Some(ComponentsBuilder::new().build());
    }

    // Add the schema
    if let Some(ref mut components) = openapi.components {
        components
            .schemas
            .insert(balance_name.to_string(), balance_schema);
    }

    println!("OpenAPI Specification:");
    println!("{}\n", openapi.to_pretty_json().unwrap());

    // You can also generate YAML
    // let openapi_yaml = ApiDoc::openapi().to_yaml().unwrap();

    if let Err(err) = project_mapper_base::entrypoint::run_main() {
        panic!("{}", err)
    };
}
