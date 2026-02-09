use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, Expr, ExprLit, FnArg, ItemFn, Lit, Meta, Pat, Type, parse_macro_input};

#[proc_macro_attribute]
pub fn tool(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();
    let struct_name_str = to_pascal_case(&fn_name_str) + "Tool";
    let tool_struct_name = format_ident!("{}", struct_name_str);

    let fn_vis = &input_fn.vis;
    let fn_block = &input_fn.block;

    // Create a clean signature for the output function (stripping doc and tool attributes)
    let mut clean_sig = input_fn.sig.clone();
    for input in &mut clean_sig.inputs {
        if let FnArg::Typed(pat_type) = input {
            // Retain only attributes that are NOT `doc` or `tool`
            pat_type
                .attrs
                .retain(|attr| !attr.path().is_ident("doc") && !attr.path().is_ident("tool"));
        }
    }

    let tool_description = extract_doc_comments(&input_fn.attrs);

    let mut param_definitions = Vec::new();
    let mut param_parsing = Vec::new();
    let mut param_names = Vec::new();

    for arg in &input_fn.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            let pat = &pat_type.pat;
            let ty = &pat_type.ty;

            let arg_name = if let Pat::Ident(pat_ident) = &**pat {
                &pat_ident.ident
            } else {
                continue;
            };
            let arg_name_str = arg_name.to_string();

            let arg_desc = extract_doc_comments(&pat_type.attrs);

            let mut default_val = None;
            for attr in &pat_type.attrs {
                if attr.path().is_ident("tool") {
                    let _ = attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("default") {
                            let value = meta.value()?;
                            let s: Lit = value.parse()?;
                            if let Lit::Str(lit_str) = s {
                                default_val = Some(lit_str.value());
                            }
                            Ok(())
                        } else {
                            Err(meta.error("unsupported tool attribute"))
                        }
                    });
                }
            }

            let schema_type = map_type_to_schema(ty);

            // Construct schema logic
            let desc_setter = if let Some(d) = &arg_desc {
                quote! { s = s.description(#d); }
            } else {
                quote! {}
            };

            let default_setter = if let Some(d) = &default_val {
                quote! { s.default = Some(oxide_llm::reexports::serde_json::json!(#d)); }
            } else {
                quote! {}
            };

            param_definitions.push(quote! {
                {
                    let mut s = #schema_type;
                    #desc_setter
                    #default_setter
                    properties.insert(#arg_name_str.to_string(), s);
                    required.push(#arg_name_str.to_string());
                }
            });

            // Parsing logic
            let parse_logic = if let Some(def) = default_val {
                quote! {
                    let #arg_name: #ty = match args.get(#arg_name_str) {
                        Some(val) => oxide_llm::reexports::serde_json::from_value(val.clone())
                            .map_err(|e| format!("Arg '{}' type error: {}", #arg_name_str, e))?,
                        None => {
                             let def = #def;
                             if let Ok(v) = oxide_llm::reexports::serde_json::from_str::<#ty>(def) {
                                 v
                             } else if let Ok(v) = oxide_llm::reexports::serde_json::from_str::<#ty>(&format!("\"{}\"", def)) {
                                 v
                             } else {
                                 let v = oxide_llm::reexports::serde_json::Value::String(def.to_string());
                                 oxide_llm::reexports::serde_json::from_value(v)
                                    .map_err(|_| format!("Failed to parse default value '{}' for arg '{}'", def, #arg_name_str))?
                             }
                        }
                    };
                }
            } else {
                quote! {
                    let #arg_name: #ty = args.get(#arg_name_str)
                        .ok_or_else(|| format!("Missing argument: {}", #arg_name_str))
                        .and_then(|v| oxide_llm::reexports::serde_json::from_value(v.clone())
                        .map_err(|e| format!("Arg '{}' error: {}", #arg_name_str, e)))?;
                }
            };

            param_parsing.push(parse_logic);
            param_names.push(arg_name);
        }
    }

    let description_opt = if let Some(desc) = tool_description {
        quote! { Some(#desc.to_string()) }
    } else {
        quote! { None }
    };

    let is_async = input_fn.sig.asyncness.is_some();

    let is_result = if let syn::ReturnType::Type(_, ty) = &input_fn.sig.output {
        if let Type::Path(type_path) = &**ty {
            if let Some(segment) = type_path.path.segments.last() {
                segment.ident == "Result"
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    let result_handling = if is_result {
        quote! {
             match result {
                Ok(v) => Ok(vec![oxide_llm::core::message::ContentPart::Text { text: v.to_string() }]),
                Err(e) => Err(e.to_string()),
            }
        }
    } else {
        quote! {
            Ok(vec![oxide_llm::core::message::ContentPart::Text { text: result.to_string() }])
        }
    };

    let call_expr = if is_async {
        quote! { #fn_name(#(#param_names),*).await }
    } else {
        quote! { #fn_name(#(#param_names),*) }
    };

    let expanded = quote! {
        // Emit the function with cleaned signature
        #fn_vis #clean_sig #fn_block

        // Generate the tool struct
        #[derive(Clone)]
        #fn_vis struct #tool_struct_name;

        impl oxide_llm::core::tool::ToolRunnable for #tool_struct_name {
            fn definition(&self) -> oxide_llm::core::tool::Tool {
                use oxide_llm::core::tool::{Tool, JSONSchema};

                let mut properties = std::collections::BTreeMap::new();
                let mut required = Vec::new();

                #(#param_definitions)*

                let mut schema = JSONSchema::object();
                schema.properties = Some(properties);
                schema.required = Some(required);

                Tool::builder(#fn_name_str)
                    .description(#description_opt.unwrap_or_default())
                    .parameters(schema)
                    .build()
            }

            fn run(&self, args: oxide_llm::reexports::serde_json::Value) -> oxide_llm::core::tool::ToolFuture {
                Box::pin(async move {
                    let args = match args {
                        oxide_llm::reexports::serde_json::Value::Object(map) => map,
                        _ => return Err("Arguments must be a JSON object".to_string()),
                    };

                    #(#param_parsing)*

                    let result = #call_expr;

                    #result_handling


                })
            }
        }
    };

    TokenStream::from(expanded)
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

fn extract_doc_comments(attrs: &[Attribute]) -> Option<String> {
    let mut lines = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(nv) = &attr.meta {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                }) = &nv.value
                {
                    lines.push(s.value().trim().to_string());
                }
            }
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

fn map_type_to_schema(ty: &Type) -> proc_macro2::TokenStream {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let ident = segment.ident.to_string();
            match ident.as_str() {
                "String" | "str" | "char" => {
                    return quote! { oxide_llm::core::tool::JSONSchema::string() };
                }
                "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
                | "u128" | "usize" => {
                    return quote! { oxide_llm::core::tool::JSONSchema::integer() };
                }
                "f32" | "f64" => return quote! { oxide_llm::core::tool::JSONSchema::number() },
                "bool" => return quote! { oxide_llm::core::tool::JSONSchema::boolean() },
                _ => {}
            }
        }
    }
    quote! { oxide_llm::core::tool::JSONSchema::string() }
}
