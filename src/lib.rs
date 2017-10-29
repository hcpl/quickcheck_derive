#![allow(warnings)]
#![recursion_limit="128"]

#[macro_use]
extern crate quote;
extern crate syn;
extern crate proc_macro;

use std::usize;

use quote::Tokens;
use proc_macro::TokenStream;
use syn::{Body, Ident, Generics, VariantData};

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

fn add_type_bounds(generics: &mut Generics, bound: &str) {
    let bound = syn::parse_ty_param_bound(bound).unwrap();

    for param in &mut generics.ty_params {
        param.bounds.push(bound.clone());
    }
}


fn arbitrary_body(name: &Ident, body: &Body) -> Tokens {
    match *body {
        Body::Enum(ref variants) => {
            let arms = variants.iter().enumerate().map(|(i, variant)| {
                let variant_qualified_name = Ident::new(format!("{}::{}", name, &variant.ident));
                let variant_arbitrary_body =
                    variant_data_arbitrary_body(&variant_qualified_name, &variant.data);

                quote! {
                    #i =>  { #variant_arbitrary_body },
                }
            }).collect::<Vec<_>>();

            let variants_count = variants.len();

            let opt_unreachable = if variants_count == usize::MAX {
                None
            } else {
                Some(quote! { x => unreachable!("A number generated out of range: {}", x), })
            };

            quote! {
                match gen.gen_range::<usize>(0, #variants_count) {
                    #(#arms)*
                    #opt_unreachable
                }
            }
        },
        Body::Struct(ref variant_data) => {
            variant_data_arbitrary_body(name, variant_data)
        },
    }
}

fn variant_data_arbitrary_body(name: &Ident, variant_data: &VariantData) -> Tokens {
    match *variant_data {
        VariantData::Struct(ref fields) => {
            let field_name = fields.iter().map(|field| &field.ident);
            quote! {
                #name {
                    #(#field_name: ::quickcheck::Arbitrary::arbitrary(gen)),*
                }
            }
        },
        VariantData::Tuple(ref fields) => {
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
        VariantData::Unit => quote! {
            drop(gen);
            #name
        },
    }
}


