extern crate proc_macro;

use quote::quote;
use crate::proc_macro::TokenStream;
use syn::{Lit, DeriveInput, Meta, MetaNameValue, Data};

#[proc_macro_derive(EventType, attributes(stream, event))]
pub fn event_type_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    match &ast.data {
        Data::Enum(..) => (),
        _ => panic!("unsupported, use enum for events")
    };
    let event = &ast.ident;
    let stream_type = ast.attrs.into_iter()
        .map(|attr| attr.parse_meta().unwrap())
        .filter_map(|meta| {
            match meta {
                Meta::NameValue(MetaNameValue { ref path, ref lit, .. }) if path.get_ident().unwrap() == "stream" => {
                    if let Lit::Str(lit) = lit {
                        Some(lit.value())
                    } else {
                        None
                    }
                }
                _ => None,
            }
        })
        .next()
        .unwrap_or(event.to_string());

    let gen = quote! {
        impl EventType for #event {
            fn stream_type() -> &'static str {
                #stream_type
            }
        }
    };

    gen.into()
}

#[proc_macro_derive(State)]
pub fn state_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

    let gen = quote! {
        impl State for #name {}
    };

    gen.into()
}

