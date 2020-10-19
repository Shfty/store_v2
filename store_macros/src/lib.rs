extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{
    export::quote::quote, export::Span, export::TokenStream2, parse::Parse, parse::ParseStream,
    parse_macro_input, ExprRange, Ident,
};

struct StoreFieldsInput {
    range: ExprRange,
}

impl Parse for StoreFieldsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let range = input.parse()?;

        Ok(StoreFieldsInput { range })
    }
}

#[proc_macro]
pub fn impl_store_fields_iterator(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as StoreFieldsInput);

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
            let mut combo_option_ref_idents: Vec<&Ident> = Vec::new();
            let mut combo_mut_ref_idents: Vec<&Ident> = Vec::new();
            let mut combo_option_mut_ref_idents: Vec<&Ident> = Vec::new();

            combo
                .into_iter()
                .enumerate()
                .for_each(|(idx, ptr)| match ptr {
                    0 => combo_ref_idents.push(&type_idents[idx]),
                    1 => combo_option_ref_idents.push(&type_idents[idx]),
                    2 => combo_mut_ref_idents.push(&type_idents[idx]),
                    3 => combo_option_mut_ref_idents.push(&type_idents[idx]),
                    _ => panic!("Unrecognized variant index"),
                });

            impl_store_fields_inner(
                combo_ref_idents,
                combo_option_ref_idents,
                combo_mut_ref_idents,
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
    combo_ref_idents: Vec<&Ident>,
    combo_option_ref_idents: Vec<&Ident>,
    combo_mut_ref_idents: Vec<&Ident>,
    combo_option_mut_ref_idents: Vec<&Ident>,
) -> TokenStream2 {
    let type_idents: Vec<&Ident> = combo_ref_idents
        .iter()
        .chain(combo_option_ref_idents.iter())
        .chain(combo_mut_ref_idents.iter())
        .chain(combo_option_mut_ref_idents.iter())
        .copied()
        .collect();

    let storage_vars: Vec<Ident> = (0..type_idents.len())
        .map(|i| syn::Ident::new(&format!("t{}", i), Span::call_site()))
        .collect();

    let (ref_storage_vars, sv) = storage_vars.split_at(combo_ref_idents.len());
    let (option_ref_storage_vars, sv) = sv.split_at(combo_option_ref_idents.len());
    let (mut_ref_storage_vars, option_mut_ref_storage_vars) =
        sv.split_at(combo_mut_ref_idents.len());

    let first_storage_var = &storage_vars[0];
    let remaining_storage_vars: Vec<&Ident> = if storage_vars.len() > 1 {
        storage_vars[1..].iter().collect()
    } else {
        vec![]
    };

    let concrete_idents: Vec<&Ident> = combo_ref_idents
        .iter()
        .chain(combo_mut_ref_idents.iter())
        .copied()
        .collect();

    let filter_routine = if ref_storage_vars.len() + mut_ref_storage_vars.len() > 0 {
        quote!(
            .filter(|key| {
                #(
                    StoreTrait::contains_type_key::<#concrete_idents>(self, key)
                )&&*
            })
        )
    } else {
        quote!()
    };

    quote!(
        impl<'a, Key, #(#type_idents),*> StoreQuery<'a, (
            Key,
            #(Ref<'a, #combo_ref_idents>,)*
            #(Option<Ref<'a, #combo_option_ref_idents>>,)*
            #(RefMut<'a, #combo_mut_ref_idents>,)*
            #(Option<RefMut<'a, #combo_option_mut_ref_idents>>,)*
        )> for HybridStore<Key>
        where
            Key: Default + Copy + Ord + Hash + Into<usize> + 'static,
            #(
                #type_idents: 'static,
            )*
        {
            type Key = Key;

            fn get(&'a self, key: Key) -> (
                Key,
                #(Ref<'a, #combo_ref_idents>,)*
                #(Option<Ref<'a, #combo_option_ref_idents>>,)*
                #(RefMut<'a, #combo_mut_ref_idents>,)*
                #(Option<RefMut<'a, #combo_option_mut_ref_idents>>,)*
            ) {
                #(
                    let #ref_storage_vars = StoreTrait::get::<#combo_ref_idents>(self, key).expect(&format!("Supplied key has no {} fields", std::any::type_name::<#combo_ref_idents>()));
                )*
                #(
                    let #option_ref_storage_vars = StoreTrait::get::<#combo_option_ref_idents>(self, key);
                )*
                #(
                    let #mut_ref_storage_vars = StoreTrait::get_mut::<#combo_mut_ref_idents>(self, key).expect(&format!("Supplied key has no {} fields", std::any::type_name::<#combo_mut_ref_idents>()));
                )*
                #(
                    let #option_mut_ref_storage_vars = StoreTrait::get_mut::<#combo_option_mut_ref_idents>(self, key);
                )*

                (key, #(#storage_vars),*)
            }

            fn iter(&'a self) -> StoreIterator<Key, (
                Key,
                #(Ref<'a, #combo_ref_idents>,)*
                #(Option<Ref<'a, #combo_option_ref_idents>>,)*
                #(RefMut<'a, #combo_mut_ref_idents>,)*
                #(Option<RefMut<'a, #combo_option_mut_ref_idents>>,)*
            )> {
                #(
                    let #ref_storage_vars = StoreTrait::keys::<#combo_ref_idents>(self);
                )*
                #(
                    let #option_ref_storage_vars = StoreTrait::keys::<#combo_option_ref_idents>(self);
                )*
                #(
                    let #mut_ref_storage_vars = StoreTrait::keys::<#combo_mut_ref_idents>(self);
                )*
                #(
                    let #option_mut_ref_storage_vars = StoreTrait::keys::<#combo_option_mut_ref_idents>(self);
                )*

                let mut keys: Vec<Key> = #first_storage_var
                    .into_iter()
                    #(
                        .chain(#remaining_storage_vars.into_iter())
                    )*
                    #filter_routine
                    .copied()
                    .collect();

                keys.sort_unstable();
                keys.dedup();
                keys.reverse();

                StoreIterator {
                    store: self,
                    keys,
                    _phantom_data: PhantomData,
                }
            }

            fn iter_keys(&'a self, keys: &[Key]) -> StoreIterator<Key, (
                Key,
                #(Ref<'a, #combo_ref_idents>,)*
                #(Option<Ref<'a, #combo_option_ref_idents>>,)*
                #(RefMut<'a, #combo_mut_ref_idents>,)*
                #(Option<RefMut<'a, #combo_option_mut_ref_idents>>,)*
            )> {
                let mut keys: Vec<Key> = keys
                    .into_iter()
                    .copied()
                    .collect();

                keys.sort_unstable();
                keys.dedup();
                keys.reverse();

                StoreIterator {
                    store: self,
                    keys,
                    _phantom_data: PhantomData,
                }
            }
        }

        impl<'a, Key, #(#type_idents),*> Iterator for StoreIterator<'a, Key, (
            Key,
            #(Ref<'a, #combo_ref_idents>,)*
            #(Option<Ref<'a, #combo_option_ref_idents>>,)*
            #(RefMut<'a, #combo_mut_ref_idents>,)*
            #(Option<RefMut<'a, #combo_option_mut_ref_idents>>,)*
        )>
        where
            Key: Default + Copy + Ord + Into<usize> + Hash + 'static,
            #(
                #type_idents: 'static,
            )*
        {
            type Item = (
                Key,
                #(Ref<'a, #combo_ref_idents>,)*
                #(Option<Ref<'a, #combo_option_ref_idents>>,)*
                #(RefMut<'a, #combo_mut_ref_idents>,)*
                #(Option<RefMut<'a, #combo_option_mut_ref_idents>>,)*
            );

            fn next(&mut self) -> Option<Self::Item> {
                if let Some(key) = self.keys.pop() {
                    #(
                        let #ref_storage_vars = StoreTrait::get::<#combo_ref_idents>(self.store, key).expect(&format!("Supplied key has no {} fields", std::any::type_name::<#combo_ref_idents>()));
                    )*
                    #(
                        let #option_ref_storage_vars = StoreTrait::get::<#combo_option_ref_idents>(self.store, key);
                    )*
                    #(
                        let #mut_ref_storage_vars = StoreTrait::get_mut::<#combo_mut_ref_idents>(self.store, key).expect(&format!("Supplied key has no {} fields", std::any::type_name::<#combo_mut_ref_idents>()));
                    )*
                    #(
                        let #option_mut_ref_storage_vars = StoreTrait::get_mut::<#combo_option_mut_ref_idents>(self.store, key);
                    )*

                    Some((key, #(#storage_vars),*))
                } else {
                    None
                }
            }
        }
    )
}
