use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{ConstParam, GenericParam, Lifetime, LifetimeParam, TypeParam};

#[proc_macro_derive(Zonbi)]
pub fn derive_zonbi(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let name = &ast.ident;
    let generics = ast.generics;

    let zonbi_life = Lifetime::new("'__zonbi_life", Span::call_site());

    let mut zonbi_lifetimes = quote!();
    for ig in &generics.params {
        match ig {
            GenericParam::Lifetime(_) => {
                zonbi_lifetimes = quote! {
                    #zonbi_lifetimes
                    #zonbi_life // here we inject the zonbi lifetime
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

    let mut generics2 = generics.clone();
    let (_, ty_generics, _) = generics.split_for_impl();

    let life_param = LifetimeParam::new(zonbi_life.clone());
    generics2.params.push(GenericParam::Lifetime(life_param));

    let (impl_generics, _, where_clause) = generics2.split_for_impl();
    let other_clauses = where_clause.map(|w| &w.predicates);

    quote! {
        unsafe impl #impl_generics ::zonbi::Zonbi<#zonbi_life> for #name #ty_generics
        where
            Self: #zonbi_life,
            #other_clauses
        {
            type Casted = #name<#zonbi_lifetimes>;

            fn zonbi_id() -> ::zonbi::ZonbiId {
                ::zonbi::ZonbiId::from(core::any::TypeId::of::<#name<'static>>())
            }

            unsafe fn zonbify(self) -> Self::Casted {
                ::core::mem::transmute(self)
            }

            unsafe fn zonbify_ref(&self) -> &Self::Casted {
                ::core::mem::transmute(self)
            }

            unsafe fn zonbify_mut(&mut self) -> &mut Self::Casted {
                ::core::mem::transmute(self)
            }
        }
    }
    .into()
}
