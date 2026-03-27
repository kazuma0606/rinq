// rinq-derive/src/from.rs
// Implementation of #[derive(QueryableFrom)]

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, GenericArgument, PathArguments, Type};

/// Extract the inner type `T` from `Vec<T>`.
fn extract_vec_inner(ty: &Type) -> Option<&Type> {
    if let Type::Path(tp) = ty {
        let seg = tp.path.segments.last()?;
        if seg.ident != "Vec" {
            return None;
        }
        if let PathArguments::AngleBracketed(args) = &seg.arguments
            && let Some(GenericArgument::Type(inner)) = args.args.first()
        {
            return Some(inner);
        }
    }
    None
}

pub fn derive_queryable_from(input: DeriveInput) -> TokenStream {
    let struct_name = &input.ident;

    if !input.generics.params.is_empty() {
        return syn::Error::new_spanned(
            &input.generics,
            "QueryableFrom does not support generic structs",
        )
        .to_compile_error();
    }

    // Determine how to access the inner Vec and extract its element type.
    // We generate two access expressions:
    //   `from_expr`  — used in `From` impl where the parameter is named `__rinq_val`
    //   `self_expr`  — used in `IntoQuery` impl where the receiver is `self`
    let (from_expr, self_expr, inner_type) = match &input.data {
        Data::Struct(s) => match &s.fields {
            // Tuple struct: struct Foo(Vec<T>)
            Fields::Unnamed(f) => {
                if f.unnamed.len() != 1 {
                    return syn::Error::new_spanned(
                        struct_name,
                        "QueryableFrom requires a newtype struct with exactly one field (Vec<T>)",
                    )
                    .to_compile_error();
                }
                let field_ty = &f.unnamed[0].ty;
                match extract_vec_inner(field_ty) {
                    Some(inner) => (quote! { __rinq_val.0 }, quote! { self.0 }, inner.clone()),
                    None => {
                        return syn::Error::new_spanned(
                            field_ty,
                            "QueryableFrom: the single field must be Vec<T>",
                        )
                        .to_compile_error()
                    }
                }
            }
            // Named struct with a single field: struct Foo { items: Vec<T> }
            Fields::Named(f) => {
                if f.named.len() != 1 {
                    return syn::Error::new_spanned(
                        struct_name,
                        "QueryableFrom requires a struct with exactly one named field (Vec<T>)",
                    )
                    .to_compile_error();
                }
                let field = &f.named[0];
                let field_name = field.ident.as_ref().unwrap();
                match extract_vec_inner(&field.ty) {
                    Some(inner) => (
                        quote! { __rinq_val.#field_name },
                        quote! { self.#field_name },
                        inner.clone(),
                    ),
                    None => {
                        return syn::Error::new_spanned(
                            &field.ty,
                            "QueryableFrom: the single named field must be Vec<T>",
                        )
                        .to_compile_error()
                    }
                }
            }
            Fields::Unit => {
                return syn::Error::new_spanned(
                    struct_name,
                    "QueryableFrom cannot be derived for unit structs",
                )
                .to_compile_error()
            }
        },
        _ => {
            return syn::Error::new_spanned(
                struct_name,
                "QueryableFrom can only be derived for structs",
            )
            .to_compile_error()
        }
    };

    quote! {
        impl ::std::convert::From<#struct_name>
            for ::rinq::QueryBuilder<#inner_type, ::rinq::Initial>
        {
            fn from(__rinq_val: #struct_name) -> Self {
                ::rinq::QueryBuilder::from(#from_expr)
            }
        }

        impl ::rinq::IntoQuery for #struct_name {
            type Item = #inner_type;

            fn into_query(self) -> ::rinq::QueryBuilder<#inner_type, ::rinq::Initial> {
                ::rinq::QueryBuilder::from(#self_expr)
            }
        }
    }
}
