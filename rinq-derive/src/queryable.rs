// rinq-derive/src/queryable.rs
// Implementation of #[derive(Queryable)]

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, LitStr, Type};

// ── type classification ───────────────────────────────────────────────────────

enum FieldKind {
    Bool,
    Str,     // String or &str
    Numeric, // integer / float primitive
    Other,
}

fn classify_type(ty: &Type) -> FieldKind {
    match ty {
        Type::Path(tp) => {
            let last = tp
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default();
            match last.as_str() {
                "bool" => FieldKind::Bool,
                "String" => FieldKind::Str,
                "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64"
                | "i128" | "isize" | "f32" | "f64" => FieldKind::Numeric,
                _ => FieldKind::Other,
            }
        }
        Type::Reference(_) => FieldKind::Str,
        _ => FieldKind::Other,
    }
}

// ── name helpers ─────────────────────────────────────────────────────────────

/// "UserProfile" → "user_profile"
fn to_snake_case(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.extend(c.to_lowercase());
    }
    out
}

/// "age" → "Age", "first_name" → "FirstName"
fn to_pascal_case(s: &str) -> String {
    let mut out = String::new();
    let mut cap_next = true;
    for c in s.chars() {
        if c == '_' {
            cap_next = true;
        } else if cap_next {
            out.extend(c.to_uppercase());
            cap_next = false;
        } else {
            out.push(c);
        }
    }
    out
}

// ── attribute parsing ─────────────────────────────────────────────────────────

struct FieldAttrs {
    skip: bool,
    rename: Option<String>,
    key: bool,
}

fn parse_field_attrs(field: &syn::Field) -> FieldAttrs {
    let mut skip = false;
    let mut rename = None;
    let mut key = false;

    for attr in &field.attrs {
        if !attr.path().is_ident("queryable") {
            continue;
        }
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") {
                skip = true;
                Ok(())
            } else if meta.path.is_ident("rename") {
                let value = meta.value()?;
                let s: LitStr = value.parse()?;
                rename = Some(s.value());
                Ok(())
            } else if meta.path.is_ident("key") {
                key = true;
                Ok(())
            } else {
                Err(meta.error("unknown queryable attribute; expected `skip`, `rename`, or `key`"))
            }
        });
    }

    FieldAttrs { skip, rename, key }
}

// ── main derive entry point ───────────────────────────────────────────────────

