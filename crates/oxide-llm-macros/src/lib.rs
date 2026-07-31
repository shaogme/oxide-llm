use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr, Meta, parse_macro_input};

/// Derive macro for generating `JSONSchema` from Rust types.
///
/// 从 Rust 类型生成 `JSONSchema` 的派生宏。
#[proc_macro_derive(Schema, attributes(schema, serde))]
pub fn derive_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // 1. Extract container metadata
    let container_meta = extract_container_meta(&input.attrs);
    let description = container_meta
        .description
        .or_else(|| extract_doc_comments(&input.attrs));

    // 2. Generate schema implementation based on type
    let expanded = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => {
                let schema_fields = fields.named.iter().filter_map(|f| {
                    let field_meta = extract_field_meta(&f.attrs);
                    if field_meta.skip {
                        return None;
                    }

                    let raw_field_name = f.ident.as_ref()?.to_string();
                    let field_name = field_meta.rename.unwrap_or_else(|| {
                        if let Some(ref rule) = container_meta.rename_all {
                            apply_rename_all(&raw_field_name, rule)
                        } else {
                            raw_field_name
                        }
                    });

                    let field_type = &f.ty;
                    let field_desc = field_meta
                        .description
                        .or_else(|| extract_doc_comments(&f.attrs))
                        .unwrap_or_default();
                    let format_setter = if let Some(fmt) = &field_meta.format {
                        quote! { .format(#fmt) }
                    } else {
                        quote! {}
                    };

                    let is_forced_required = field_meta.required;

                    Some(quote! {
                        {
                            let mut field_schema = <#field_type as ::oxide_llm::core::tool::model::Schema>::json_schema()
                                .description(#field_desc) #format_setter;

                            if #is_forced_required || !<#field_type as ::oxide_llm::core::tool::model::Schema>::is_optional() {
                                schema = schema.required_property(#field_name, field_schema);
                            } else {
                                schema = schema.property(#field_name, field_schema);
                            }
                        }
                    })
                });

                let desc_setter = if let Some(desc) = description {
                    quote! { .description(#desc) }
                } else {
                    quote! {}
                };

                quote! {
                    impl ::oxide_llm::core::tool::model::Schema for #name {
                        fn json_schema() -> ::oxide_llm::core::tool::model::JSONSchema {
                            #[allow(unused_mut)]
                            let mut schema = ::oxide_llm::core::tool::model::JSONSchema::object()
                                #desc_setter;

                            #(#schema_fields)*

                            schema
                        }
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let first_type = fields.unnamed.first().map(|f| &f.ty);
                let desc_setter = if let Some(desc) = description {
                    quote! { .description(#desc) }
                } else {
                    quote! {}
                };

                if let Some(ty) = first_type {
                    quote! {
                        impl ::oxide_llm::core::tool::model::Schema for #name {
                            fn json_schema() -> ::oxide_llm::core::tool::model::JSONSchema {
                                let mut schema = ::oxide_llm::core::tool::model::JSONSchema::array(
                                    <#ty as ::oxide_llm::core::tool::model::Schema>::json_schema()
                                ) #desc_setter;
                                schema
                            }
                        }
                    }
                } else {
                    quote! {
                        impl ::oxide_llm::core::tool::model::Schema for #name {
                            fn json_schema() -> ::oxide_llm::core::tool::model::JSONSchema {
                                let mut schema = ::oxide_llm::core::tool::model::JSONSchema::object()
                                    #desc_setter;
                                schema
                            }
                        }
                    }
                }
            }
            Fields::Unit => {
                let desc_setter = if let Some(desc) = description {
                    quote! { .description(#desc) }
                } else {
                    quote! {}
                };

                quote! {
                    impl ::oxide_llm::core::tool::model::Schema for #name {
                        fn json_schema() -> ::oxide_llm::core::tool::model::JSONSchema {
                            let mut schema = ::oxide_llm::core::tool::model::JSONSchema::object()
                                #desc_setter;
                            schema
                        }
                    }
                }
            }
        },
        Data::Enum(data_enum) => {
            // Check if all variants are unit variants
            let is_unit_enum = data_enum
                .variants
                .iter()
                .all(|v| matches!(v.fields, Fields::Unit));

            let desc_setter = if let Some(desc) = description {
                quote! { .description(#desc) }
            } else {
                quote! {}
            };

            if is_unit_enum {
                let enum_values: Vec<String> = data_enum
                    .variants
                    .iter()
                    .filter_map(|v| {
                        let meta = extract_field_meta(&v.attrs);
                        if meta.skip {
                            return None;
                        }
                        let raw_name = v.ident.to_string();
                        let name = meta.rename.unwrap_or_else(|| {
                            if let Some(ref rule) = container_meta.rename_all {
                                apply_rename_all(&raw_name, rule)
                            } else {
                                raw_name
                            }
                        });
                        Some(name)
                    })
                    .collect();

                quote! {
                    impl ::oxide_llm::core::tool::model::Schema for #name {
                        fn json_schema() -> ::oxide_llm::core::tool::model::JSONSchema {
                            let mut schema = ::oxide_llm::core::tool::model::JSONSchema::string()
                                .enum_values(vec![#(#enum_values),*])
                                #desc_setter;
                            schema
                        }
                    }
                }
            } else {
                // Non-unit enum (fallback to string or object schema)
                quote! {
                    impl ::oxide_llm::core::tool::model::Schema for #name {
                        fn json_schema() -> ::oxide_llm::core::tool::model::JSONSchema {
                            let mut schema = ::oxide_llm::core::tool::model::JSONSchema::object()
                                #desc_setter;
                            schema
                        }
                    }
                }
            }
        }
        Data::Union(_) => panic!("Unions are not supported for Schema derive"),
    };

    TokenStream::from(expanded)
}

#[derive(Default)]
struct ContainerMeta {
    rename_all: Option<String>,
    description: Option<String>,
}

fn extract_container_meta(attrs: &[syn::Attribute]) -> ContainerMeta {
    let mut meta_info = ContainerMeta::default();

    for attr in attrs {
        if attr.path().is_ident("schema") || attr.path().is_ident("serde") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename_all") {
                    let value = meta.value()?;
                    let s: LitStr = value.parse()?;
                    meta_info.rename_all = Some(s.value());
                    Ok(())
                } else if meta.path.is_ident("description") {
                    let value = meta.value()?;
                    let s: LitStr = value.parse()?;
                    meta_info.description = Some(s.value());
                    Ok(())
                } else {
                    Ok(())
                }
            });
        }
    }

    meta_info
}

#[derive(Default)]
struct FieldMeta {
    rename: Option<String>,
    skip: bool,
    description: Option<String>,
    format: Option<String>,
    required: bool,
}

fn extract_field_meta(attrs: &[syn::Attribute]) -> FieldMeta {
    let mut meta_info = FieldMeta::default();

    for attr in attrs {
        if attr.path().is_ident("schema") || attr.path().is_ident("serde") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    let value = meta.value()?;
                    let s: LitStr = value.parse()?;
                    meta_info.rename = Some(s.value());
                    Ok(())
                } else if meta.path.is_ident("skip") {
                    meta_info.skip = true;
                    Ok(())
                } else if meta.path.is_ident("description") {
                    let value = meta.value()?;
                    let s: LitStr = value.parse()?;
                    meta_info.description = Some(s.value());
                    Ok(())
                } else if meta.path.is_ident("format") {
                    let value = meta.value()?;
                    let s: LitStr = value.parse()?;
                    meta_info.format = Some(s.value());
                    Ok(())
                } else if meta.path.is_ident("required") {
                    meta_info.required = true;
                    Ok(())
                } else {
                    Ok(())
                }
            });
        }
    }

    meta_info
}

