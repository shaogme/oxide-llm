use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr, Meta, parse_macro_input};

#[proc_macro_derive(Schema, attributes(schema))]
pub fn derive_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // 1. Extract struct description
    let description = extract_description(&input.attrs).unwrap_or_default();

    // 2. Generate fields schema implementation
    let fields_impl = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => {
                let schema_fields = fields.named.iter().map(|f| {
                    let field_name = &f.ident;
                    let field_type = &f.ty;
                    // Extract field description
                    let field_desc = extract_description(&f.attrs).unwrap_or_default();

                    quote! {
                        {
                            let field_schema = <#field_type as ::oxide_llm::core::tool::model::Schema>::json_schema()
                                .description(#field_desc);

                            if !<#field_type as ::oxide_llm::core::tool::model::Schema>::is_optional() {
                                schema = schema.required_property(stringify!(#field_name), field_schema);
                            } else {
                                schema = schema.property(stringify!(#field_name), field_schema);
                            }
                        }
                    }
                });

                quote! {
                    #(#schema_fields)*
                }
            }
            _ => panic!("Only named fields are supported for Schema derive"),
        },
        _ => panic!("Only structs are supported for Schema derive"),
    };

    // 3. Generate final impl block
    // Note: We use fully qualified path ::oxide_llm::core to refer to the core crate.
    let expanded = quote! {
        impl ::oxide_llm::core::tool::model::Schema for #name {
            fn json_schema() -> ::oxide_llm::core::tool::model::JSONSchema {
                #[allow(unused_mut)]
                let mut schema = ::oxide_llm::core::tool::model::JSONSchema::object()
                    .description(#description);

                #fields_impl

                schema
            }
        }
    };

    TokenStream::from(expanded)
}

/// Helper to extract description from #[schema(description = "...")] or doc comments
fn extract_description(attrs: &[syn::Attribute]) -> Option<String> {
    let mut doc_lines = Vec::new();
    let mut schema_desc = None;

    for attr in attrs {
        if attr.path().is_ident("schema") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("description") {
                    let value = meta.value()?;
                    let s: LitStr = value.parse()?;
                    schema_desc = Some(s.value());
                    Ok(())
                } else {
                    Err(meta.error("unsupported schema attribute"))
                }
            });
        } else if attr.path().is_ident("doc")
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

    schema_desc.or_else(|| {
        if doc_lines.is_empty() {
            None
        } else {
            Some(doc_lines.join("\n"))
        }
    })
}