fn shrink_body(name: &Ident, body: &Body) -> Tokens {
    match *body {
        Body::Enum(ref variants) => {
            // Pre-collect all unit variants as shrinking targets
            let mut unit_variants = variants.iter()
                .filter(|v| v.data == VariantData::Unit)
                .map(|v| quote::Ident::new(format!("{}::{}", name, &v.ident)))
                .peekable();

            let init_shrink = &if unit_variants.peek().is_some() {
                quote! {
                    vec![ #(#unit_variants),* ].into_iter()
                }
            } else {
                quote! {
                    ::std::iter::empty()
                }
            };

            let opt_unreachable = &if variants.len() == 1 {
                None
            } else {
                Some(quote! { _ => unreachable!("Must not be matched while shrinking"), })
            };

            let arms = variants.iter().map(|variant| {
                let variant_name = &variant.ident;

                match variant.data {
                    VariantData::Struct(ref fields) => {
                        let qualified_variant_name = vec![
                            quote::Ident::new(format!("{}::{}", name, variant_name));
                            fields.len()
                        ];
                        // Needs to be repeated in loop
                        let opt_unreachable = vec![opt_unreachable; fields.len()];

                        let field_name = fields.iter()
                            .map(|f| f.ident.as_ref().unwrap()) // struct variant fields must be named
                            .collect::<Vec<_>>();

                        let cloned_for_field = &field_name.iter()
                            .map(|name| quote::Ident::new(format!("cloned_for_{}", name)))
                            .collect::<Vec<_>>();

                        // Just to circumvent the limitation of `quote!` which doesn't allow the same
                        // identifier to be bound more than once
                        let field_name1 = &field_name;
                        let field_name2 = &field_name;
                        let field_name3 = &field_name;

                        quote! {
                            #name::#variant_name { #(ref #field_name1),* } => {
                                #(
                                    let #cloned_for_field = self.clone();
                                )*

                                Box::new(
                                    #init_shrink
                                    #(
                                        .chain(#field_name1.shrink().map(move |shr_value| {
                                            let mut result = #cloned_for_field.clone();
                                            match *&mut result {
                                                #qualified_variant_name {
                                                    ref mut #field_name2,
                                                    ..
                                                } => *#field_name3 = shr_value,
                                                #opt_unreachable
                                            }
                                            result
                                        }))
                                    )*
                                )
                            }
                        }
                    },
                    VariantData::Tuple(ref fields) => {
                        let qualified_variant_name = vec![
                            quote::Ident::new(format!("{}::{}", name, variant_name));
                            fields.len()
                        ];
                        // Needs to be repeated in loop
                        let opt_unreachable = vec![opt_unreachable; fields.len()];

                        let field_num = (0..fields.len())
                            .map(|n| quote::Ident::new(n.to_string()))
                            .collect::<Vec<_>>();
                        let field_num_name = (0..fields.len())
                            .map(|n| quote::Ident::new(format!("field_{}", n)))
                            .collect::<Vec<_>>();

                        let cloned_for_num = &(0..fields.len())
                            .map(|name| quote::Ident::new(format!("cloned_for_{}", name)))
                            .collect::<Vec<_>>();

                        // Just to circumvent the limitation of `quote!` which doesn't allow the same
                        // identifier to be bound more than once
                        let field_num_name1 = &field_num_name;
                        let field_num_name2 = &field_num_name;
                        let field_num_name3 = &field_num_name;

                        quote! {
                            #name::#variant_name( #(ref #field_num_name1),* ) => {
                                #(
                                    let #cloned_for_num = self.clone();
                                )*

                                Box::new(
                                    #init_shrink
                                    #(
                                        .chain(#field_num_name1.shrink().map(move |shr_value| {
                                            let mut result = #cloned_for_num.clone();
                                            match *&mut result {
                                                #qualified_variant_name {
                                                    #field_num: ref mut #field_num_name2,
                                                    ..
                                                } => *#field_num_name3 = shr_value,
                                                #opt_unreachable
                                            }
                                            result
                                        }))
                                    )*
                                )
                            }
                        }
                    },
                    VariantData::Unit => {
                        quote! {
                            #name::#variant_name => ::quickcheck::empty_shrinker(),
                        }
                    },
                }
            }).collect::<Vec<_>>();

            quote! {
                match *self {
                    #(#arms)*
                }
            }
        },
        Body::Struct(VariantData::Struct(ref fields)) => {
            // Needs to be repeated in loop
            let name = vec![name; fields.len()];
            let field_name = fields.iter()
                .map(|f| f.ident.as_ref().unwrap()) // struct variant fields must be named
                .collect::<Vec<_>>();

            let cloned_for_field = &field_name.iter()
                .map(|name| quote::Ident::new(format!("cloned_for_{}", name)))
                .collect::<Vec<_>>();

            // Just to circumvent the limitation of `quote!` which doesn't allow the same
            // identifier to be bound more than once
            let field_name1 = &field_name;
            let field_name2 = &field_name;

            quote! {
                #(
                    let #cloned_for_field = self.clone();
                )*

                Box::new(
                    ::std::iter::empty()
                    #(
                        .chain(self.#field_name1.shrink().map(move |shr_value| {
                            let mut result = #cloned_for_field.clone();
                            result.#field_name2 = shr_value;
                            result
                        }))
                    )*
                )
            }
        },
        Body::Struct(VariantData::Tuple(ref fields)) => {
            // Needs to be repeated in loop
            let name = vec![name; fields.len()];
            let field_num = (0..fields.len())
                .map(|n| quote::Ident::new(n.to_string()))
                .collect::<Vec<_>>();

            let cloned_for_num = &(0..fields.len())
                .map(|name| quote::Ident::new(format!("cloned_for_{}", name)))
                .collect::<Vec<_>>();

            // Just to circumvent the limitation of `quote!` which doesn't allow the same
            // identifier to be bound more than once
            let field_num1 = &field_num;
            let field_num2 = &field_num;

            quote! {
                #(
                    let #cloned_for_num = self.clone();
                )*

                Box::new(
                    ::std::iter::empty()
                    #(
                        .chain(self.#field_num1.shrink().map(move |shr_value| {
                            let mut result = #cloned_for_num.clone();
                            result.#field_num2 = shr_value;
                            result
                        }))
                    )*
                )
            }
        },
        Body::Struct(VariantData::Unit) => {
            quote! {
                ::quickcheck::empty_shrinker()
            }
        },
    }
}
