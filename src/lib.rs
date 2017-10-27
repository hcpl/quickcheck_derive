#![allow(warnings)]
#![recursion_limit="128"]
#[macro_use]
extern crate quote;
extern crate syn;
extern crate proc_macro;

use quote::Tokens;
use proc_macro::TokenStream;
use syn::{Ident,Body,Generics};

#[proc_macro_derive(Arbitrary)]
pub fn arbitrary(input: TokenStream) -> TokenStream {
    let input = input.to_string();
    let mut input = syn::parse_derive_input(&input).unwrap();

    add_type_bounds(&mut input.generics, "::quickcheck::Arbitrary");
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let name = input.ident;
    let arbitrary_body = arbitrary_body(&name, &input.body);
    let shrink_body = shrink_body(&name, &input.body);

    let output = quote! {
        impl #impl_generics ::quickcheck::Arbitrary for #name #ty_generics #where_clause {
            fn arbitrary<G: ::quickcheck::Gen>(gen: &mut G) -> Self {
                #arbitrary_body
            }

            fn shrink(&self) -> Box<Iterator<Item=Self>> {
                #shrink_body
            }
        }
    };

    output.parse().unwrap()
}

fn arbitrary_body(name: &Ident, body: &Body) -> Tokens {
    use syn::VariantData::*;
    match *body {
        Body::Enum(..) => panic!("derive(Arbitrary) only supports structs"),
        Body::Struct(Struct(ref fields)) => {
            let field_name = fields.iter().map(|field| &field.ident);
            quote! {
                #name {
                    #(#field_name: ::quickcheck::Arbitrary::arbitrary(gen)),*
                }
            }
        },
        Body::Struct(Tuple(ref fields)) => {
            // Tuples have no field names but we use this to execute the loop in `quote!`.
            // Otherwise, the loop will run zero times and produce invalid output for tuples with
            // 1+ arity.
            let field_name = fields.iter().map(|field| &field.ident);
            quote! {
                #name (
                    #(#field_name ::quickcheck::Arbitrary::arbitrary(gen)),*
                )
            }
        },
        Body::Struct(Unit) => quote! {
            drop(gen);
            #name
        },
    }
}

fn shrink_body(name: &Ident, body: &Body) -> Tokens {
    use syn::VariantData::*;
    match *body {
        Body::Enum(..) => panic!("derive(Arbitrary) only supports structs"),
        Body::Struct(Struct(ref fields)) => {
            // Safe to unwrap: there must be fields in structs
            let field_name = fields.iter().map(|field| field.ident.as_ref().unwrap()).collect::<Vec<_>>();
            // Just to circumvent the limitation of `quote!` which doesn't allow the same
            // identifier to be bound more than once
            let field_name2 = field_name.clone();
            let cloned_for_field = &field_name.iter()
                .map(|name| quote::Ident::new(format!("cloned_for_{}", name))).collect::<Vec<_>>();

            quote! {
                #(
                    let #cloned_for_field = self.clone();
                )*

                Box::new(
                    ::std::iter::empty()
                    #(
                        .chain(self.#field_name.shrink().map(move |shr_value| {
                            let mut result = #cloned_for_field.clone();
                            result.#field_name2 = shr_value;
                            result
                        }))
                    )*
                )
            }
        },
        Body::Struct(Tuple(ref fields)) => {
            let field_num = (0..fields.len()).map(|n| quote::Ident::new(n.to_string())).collect::<Vec<_>>();
            // Just to circumvent the limitation of `quote!` which doesn't allow the same
            // identifier to be bound more than once
            let field_num2 = field_num.clone();
            let cloned_for_field = &(0..fields.len())
                .map(|num| quote::Ident::new(format!("cloned_for_{}", num))).collect::<Vec<_>>();

            quote! {
                #(
                    let #cloned_for_field = self.clone();
                )*

                Box::new(
                    ::std::iter::empty()
                    #(
                        .chain(self.#field_num.shrink().map(move |shr_value| {
                            let mut result = #cloned_for_field.clone();
                            result.#field_num2 = shr_value;
                            result
                        }))
                    )*
                )
            }
        },
        Body::Struct(Unit) => quote! {
            ::quickcheck::empty_shrinker()
        },
    }
}

fn add_type_bounds(generics: &mut Generics, bound: &str) {
    let bound = syn::parse_ty_param_bound(bound).unwrap();

    for param in &mut generics.ty_params {
        param.bounds.push(bound.clone());
    }
}
