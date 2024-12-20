use bevy_macro_utils::{derive_label, BevyManifest};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::format_ident;
use quote::quote;

pub fn derive_label_inner(input: TokenStream) -> syn::Result<TokenStream2> {
    let input: syn::DeriveInput = syn::parse(input)?;

    let bevy_ecs = BevyManifest::default().get_path("bevy_ecs");

    let ident = &input.ident;
    let component_derive = quote! {
        impl #bevy_ecs::component::Component for #ident {
            const STORAGE_TYPE: #bevy_ecs::component::StorageType = #bevy_ecs::component::StorageType::Table;

            fn register_required_components(
                requiree: #bevy_ecs::component::ComponentId,
                components: &mut #bevy_ecs::component::Components,
                storages: &mut #bevy_ecs::storage::Storages,
                required_components: &mut #bevy_ecs::component::RequiredComponents,
                inheritance_depth: u16,
            ) {
            }

            #[allow(unused_variables)]
            fn register_component_hooks(hooks: &mut #bevy_ecs::component::ComponentHooks) {}
        }
    };

    let mut dyn_eq_path = bevy_ecs.clone();
    dyn_eq_path.segments.push(format_ident!("schedule").into());
    dyn_eq_path.segments.push(format_ident!("DynEq").into());

    let label_path = syn::parse2(quote! { ::bevy_seedling::label::NodeLabel }).unwrap();
    let label_derive: TokenStream2 =
        derive_label(input, "NodeLabel", &label_path, &dyn_eq_path).into();

    Ok(quote! {
        #component_derive
        #label_derive
    })
}

// impl bevy_ecs::component::Component for EffectsBus
// where
//     Self: Send + Sync + 'static,
// {
//     const STORAGE_TYPE: bevy_ecs::component::StorageType = bevy_ecs::component::StorageType::Table;
//     fn register_required_components(
//         requiree: bevy_ecs::component::ComponentId,
//         components: &mut bevy_ecs::component::Components,
//         storages: &mut bevy_ecs::storage::Storages,
//         required_components: &mut bevy_ecs::component::RequiredComponents,
//         inheritance_depth: u16,
//     ) {
//     }
//     #[allow(unused_variables)]
//     fn register_component_hooks(hooks: &mut bevy_ecs::component::ComponentHooks) {}
// }
//
