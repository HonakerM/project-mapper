// OpenAPI 3.1 Schema Object and related types
// Based on: https://spec.openapis.org/oas/v3.1.0
//
// The Schema Object in OpenAPI 3.1 is based on JSON Schema Draft 2020-12
// with additional OpenAPI-specific keywords.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The Schema Object allows the definition of input and output data types.
/// These types can be objects, but also primitives and arrays.
/// This object is a superset of JSON Schema Specification Draft 2020-12.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    // ==================== JSON Schema Core Keywords ====================
    /// A URI that identifies the schema
    #[serde(rename = "$id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The JSON Schema dialect URI
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    /// A reference to another schema
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub ref_: Option<String>,

    /// Dynamic reference for recursive schemas
    #[serde(rename = "$dynamicRef", skip_serializing_if = "Option::is_none")]
    pub dynamic_ref: Option<String>,

    /// Dynamic anchor for recursive schemas
    #[serde(rename = "$dynamicAnchor", skip_serializing_if = "Option::is_none")]
    pub dynamic_anchor: Option<String>,

    /// Anchor for schema identification
    #[serde(rename = "$anchor", skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,

    /// Schema vocabulary URIs
    #[serde(rename = "$vocabulary", skip_serializing_if = "Option::is_none")]
    pub vocabulary: Option<HashMap<String, bool>>,

    /// Schema comments (not exposed in output)
    #[serde(rename = "$comment", skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,

    /// Schema definitions/reusable schemas
    #[serde(rename = "$defs", skip_serializing_if = "Option::is_none")]
    pub defs: Option<HashMap<String, Box<Schema>>>,

    // ==================== JSON Schema Validation Keywords ====================
    /// The type of the schema. In OpenAPI 3.1, this can be a single type or an array of types
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<SchemaType>,

    /// Enumeration of possible values
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_: Option<Vec<serde_json::Value>>,

    /// Constant value (must be this exact value)
    #[serde(rename = "const", skip_serializing_if = "Option::is_none")]
    pub const_: Option<serde_json::Value>,

    // ==================== Numeric Validation Keywords ====================
    /// Must be a multiple of this value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<f64>,

    /// Maximum value (inclusive by default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,

    /// Whether maximum is exclusive
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_maximum: Option<NumberOrBool>,

    /// Minimum value (inclusive by default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,

    /// Whether minimum is exclusive
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_minimum: Option<NumberOrBool>,

    // ==================== String Validation Keywords ====================
    /// Maximum length of a string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,

    /// Minimum length of a string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,

    /// Regular expression pattern (ECMA-262)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    // ==================== Array Validation Keywords ====================
    /// Maximum number of items in array
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,

    /// Minimum number of items in array
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,

    /// Whether all items must be unique
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_items: Option<bool>,

    /// Maximum number of items that `contains` can match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_contains: Option<usize>,

    /// Minimum number of items that `contains` can match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_contains: Option<usize>,

    // ==================== Object Validation Keywords ====================
    /// Maximum number of properties in object
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<usize>,

    /// Minimum number of properties in object
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<usize>,

    /// Array of required property names
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    /// Schema for properties that depend on other properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependent_required: Option<HashMap<String, Vec<String>>>,

    // ==================== Schema Composition Keywords ====================
    /// Must match all of these schemas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_of: Option<Vec<ReferenceOr<Schema>>>,

    /// Must match any of these schemas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<ReferenceOr<Schema>>>,

    /// Must match exactly one of these schemas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one_of: Option<Vec<ReferenceOr<Schema>>>,

    /// Must not match this schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not: Option<Box<ReferenceOr<Schema>>>,

    // ==================== Conditional Application Keywords ====================
    /// Conditional schema application
    #[serde(rename = "if", skip_serializing_if = "Option::is_none")]
    pub if_: Option<Box<ReferenceOr<Schema>>>,

    /// Schema to apply if `if` is valid
    #[serde(rename = "then", skip_serializing_if = "Option::is_none")]
    pub then: Option<Box<ReferenceOr<Schema>>>,

    /// Schema to apply if `if` is invalid
    #[serde(rename = "else", skip_serializing_if = "Option::is_none")]
    pub else_: Option<Box<ReferenceOr<Schema>>>,

    // ==================== Schema Application to Arrays ====================
    /// Schema for array items (single schema for all items)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<ReferenceOr<Schema>>>,

    /// Schema for prefix items (tuple validation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix_items: Option<Vec<ReferenceOr<Schema>>>,

    /// Schema that at least one item must match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contains: Option<Box<ReferenceOr<Schema>>>,

    // ==================== Schema Application to Objects ====================
    /// Object property schemas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, ReferenceOr<Schema>>>,

    /// Schema for properties matching regex patterns
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern_properties: Option<HashMap<String, ReferenceOr<Schema>>>,

    /// Schema for additional properties not matched by `properties` or `patternProperties`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<AdditionalProperties>,

    /// Object property names must match this schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_names: Option<Box<ReferenceOr<Schema>>>,

    /// Schemas that depend on the presence of certain properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependent_schemas: Option<HashMap<String, ReferenceOr<Schema>>>,

    // ==================== Unevaluated Locations ====================
    /// Schema for unevaluated items in arrays
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unevaluated_items: Option<Box<ReferenceOr<Schema>>>,

    /// Schema for unevaluated properties in objects
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unevaluated_properties: Option<Box<ReferenceOr<Schema>>>,

    // ==================== Metadata Keywords ====================
    /// Short title of the schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Description of the schema (CommonMark syntax MAY be used)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,

    /// Deprecated flag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,

    /// Whether the value can be read but not written (response only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,

    /// Whether the value can be written but not read (request only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_only: Option<bool>,

    /// Array of example values (replaces singular `example` from OpenAPI 3.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<Vec<serde_json::Value>>,

    // ==================== Content Keywords ====================
    /// Media type of the string content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_media_type: Option<String>,

    /// Content encoding (e.g., "base64", "base64url")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_encoding: Option<String>,

    /// Schema for decoded content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_schema: Option<Box<ReferenceOr<Schema>>>,

    // ==================== Format Keyword ====================
    /// Format hint for the value (e.g., "date-time", "email", "uuid", "int32", "int64")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    // ==================== OpenAPI-Specific Keywords ====================
    /// Adds support for polymorphism
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<Discriminator>,

    /// XML representation details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xml: Option<Xml>,

    /// Additional external documentation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocumentation>,

    /// Single example (deprecated in favor of `examples` array, kept for compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,

    // ==================== Nullable (OpenAPI 3.0 compatibility) ====================
    /// Whether null is a valid value (OpenAPI 3.0 compatibility)
    /// In OpenAPI 3.1, use `type: ["string", "null"]` instead
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable: Option<bool>,

    // ==================== Extension Fields ====================
    /// Specification extensions (fields starting with "x-")
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty", default)]
    pub extensions: HashMap<String, serde_json::Value>,
}