fn extract_doc_comments(attrs: &[syn::Attribute]) -> Option<String> {
    let mut doc_lines = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("doc")
            && let Meta::NameValue(nv) = &attr.meta
            && let syn::Expr::Lit(expr_lit) = &nv.value
            && let syn::Lit::Str(lit_str) = &expr_lit.lit
        {
            let value = lit_str.value();
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                doc_lines.push(trimmed.to_string());
            }
        }
    }

    if doc_lines.is_empty() {
        None
    } else {
        Some(doc_lines.join("\n"))
    }
}

fn apply_rename_all(name: &str, rule: &str) -> String {
    match rule {
        "lowercase" => name.to_lowercase(),
        "UPPERCASE" => name.to_uppercase(),
        "snake_case" => to_snake_case(name),
        "camelCase" => to_camel_case(name, false),
        "PascalCase" => to_camel_case(name, true),
        "SCREAMING_SNAKE_CASE" => to_snake_case(name).to_uppercase(),
        "kebab-case" => to_snake_case(name).replace('_', "-"),
        "SCREAMING-KEBAB-CASE" => to_snake_case(name).replace('_', "-").to_uppercase(),
        _ => name.to_string(),
    }
}

fn to_snake_case(s: &str) -> String {
    let mut acc = String::new();
    let mut prev_is_lower = false;

    for c in s.chars() {
        if c.is_uppercase() {
            if prev_is_lower {
                acc.push('_');
            }
            acc.push(c.to_ascii_lowercase());
            prev_is_lower = false;
        } else {
            acc.push(c);
            prev_is_lower = c.is_alphanumeric();
        }
    }
    acc
}

fn to_camel_case(s: &str, uppercase_first: bool) -> String {
    let mut acc = String::new();
    let mut capitalize_next = uppercase_first;

    for c in s.chars() {
        if c == '_' || c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            acc.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            acc.push(c);
        }
    }
    acc
}
