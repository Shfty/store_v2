extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{
    export::quote::quote, export::Span, export::TokenStream2, parse::Parse, parse::ParseStream,
    parse_macro_input, ExprRange, Ident, Index, Token,
};

struct StoreFieldsInput {
    storage: Ident,
    range: ExprRange,
}

impl Parse for StoreFieldsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let storage = input.parse()?;
        input.parse::<Token!(,)>()?;
        let range = input.parse()?;

        Ok(StoreFieldsInput { storage, range })
    }
}

#[proc_macro]
pub fn impl_store_fields_iterator(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as StoreFieldsInput);

    let storage_type = input.storage;
    let range_from = input.range.from.expect("No from range");
    let range_to = input.range.to.expect("No to range");

    let range_from = if let syn::Expr::Lit(expr) = *range_from {
        if let syn::Lit::Int(lit) = expr.lit {
            lit.base10_parse::<usize>().expect("Not an integer")
        } else {
            panic!("Not a literal integer")
        }
    } else {
        panic!("Not a literal expression")
    };

    let range_to = if let syn::Expr::Lit(expr) = *range_to {
        if let syn::Lit::Int(lit) = expr.lit {
            lit.base10_parse::<usize>().expect("Not an integer")
        } else {
            panic!("Not a literal integer")
        }
    } else {
        panic!("Not a literal expression")
    };

    let type_idents: Vec<Ident> = (0..range_to)
        .map(|i| Ident::new(&format!("T{}", i), Span::call_site()))
        .collect();

    let combos_tokens: Vec<TokenStream2> = (range_from..=range_to)
        .flat_map(|i| combinate_with_repitition((0..4).collect::<Vec<usize>>(), i))
        .map(|combo| {
            let mut combo_ref_idents: Vec<&Ident> = Vec::new();
            let mut combo_mut_ref_idents: Vec<&Ident> = Vec::new();
            let mut combo_option_ref_idents: Vec<&Ident> = Vec::new();
            let mut combo_option_mut_ref_idents: Vec<&Ident> = Vec::new();

            combo
                .into_iter()
                .enumerate()
                .for_each(|(idx, ptr)| match ptr {
                    0 => combo_ref_idents.push(&type_idents[idx]),
                    1 => combo_mut_ref_idents.push(&type_idents[idx]),
                    2 => combo_option_ref_idents.push(&type_idents[idx]),
                    3 => combo_option_mut_ref_idents.push(&type_idents[idx]),
                    _ => panic!("Unrecognized variant index"),
                });

            impl_store_fields_inner(
                &storage_type,
                combo_ref_idents,
                combo_mut_ref_idents,
                combo_option_ref_idents,
                combo_option_mut_ref_idents,
            )
        })
        .collect();

    (quote! {
        #(
            #combos_tokens
        )*
    })
    .into()
}

fn combinate_with_repitition(options: Vec<usize>, length: usize) -> Vec<Vec<usize>> {
    if length == 1 {
        return options.into_iter().map(|option| vec![option]).collect();
    }

    options
        .iter()
        .enumerate()
        .flat_map(|(i, option)| {
            combinate_with_repitition(options[i..].to_vec(), length - 1)
                .into_iter()
                .map(|smaller_combo| {
                    [].iter()
                        .chain([*option].iter())
                        .chain(smaller_combo.iter())
                        .copied()
                        .collect()
                })
                .collect::<Vec<Vec<usize>>>()
        })
        .collect()
}

