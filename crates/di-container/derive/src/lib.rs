#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, parse_macro_input};

#[proc_macro_derive(Injectable, attributes(injectable))]
pub fn derive_injectable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(&input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand(input: &DeriveInput) -> syn::Result<TokenStream2> {
    if !input.generics.params.is_empty() {
        return Err(Error::new_spanned(
            &input.generics,
            "#[derive(Injectable)] does not support generic types",
        ));
    }

    let name = &input.ident;
    let construction = match &input.data {
        Data::Struct(data) => construction(&data.fields),
        Data::Enum(_) | Data::Union(_) => {
            return Err(Error::new_spanned(
                input,
                "#[derive(Injectable)] can only be used on structs",
            ));
        }
    };

    Ok(quote! {
        impl ::di_container::Injectable for #name {
            fn inject(ctx: &::di_container::BuildContext) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::di_container::Result<Self>> + '_>> {
                ::std::boxed::Box::pin(::std::future::ready(
                    (|| -> ::di_container::Result<Self> { #construction })()
                ))
            }
        }
    })
}

fn construction(fields: &Fields) -> TokenStream2 {
    match fields {
        Fields::Named(fields) => {
            let inits = fields.named.iter().map(|field| {
                let ident = field.ident.as_ref().expect("named field");
                quote!(#ident: ctx.get()?)
            });
            quote!(Ok(Self { #(#inits),* }))
        }
        Fields::Unnamed(fields) => {
            let inits = fields.unnamed.iter().map(|_| quote!(ctx.get()?));
            quote!(Ok(Self(#(#inits),*)))
        }
        Fields::Unit => quote!(Ok(Self)),
    }
}