pub fn derive_queryable(input: DeriveInput) -> TokenStream {
    let struct_name = &input.ident;
    let struct_name_str = struct_name.to_string();

    // Reject generics for now
    if !input.generics.params.is_empty() {
        return syn::Error::new_spanned(
            &input.generics,
            "Queryable does not support generic structs",
        )
        .to_compile_error();
    }

    let module_name = format_ident!("{}_fields", to_snake_case(&struct_name_str));

    let named_fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => f.named.iter().collect::<Vec<_>>(),
            _ => {
                return syn::Error::new_spanned(
                    struct_name,
                    "Queryable only supports structs with named fields",
                )
                .to_compile_error();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                struct_name,
                "Queryable can only be derived for structs",
            )
            .to_compile_error();
        }
    };

    let mut accessor_fns: Vec<TokenStream> = Vec::new();
    let mut predicate_items: Vec<TokenStream> = Vec::new();
    let mut default_key_fn: Option<TokenStream> = None;

    for field in &named_fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        let attrs = parse_field_attrs(field);

        if attrs.skip {
            continue;
        }

        // Accessor function name: by_{field} or by_{rename}
        let accessor_name = if let Some(ref rename) = attrs.rename {
            format_ident!("by_{}", rename)
        } else {
            format_ident!("by_{}", field_name)
        };

        // Predicate struct name: PascalCase of field name
        let field_name_str = field_name.to_string();
        let pred_struct = format_ident!("{}", to_pascal_case(&field_name_str));

        // Hygienic closure parameter to avoid name collisions
        let item_var = proc_macro2::Ident::new("__rinq_item", Span::mixed_site());

        // Accessor — always clones so it works with both Clone and Copy types
        accessor_fns.push(quote! {
            /// Returns a cloned copy of the `#field_name` field.
            /// Suitable as a key selector for `order_by` and `group_by`.
            #[inline]
            pub fn #accessor_name(item: &#struct_name) -> #field_type {
                ::std::clone::Clone::clone(&item.#field_name)
            }
        });

        // Default-key convenience method for #[queryable(key)] fields
        if attrs.key && default_key_fn.is_none() {
            accessor_fns.push(quote! {
                /// Returns the default query key (marked with `#[queryable(key)]`).
                #[inline]
                pub fn default_key(item: &#struct_name) -> #field_type {
                    ::std::clone::Clone::clone(&item.#field_name)
                }
            });
            default_key_fn = Some(quote! {}); // sentinel: only generate once
        }

        // Predicate struct + methods
        let kind = classify_type(field_type);
        let pred_impl = match kind {
            FieldKind::Bool => quote! {
                /// Typed predicate builder for the `#field_name` (`bool`) field.
                pub struct #pred_struct;
                impl #pred_struct {
                    /// Returns a predicate that is `true` when `#field_name` is `true`.
                    pub fn is_true() -> impl Fn(&#struct_name) -> bool {
                        |#item_var| #item_var.#field_name
                    }
                    /// Returns a predicate that is `true` when `#field_name` is `false`.
                    pub fn is_false() -> impl Fn(&#struct_name) -> bool {
                        |#item_var| !#item_var.#field_name
                    }
                }
            },

            FieldKind::Str => quote! {
                /// Typed predicate builder for the `#field_name` (string) field.
                pub struct #pred_struct;
                impl #pred_struct {
                    /// Returns a predicate that matches when `#field_name == s`.
                    pub fn eq(s: &str) -> impl Fn(&#struct_name) -> bool {
                        let __s = s.to_owned();
                        move |#item_var| #item_var.#field_name == __s
                    }
                    /// Returns a predicate that matches when `#field_name` contains `s`.
                    pub fn contains(s: &str) -> impl Fn(&#struct_name) -> bool {
                        let __s = s.to_owned();
                        move |#item_var| #item_var.#field_name.contains(__s.as_str())
                    }
                    /// Returns a predicate that matches when `#field_name` starts with `s`.
                    pub fn starts_with(s: &str) -> impl Fn(&#struct_name) -> bool {
                        let __s = s.to_owned();
                        move |#item_var| #item_var.#field_name.starts_with(__s.as_str())
                    }
                    /// Returns a predicate that matches when `#field_name` is empty.
                    pub fn is_empty() -> impl Fn(&#struct_name) -> bool {
                        |#item_var| #item_var.#field_name.is_empty()
                    }
                }
            },

            FieldKind::Numeric => quote! {
                /// Typed predicate builder for the `#field_name` (numeric) field.
                pub struct #pred_struct;
                impl #pred_struct {
                    /// Returns a predicate that is `true` when `#field_name > n`.
                    pub fn gt(n: #field_type) -> impl Fn(&#struct_name) -> bool {
                        move |#item_var| #item_var.#field_name > n
                    }
                    /// Returns a predicate that is `true` when `#field_name < n`.
                    pub fn lt(n: #field_type) -> impl Fn(&#struct_name) -> bool {
                        move |#item_var| #item_var.#field_name < n
                    }
                    /// Returns a predicate that is `true` when `#field_name == n`.
                    pub fn eq(n: #field_type) -> impl Fn(&#struct_name) -> bool {
                        move |#item_var| #item_var.#field_name == n
                    }
                    /// Returns a predicate that is `true` when `lo <= #field_name <= hi`.
                    pub fn between(lo: #field_type, hi: #field_type) -> impl Fn(&#struct_name) -> bool {
                        move |#item_var| #item_var.#field_name >= lo && #item_var.#field_name <= hi
                    }
                }
            },

            FieldKind::Other => quote! {
                /// Typed predicate builder for the `#field_name` field.
                pub struct #pred_struct;
                impl #pred_struct {
                    /// Returns a predicate that is `true` when `#field_name == val`.
                    pub fn eq(val: #field_type) -> impl Fn(&#struct_name) -> bool
                    where
                        #field_type: ::std::cmp::PartialEq + 'static,
                    {
                        move |#item_var| #item_var.#field_name == val
                    }
                }
            },
        };

        predicate_items.push(pred_impl);
    }

    quote! {
        impl #struct_name {
            #(#accessor_fns)*
        }

        /// Auto-generated predicate builders for [`#struct_name`] fields.
        ///
        /// Each sub-struct exposes typed factory methods that return closures
        /// suitable for use with [`QueryBuilder::where_`].
        pub mod #module_name {
            #[allow(unused_imports)]
            use super::*;

            #(#predicate_items)*
        }
    }
}
