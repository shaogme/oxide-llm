use oxide_llm::{
    core::tool::model::{JSONSchemaType, Schema},
    macros::Schema,
};

/// Struct doc comment.
///
/// 结构体文档注释。
#[derive(Schema)]
#[schema(rename_all = "camelCase")]
pub struct UserProfile {
    /// User name
    pub user_name: String,

    /// Age in years
    pub user_age: Option<u32>,

    #[schema(rename = "customEmail", format = "email")]
    pub email_address: String,

    #[schema(skip)]
    pub internal_id: u64,
}

#[derive(Schema)]
#[schema(rename_all = "lowercase")]
pub enum Role {
    Admin,
    User,
    #[schema(rename = "guest_user")]
    Guest,
    #[schema(skip)]
    InternalService,
}

// -----------------------------------------------------------------------------
// Test Structs for rename_all casing rules & field overrides
// -----------------------------------------------------------------------------

#[derive(Schema)]
#[schema(rename_all = "snake_case", description = "Explicit struct container description")]
pub struct SnakeCaseStruct {
    pub first_field_name: String,
    #[schema(description = "Explicit field description override")]
    pub second_field_name: u32,
}

#[derive(Schema)]
#[schema(rename_all = "UPPERCASE")]
pub struct UppercaseStruct {
    pub item_code: String,
}

#[derive(Schema)]
#[schema(rename_all = "PascalCase")]
pub struct PascalCaseStruct {
    pub user_first_name: String,
}

#[derive(Schema)]
#[schema(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct ScreamingSnakeStruct {
    pub max_retry_count: u32,
}

#[derive(Schema)]
#[schema(rename_all = "kebab-case")]
pub struct KebabCaseStruct {
    pub api_key_value: String,
}

#[derive(Schema)]
#[schema(rename_all = "SCREAMING-KEBAB-CASE")]
pub struct ScreamingKebabStruct {
    pub client_secret_id: String,
}

#[derive(Schema)]
pub struct ForcedRequiredStruct {
    #[schema(required)]
    pub optional_field: Option<String>,
}

#[derive(Schema)]
/// Tuple struct with single item doc
pub struct SingleItemTupleStruct(pub String);

#[derive(Schema)]
pub struct EmptyTupleStruct();

#[derive(Schema)]
/// Unit struct doc
pub struct UnitStruct;

#[derive(Schema)]
pub enum ComplexEnum {
    Text(String),
    Number { value: i64 },
}

// -----------------------------------------------------------------------------
// Unit Tests
// -----------------------------------------------------------------------------

#[test]
fn test_struct_schema_derivation() {
    let schema = UserProfile::json_schema();

    assert_eq!(schema.schema_type, Some(JSONSchemaType::Object));
    assert_eq!(
        schema.description.as_deref(),
        Some("Struct doc comment.\n结构体文档注释。")
    );

    let props = schema.properties.expect("Properties should exist");
    assert!(props.contains_key("userName"));
    assert!(props.contains_key("userAge"));
    assert!(props.contains_key("customEmail"));
    assert!(!props.contains_key("internal_id"));
    assert!(!props.contains_key("internalId"));

    let name_schema = props.get("userName").unwrap();
    assert_eq!(name_schema.description.as_deref(), Some("User name"));

    let req = schema.required.expect("Required list should exist");
    assert!(req.contains(&"userName".into()));
    assert!(req.contains(&"customEmail".into()));
    assert!(!req.contains(&"userAge".into()));

    let email_schema = props.get("customEmail").unwrap();
    assert_eq!(email_schema.format.as_deref(), Some("email"));
}

#[test]
fn test_unit_enum_schema_derivation() {
    let schema = Role::json_schema();

    assert_eq!(schema.schema_type, Some(JSONSchemaType::String));
    let enums = schema.enum_values.expect("Enum values should exist");
    assert_eq!(
        enums.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        vec!["admin", "user", "guest_user"]
    );
}

#[test]
fn test_rename_all_casing_rules() {
    let snake_props = SnakeCaseStruct::json_schema().properties.unwrap();
    assert!(snake_props.contains_key("first_field_name"));
    assert_eq!(
        SnakeCaseStruct::json_schema().description.as_deref(),
        Some("Explicit struct container description")
    );
    assert_eq!(
        snake_props.get("second_field_name").unwrap().description.as_deref(),
        Some("Explicit field description override")
    );

    let upper_props = UppercaseStruct::json_schema().properties.unwrap();
    assert!(upper_props.contains_key("ITEM_CODE"));

    let pascal_props = PascalCaseStruct::json_schema().properties.unwrap();
    assert!(pascal_props.contains_key("UserFirstName"));

    let screaming_snake_props = ScreamingSnakeStruct::json_schema().properties.unwrap();
    assert!(screaming_snake_props.contains_key("MAX_RETRY_COUNT"));

    let kebab_props = KebabCaseStruct::json_schema().properties.unwrap();
    assert!(kebab_props.contains_key("api-key-value"));

    let screaming_kebab_props = ScreamingKebabStruct::json_schema().properties.unwrap();
    assert!(screaming_kebab_props.contains_key("CLIENT-SECRET-ID"));
}

#[test]
fn test_forced_required_attr() {
    let schema = ForcedRequiredStruct::json_schema();
    let req = schema.required.expect("Required list should exist");
    assert!(req.contains(&"optional_field".into()));
}

#[test]
fn test_unnamed_fields_structs() {
    let single_tuple = SingleItemTupleStruct::json_schema();
    assert_eq!(single_tuple.schema_type, Some(JSONSchemaType::Array));
    assert_eq!(
        single_tuple.items.expect("Items schema should exist").schema_type,
        Some(JSONSchemaType::String)
    );
    assert_eq!(
        single_tuple.description.as_deref(),
        Some("Tuple struct with single item doc")
    );

    let empty_tuple = EmptyTupleStruct::json_schema();
    assert_eq!(empty_tuple.schema_type, Some(JSONSchemaType::Object));
}

#[test]
fn test_unit_struct_and_complex_enum() {
    let unit_struct = UnitStruct::json_schema();
    assert_eq!(unit_struct.schema_type, Some(JSONSchemaType::Object));
    assert_eq!(
        unit_struct.description.as_deref(),
        Some("Unit struct doc")
    );

    let complex_enum = ComplexEnum::json_schema();
    assert_eq!(complex_enum.schema_type, Some(JSONSchemaType::Object));
}

#[test]
fn test_primitive_and_collection_types() {
    let int_schema = i32::json_schema();
    assert_eq!(int_schema.schema_type, Some(JSONSchemaType::Integer));

    let float_schema = f64::json_schema();
    assert_eq!(float_schema.schema_type, Some(JSONSchemaType::Number));

    let vec_schema = Vec::<String>::json_schema();
    assert_eq!(vec_schema.schema_type, Some(JSONSchemaType::Array));
    assert_eq!(
        vec_schema.items.unwrap().schema_type,
        Some(JSONSchemaType::String)
    );
}

