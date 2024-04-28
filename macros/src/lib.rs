use proc_macro::TokenStream;
use quote::quote;
use syn::{ConstParam, GenericParam, TypeParam};

#[proc_macro_derive(Zonbi)]
pub fn derive_zonbi(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let name = &ast.ident;
    let generics = ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut zonbi_lifetimes = quote!();
    for ig in &generics.params {
        match ig {
            GenericParam::Lifetime(_) => {
                zonbi_lifetimes = quote! {
                    #zonbi_lifetimes
                    '__zonbi_life
                    ,
                };
            }
            GenericParam::Const(ConstParam { ident, .. }) => {
                zonbi_lifetimes = quote! {
                    #zonbi_lifetimes
                    #ident
                    ,
                };
            }
            GenericParam::Type(TypeParam { ident, .. }) => {
                zonbi_lifetimes = quote! {
                    #zonbi_lifetimes
                    #ident
                    ,
                };
            }
        }
    }

    quote! {
        unsafe impl #impl_generics ::zonbi::Zonbi for #name #ty_generics #where_clause {
            type Casted<'__zonbi_life> = #name<#zonbi_lifetimes>;

            unsafe fn zonbify<'__zonbi_life>(self) -> Self::Casted<'__zonbi_life> {
                ::core::mem::transmute(self)
            }

            unsafe fn zonbify_ref<'__zonbi_life>(&self) -> &Self::Casted<'__zonbi_life> {
                ::core::mem::transmute(self)
            }

            unsafe fn zonbify_mut<'__zonbi_life>(&mut self) -> &mut Self::Casted<'__zonbi_life> {
                ::core::mem::transmute(self)
            }
        }
    }
    .into()
}