/// Schema type - can be a single type or an array of types (OpenAPI 3.1)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum SchemaType {
    /// Single type
    Single(SchemaTypeValue),
    /// Multiple types (JSON Schema 2020-12 feature)
    Multiple(Vec<SchemaTypeValue>),
}

/// Individual schema type values
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SchemaTypeValue {
    Null,
    Boolean,
    Object,
    Array,
    Number,
    String,
    Integer,
}

/// Either a number or boolean for exclusive minimum/maximum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum NumberOrBool {
    Number(f64),
    Bool(bool),
}

/// Additional properties can be a boolean or a schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum AdditionalProperties {
    /// Boolean flag
    Bool(bool),
    /// Schema for additional properties
    Schema(Box<ReferenceOr<Schema>>),
}

/// Reference or inline value
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ReferenceOr<T> {
    /// A reference to a component
    Reference {
        #[serde(rename = "$ref")]
        ref_: String,

        /// Summary of the reference (OpenAPI 3.1+)
        #[serde(skip_serializing_if = "Option::is_none")]
        summary: Option<String>,

        /// Description of the reference (OpenAPI 3.1+)
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
    /// An inline value
    Value(T),
}

/// Discriminator for polymorphism support
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Discriminator {
    /// The name of the property that holds the discriminator value
    pub property_name: String,

    /// Mapping between discriminator values and schema names/references
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapping: Option<HashMap<String, String>>,
}

/// XML representation details
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Xml {
    /// Replaces the name of the element/attribute
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// URI of the namespace definition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,

    /// Prefix for the name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    /// Whether the property is an XML attribute
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute: Option<bool>,

    /// Whether array elements are wrapped in a container element
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrapped: Option<bool>,

    /// Extension fields
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty", default)]
    pub extensions: HashMap<String, serde_json::Value>,
}

/// External documentation reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExternalDocumentation {
    /// Description of the external documentation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// URL of the external documentation
    pub url: String,

    /// Extension fields
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty", default)]
    pub extensions: HashMap<String, serde_json::Value>,
}

