use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, DeriveInput, GenericParam, Ident, Meta,
    MetaList,
};

#[proc_macro_derive(Handle, attributes(callback))]
pub fn create_handle_callbacks(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;

    let id = format_ident!("{ident}Handle");
    let generics = input.generics.clone();
    let mut generics_with_lifetime = input.generics.clone();
    let lifetime: GenericParam = parse_quote!('_a);
    generics_with_lifetime.params.push(lifetime.clone());

    let bounds = input.generics.where_clause.clone();
    let vis = input.vis;

    // A hashmap of callback field names as well as the metalist. Only fields with the `callback` attribute are included.
    let output = if let syn::Data::Struct(data) = input.data {
        let callbacks: Vec<(Ident, MetaList)> = data
            .fields
            .iter()
            // Only get fields with callback attributes
            .filter_map(|field| {
                let metas: Vec<Meta> = field
                    .attrs
                    .iter()
                    .filter_map(|a| a.parse_meta().ok())
                    .collect();
                // Find (if any) the attribute with the "callback" ident
                metas.iter().find_map(|meta| match meta {
                    Meta::List(meta_list) => {
                        match meta_list.path == (format_ident!("callback")).into() {
                            true => Some((field.ident.clone().unwrap(), meta_list.clone())),
                            false => None,
                        }
                    }
                    _ => None,
                })
            })
            .collect();

        let callback_idents: Vec<Ident> =
            callbacks.iter().map(|(ident, _)| ident.clone()).collect();
        let callback_types: Vec<Punctuated<_, _>> =
            callbacks.iter().map(|(_, ty)| ty.nested.clone()).collect();

        quote! {
            #vis trait #id #generics #bounds
            {
                #(
                    fn #callback_idents<F> (self, callback: F) -> Self
                    where
                        F: 'static + Fn(&mut Context, #callback_types);
                )*
            }

            impl #generics_with_lifetime #id #generics for Handle<#lifetime, #ident #generics> #bounds {
                #(
                    fn #callback_idents<F>(self, callback: F) -> Self
                    where
                        F: 'static + Fn(&mut Context, #callback_types) {
                            if let Some(view) = self.cx.views.get_mut(&self.entity) {
                                if let Some(down) = view.downcast_mut::<#ident #generics>() {
                                    down.#callback_idents = Some(Box::new(callback));
                                }
                            }
                            self
                        }
                )*
            }
        }
    } else {
        quote! {
            compile_error!("Only structs are supported.");
        }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(output)
}
