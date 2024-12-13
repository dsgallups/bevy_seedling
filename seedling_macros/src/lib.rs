extern crate proc_macro;

use bevy_macro_utils::{
    derive_label,
    fq_std::{FQOption, FQResult},
    BevyManifest,
};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::format_ident;
use quote::quote;
use syn::parse_macro_input;
use syn::spanned::Spanned;

#[proc_macro_derive(NodeLabel)]
pub fn derive_node_label(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    let mut dyn_eq_path = BevyManifest::default().get_path("bevy_ecs");
    dyn_eq_path.segments.push(format_ident!("schedule").into());
    dyn_eq_path.segments.push(format_ident!("DynEq").into());

    let label_path = syn::parse2(quote! { ::bevy_seedling::label::NodeLabel }).unwrap();

    derive_label(input, "NodeLabel", &label_path, &dyn_eq_path)
}

fn derive_param_inner(input: TokenStream) -> syn::Result<TokenStream2> {
    let input: syn::DeriveInput = syn::parse(input)?;
    let identifier = &input.ident;

    let syn::Data::Struct(data) = &input.data else {
        return Err(syn::Error::new(
            input.span(),
            "`AudioParam` can only be derived on structs",
        ));
    };

    let syn::Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new(
            input.span(),
            "`AudioParam` can only be derived on structs with named fields",
        ));
    };

    let fields: Vec<_> = fields
        .named
        .iter()
        .map(|f| (f.ident.as_ref().unwrap(), &f.ty))
        .collect();

    let messages = fields.iter().enumerate().map(|(i, (identifier, _))| {
        let index = i as u16;
        quote! {
            self.#identifier.to_messages(&cmp.#identifier, messages, path.with(#index));
        }
    });

    let patches = fields.iter().enumerate().map(|(i, (identifier, _))| {
        let index = i as u16;
        quote! {
            #FQOption::Some(#index) => self.#identifier.patch(data, &path[1..])
        }
    });

    let ticks = fields.iter().map(|(identifier, _)| {
        quote! {
            self.#identifier.tick(time);
        }
    });

    let (impl_generics, ty_generics, where_generics) = input.generics.split_for_impl();

    let mut where_generics = where_generics.cloned().unwrap_or_else(|| syn::WhereClause {
        where_token: Default::default(),
        predicates: Default::default(),
    });

    let param_path = quote! { ::bevy_seedling::param };

    for (_, ty) in &fields {
        where_generics
            .predicates
            .push(syn::parse2(quote! { #ty: #param_path::AudioParam }).unwrap());
    }

    Ok(quote! {
        impl #impl_generics #param_path::AudioParam for #identifier #ty_generics #where_generics {
            fn to_messages(&self, cmp: &Self, messages: &mut Messages, path: ParamPath) {
                #(#messages)*
            }

            fn patch(&mut self, data: #param_path::MessageData, path: &[u16]) -> #FQResult<(), #param_path::PatchError> {
                match path.first() {
                    #(#patches,)*
                    _ => #FQResult::Err(#param_path::PatchError::InvalidPath),
                }
            }

            fn tick(&mut self, time: firewheel::clock::ClockSeconds) {
                #(#ticks)*
            }
        }
    })
}

#[proc_macro_derive(AudioParam)]
pub fn derive_audio_param(input: TokenStream) -> TokenStream {
    derive_param_inner(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