impl Schema {
    /// Create a new empty schema
    pub fn new() -> Self {
        Self {
            id: None,
            schema: None,
            ref_: None,
            dynamic_ref: None,
            dynamic_anchor: None,
            anchor: None,
            vocabulary: None,
            comment: None,
            defs: None,
            type_: None,
            enum_: None,
            const_: None,
            multiple_of: None,
            maximum: None,
            exclusive_maximum: None,
            minimum: None,
            exclusive_minimum: None,
            max_length: None,
            min_length: None,
            pattern: None,
            max_items: None,
            min_items: None,
            unique_items: None,
            max_contains: None,
            min_contains: None,
            max_properties: None,
            min_properties: None,
            required: None,
            dependent_required: None,
            all_of: None,
            any_of: None,
            one_of: None,
            not: None,
            if_: None,
            then: None,
            else_: None,
            items: None,
            prefix_items: None,
            contains: None,
            properties: None,
            pattern_properties: None,
            additional_properties: None,
            property_names: None,
            dependent_schemas: None,
            unevaluated_items: None,
            unevaluated_properties: None,
            title: None,
            description: None,
            default: None,
            deprecated: None,
            read_only: None,
            write_only: None,
            examples: None,
            content_media_type: None,
            content_encoding: None,
            content_schema: None,
            format: None,
            discriminator: None,
            xml: None,
            external_docs: None,
            example: None,
            nullable: None,
            extensions: HashMap::new(),
        }
    }

    /// Create a schema with a specific type
    pub fn with_type(type_: SchemaTypeValue) -> Self {
        Self {
            type_: Some(SchemaType::Single(type_)),
            ..Self::new()
        }
    }

    /// Create a string schema
    pub fn string() -> Self {
        Self::with_type(SchemaTypeValue::String)
    }

    /// Create an integer schema
    pub fn integer() -> Self {
        Self::with_type(SchemaTypeValue::Integer)
    }

    /// Create a number schema
    pub fn number() -> Self {
        Self::with_type(SchemaTypeValue::Number)
    }

    /// Create a boolean schema
    pub fn boolean() -> Self {
        Self::with_type(SchemaTypeValue::Boolean)
    }

    /// Create an array schema
    pub fn array(items: Schema) -> Self {
        Self {
            type_: Some(SchemaType::Single(SchemaTypeValue::Array)),
            items: Some(Box::new(ReferenceOr::Value(items))),
            ..Self::new()
        }
    }

    /// Create an object schema
    pub fn object() -> Self {
        Self::with_type(SchemaTypeValue::Object)
    }

    /// Create a reference to another schema
    pub fn reference(ref_path: impl Into<String>) -> ReferenceOr<Schema> {
        ReferenceOr::Reference {
            ref_: ref_path.into(),
            summary: None,
            description: None,
        }
    }
}

impl Default for Schema {
    fn default() -> Self {
        Self::new()
    }
}