fn impl_store_fields_inner(
    storage_type: &Ident,
    combo_ref_idents: Vec<&Ident>,
    combo_mut_ref_idents: Vec<&Ident>,
    combo_option_ref_idents: Vec<&Ident>,
    combo_option_mut_ref_idents: Vec<&Ident>,
) -> TokenStream2 {
    let type_idents: Vec<&Ident> = combo_ref_idents
        .iter()
        .chain(combo_mut_ref_idents.iter())
        .chain(combo_option_ref_idents.iter())
        .chain(combo_option_mut_ref_idents.iter())
        .copied()
        .collect();

    let (ref_type_idents, ti) = type_idents.split_at(combo_ref_idents.len());
    let (mut_ref_type_idents, ti) = ti.split_at(combo_mut_ref_idents.len());
    let (option_ref_type_idents, option_mut_ref_type_idents) =
        ti.split_at(combo_option_ref_idents.len());

    let field_types: Vec<TokenStream2> = std::iter::repeat(quote!(ConcreteImmutableField))
        .take(combo_ref_idents.len())
        .chain(std::iter::repeat(quote!(ConcreteMutableField)).take(combo_mut_ref_idents.len()))
        .chain(
            std::iter::repeat(quote!(OptionalImmutableField)).take(combo_option_ref_idents.len()),
        )
        .chain(
            std::iter::repeat(quote!(OptionalMutableField)).take(combo_option_mut_ref_idents.len()),
        )
        .collect();

    let storage_fields: Vec<Index> = (0..type_idents.len()).map(Index::from).collect();

    let (ref_storage_fields, storage_fields) = storage_fields.split_at(combo_ref_idents.len());
    let (mut_ref_storage_fields, storage_fields) =
        storage_fields.split_at(combo_mut_ref_idents.len());
    let (option_ref_storage_fields, option_mut_ref_storage_fields) =
        storage_fields.split_at(combo_option_ref_idents.len());

    let storage_vars: Vec<Ident> = (0..type_idents.len())
        .map(|i| syn::Ident::new(&format!("t{}", i), Span::call_site()))
        .collect();

    let (ref_storage_vars, sv) = storage_vars.split_at(combo_ref_idents.len());
    let (mut_ref_storage_vars, sv) = sv.split_at(combo_mut_ref_idents.len());
    let (option_ref_storage_vars, option_mut_ref_storage_vars) =
        sv.split_at(combo_option_ref_idents.len());

    let first_storage_var = &storage_vars[0];
    let remaining_storage_vars: Vec<&Ident> = if storage_vars.len() > 1 {
        storage_vars[1..].iter().collect()
    } else {
        vec![]
    };

    let concrete_storage_vars: Vec<&Ident> = ref_storage_vars
        .iter()
        .chain(mut_ref_storage_vars.iter())
        .collect();

    let filter_routine = if ref_storage_vars.len() + mut_ref_storage_vars.len() > 0 {
        quote!(
            .filter(|key|#(#concrete_storage_vars.contains_key(key)) && *)
        )
    } else {
        quote!()
    };

    quote!(
        impl<'a, Key, #(#type_idents),*> Iterator
            for StoreFieldsIterator<
                Key,
                (
                    #(
                      &'a std::cell::RefCell<#storage_type<Key, #type_idents>>,
                    )*
                ),
                (
                    #(
                        #field_types,
                    )*
                ),
            >
        where
            Key: Copy + Eq + std::hash::Hash,
        {
            type Item = (
                Key,
                (
                    #(
                        std::cell::Ref<'a, #combo_ref_idents>,
                    )*
                    #(
                        std::cell::RefMut<'a, #combo_mut_ref_idents>,
                    )*
                    #(
                        Option<std::cell::Ref<'a, #combo_option_ref_idents>>,
                    )*
                    #(
                        Option<std::cell::RefMut<'a, #combo_option_mut_ref_idents>>,
                    )*
                ),
            );

            fn next(&mut self) -> Option<Self::Item> {
                if let Some(key) = self.keys.pop() {
                    Some((
                        key,
                        (
                            #(
                                std::cell::Ref::map(self.storage.#ref_storage_fields.borrow(), |storage| {
                                    storage.get(&key).unwrap()
                                }),
                            )*
                            #(
                                std::cell::RefMut::map(self.storage.#mut_ref_storage_fields.borrow_mut(), |storage| {
                                    storage.get_mut(&key).unwrap()
                                }),
                            )*
                            #(
                                if self.storage.#option_ref_storage_fields.borrow().contains_key(&key) {
                                    Some(std::cell::Ref::map(self.storage.#option_ref_storage_fields.borrow(), |storage| {
                                        storage.get(&key).unwrap()
                                    }))
                                } else {
                                    None
                                },
                            )*
                            #(
                                if self.storage.#option_mut_ref_storage_fields.borrow_mut().contains_key(&key) {
                                    Some(std::cell::RefMut::map(self.storage.#option_mut_ref_storage_fields.borrow_mut(), |storage| {
                                        storage.get_mut(&key).unwrap()
                                    }))
                                } else {
                                    None
                                },
                            )*
                        ),
                    ))
                } else {
                    None
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let len = self.keys.len();
                (len, Some(len))
            }
        }

        impl<'a, Key, #(#type_idents),*> ExactSizeIterator
            for StoreFieldsIterator<
                Key,
                (
                    #(
                      &'a std::cell::RefCell<#storage_type<Key, #type_idents>>,
                    )*
                ),
                (
                    #(
                        #field_types,
                    )*
                ),
            >
        where
            Key: Copy + Eq + std::hash::Hash,
        {
        }

        impl<'a, Key, #(#type_idents),*> std::iter::FusedIterator
            for StoreFieldsIterator<
                Key,
                (
                    #(
                      &'a std::cell::RefCell<#storage_type<Key, #type_idents>>,
                    )*
                ),
                (
                    #(
                        #field_types,
                    )*
                ),
            >
        where
            Key: Copy + Eq + std::hash::Hash,
        {
        }

        impl<'a, Key, #(#type_idents),*>
            StoreQuery<
                'a,
                Key,
                (
                    #(
                        std::cell::Ref<'a, #combo_ref_idents>,
                    )*
                    #(
                        std::cell::RefMut<'a, #combo_mut_ref_idents>,
                    )*
                    #(
                        Option<std::cell::Ref<'a, #combo_option_ref_idents>>,
                    )*
                    #(
                        Option<std::cell::RefMut<'a, #combo_option_mut_ref_idents>>,
                    )*
                ),
            > for crate::Store
        where
            Key: Copy + Eq + Ord + std::hash::Hash + 'static,
            #(
                #type_idents: crate::Storable<Storage = #storage_type<Key, #type_idents>> + 'static,
            )*
        {
            type Storage = (
                #(
                    &'a std::cell::RefCell<<#type_idents as crate::Storable>::Storage>,
                )*
            );

            type FieldTypes = (
                #(
                    #field_types,
                )*
            );

            type Item = (
                #(
                    std::cell::Ref<'a, #combo_ref_idents>,
                )*
                #(
                    std::cell::RefMut<'a, #combo_mut_ref_idents>,
                )*
                #(
                    Option<std::cell::Ref<'a, #combo_option_ref_idents>>,
                )*
                #(
                    Option<std::cell::RefMut<'a, #combo_option_mut_ref_idents>>,
                )*
            );

            fn iter(&'a self) -> StoreFieldsIterator<Key, Self::Storage, Self::FieldTypes> {
                let keys: Vec<Key>;
                {
                    #(
                        let #ref_storage_vars = self.get_storage::<#ref_type_idents>().borrow();
                    )*
                    #(
                        let #mut_ref_storage_vars = self.get_storage::<#mut_ref_type_idents>().borrow_mut();
                    )*
                    #(
                        let #option_ref_storage_vars = self.get_storage::<#option_ref_type_idents>().borrow();
                    )*
                    #(
                        let #option_mut_ref_storage_vars = self.get_storage::<#option_mut_ref_type_idents>().borrow_mut();
                    )*

                    let mut shared_keys: Vec<Key> = #first_storage_var
                        .keys()
                        #(
                        .chain(#remaining_storage_vars.keys())
                        )*
                        #filter_routine
                        .copied()
                        .collect();

                    shared_keys.sort_unstable();
                    shared_keys.dedup();

                    keys = shared_keys;
                }

                StoreQuery::<Key, (
                    #(
                        std::cell::Ref<'a, #combo_ref_idents>,
                    )*
                    #(
                        std::cell::RefMut<'a, #combo_mut_ref_idents>,
                    )*
                    #(
                        Option<std::cell::Ref<'a, #combo_option_ref_idents>>,
                    )*
                    #(
                        Option<std::cell::RefMut<'a, #combo_option_mut_ref_idents>>,
                    )*
                )>::iter_keys(self, keys)
            }

            fn iter_keys(&'a self, mut keys: Vec<Key>) -> StoreFieldsIterator<Key, Self::Storage, Self::FieldTypes> {
                keys.reverse();
                StoreFieldsIterator {
                    keys,
                    storage: (
                        #(
                            self.get_storage::<#type_idents>(),
                        )*
                    ),
                    _phantom: std::marker::PhantomData,
                }
            }

            fn get(&'a self, key: Key) -> Self::Item {
                #(
                    let #ref_storage_vars = self.get_storage::<#ref_type_idents>();
                )*
                #(
                    let #mut_ref_storage_vars = self.get_storage::<#mut_ref_type_idents>();
                )*
                #(
                    let #option_ref_storage_vars = self.get_storage::<#option_ref_type_idents>();
                )*
                #(
                    let #option_mut_ref_storage_vars = self.get_storage::<#option_mut_ref_type_idents>();
                )*

                (
                    #(
                        std::cell::Ref::map(#ref_storage_vars.borrow(), |storage| {
                            storage.get(&key).unwrap()
                        }),
                    )*
                    #(
                        std::cell::RefMut::map(#mut_ref_storage_vars.borrow_mut(), |storage| {
                            storage.get_mut(&key).unwrap()
                        }),
                    )*
                    #(
                        if #option_ref_storage_vars.borrow().contains_key(&key) {
                            Some(std::cell::Ref::map(#option_ref_storage_vars.borrow(), |storage| {
                                storage.get(&key).unwrap()
                            }))
                        } else {
                            None
                        },
                    )*
                    #(
                        if #option_mut_ref_storage_vars.borrow_mut().contains_key(&key) {
                            Some(std::cell::RefMut::map(#option_mut_ref_storage_vars.borrow_mut(), |storage| {
                                storage.get_mut(&key).unwrap()
                            }))
                        } else {
                            None
                        },
                    )*
                )
            }
        }
    )
}
