use bevy_macro_utils::{derive_label, BevyManifest};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub fn derive_node_label_inner(input: TokenStream) -> syn::Result<TokenStream2> {
    let input: syn::DeriveInput = syn::parse(input)?;

    let bevy_ecs = BevyManifest::shared().get_path("bevy_ecs");
    let label_path = syn::parse2(quote! { ::bevy_seedling::prelude::NodeLabel }).unwrap();

    let ident = &input.ident;
    let component_derive = quote! {
        impl #bevy_ecs::component::Component for #ident {
            const STORAGE_TYPE: #bevy_ecs::component::StorageType = #bevy_ecs::component::StorageType::Table;
            type Mutability = #bevy_ecs::component::Immutable;

            #[allow(unused_variables)]
            fn on_insert() -> Option<#bevy_ecs::lifecycle::ComponentHook> {
                Some(|mut world: #bevy_ecs::world::DeferredWorld, context: #bevy_ecs::lifecycle::HookContext| {
                    let value = world.get::<Self>(context.entity).unwrap();
                    let interned = <Self as #label_path>::intern(value);

                    let mut labels = world
                        .get::<::bevy_seedling::node::label::NodeLabels>(context.entity)
                        .cloned()
                        .unwrap_or_default();

                    labels.insert(interned);

                    world
                        .commands()
                        .entity(context.entity)
                        .insert(labels);
                })
            }
        }
    };

    let label_derive: TokenStream2 = derive_label(input, "NodeLabel", &label_path).into();

    Ok(quote! {
        #component_derive
        #label_derive
    })
}

pub fn derive_pool_label_inner(input: TokenStream) -> syn::Result<TokenStream2> {
    let input: syn::DeriveInput = syn::parse(input)?;

    let bevy_ecs = BevyManifest::shared().get_path("bevy_ecs");
    let label_path = syn::parse2(quote! { ::bevy_seedling::prelude::PoolLabel }).unwrap();

    let ident = &input.ident;
    let component_derive = quote! {
        impl #bevy_ecs::component::Component for #ident {
            const STORAGE_TYPE: #bevy_ecs::component::StorageType = #bevy_ecs::component::StorageType::Table;
            type Mutability = #bevy_ecs::component::Immutable;

            #[allow(unused_variables)]
            fn on_insert() -> Option<#bevy_ecs::lifecycle::ComponentHook> {
                Some(|mut world: #bevy_ecs::world::DeferredWorld, context: #bevy_ecs::lifecycle::HookContext| {
                    let value = world.get::<Self>(context.entity).unwrap();
                    let container = ::bevy_seedling::pool::label::PoolLabelContainer::new(value, context.component_id);

                    world
                        .commands()
                        .entity(context.entity)
                        .insert(container);
                })
            }
        }
    };

    let label_derive: TokenStream2 = derive_label(input, "PoolLabel", &label_path).into();

    Ok(quote! {
        #component_derive
        #label_derive
    })
}
