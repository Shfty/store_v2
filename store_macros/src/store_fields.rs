extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{
    export::quote::quote, export::Span, export::TokenStream2, parse::Parse, parse::ParseStream,
    parse_macro_input, ExprRange, Ident,
};

pub struct StoreFieldsInput {
    pub range: ExprRange,
}

impl Parse for StoreFieldsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let range = input.parse()?;

        Ok(StoreFieldsInput { range })
    }
}

pub fn get_type_idents(count: usize) -> Vec<Ident> {
    (0..count)
        .map(|i| Ident::new(&format!("T{}", i), Span::call_site()))
        .collect()
}

pub fn combinate_with_repitition(options: Vec<usize>, length: usize) -> Vec<Vec<usize>> {
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

    let type_idents: Vec<Ident> = get_type_idents(range_to);

    let combos_tokens: Vec<TokenStream2> = get_combos_tokens(&type_idents, range_from, range_to);

    (quote! {
        #(
            #combos_tokens
        )*
    })
    .into()
}

pub fn get_combos_tokens(type_idents: &[Ident], from: usize, to: usize) -> Vec<TokenStream2> {
    (from..=to)
        .flat_map(|i| combinate_with_repitition((0..5).collect::<Vec<usize>>(), i))
        .map(|combo| {
            let mut no_field_idents: Vec<&Ident> = Vec::new();
            let mut combo_ref_idents: Vec<&Ident> = Vec::new();
            let mut combo_option_ref_idents: Vec<&Ident> = Vec::new();
            let mut combo_mut_ref_idents: Vec<&Ident> = Vec::new();
            let mut combo_option_mut_ref_idents: Vec<&Ident> = Vec::new();

            combo
                .into_iter()
                .enumerate()
                .for_each(|(idx, ptr)| match ptr {
                    0 => no_field_idents.push(&type_idents[idx]),
                    1 => combo_ref_idents.push(&type_idents[idx]),
                    2 => combo_option_ref_idents.push(&type_idents[idx]),
                    3 => combo_mut_ref_idents.push(&type_idents[idx]),
                    4 => combo_option_mut_ref_idents.push(&type_idents[idx]),
                    _ => panic!("Unrecognized variant index"),
                });

            impl_store_fields_inner(
                no_field_idents,
                combo_ref_idents,
                combo_option_ref_idents,
                combo_mut_ref_idents,
                combo_option_mut_ref_idents,
            )
        })
        .collect()
}

pub fn impl_store_fields_inner(
    no_field_idents: Vec<&Ident>,
    ref_idents: Vec<&Ident>,
    option_ref_idents: Vec<&Ident>,
    mut_ref_idents: Vec<&Ident>,
    option_mut_ref_idents: Vec<&Ident>,
) -> TokenStream2 {
    let type_idents: Vec<&Ident> = no_field_idents
        .iter()
        .chain(ref_idents.iter())
        .chain(option_ref_idents.iter())
        .chain(mut_ref_idents.iter())
        .chain(option_mut_ref_idents.iter())
        .copied()
        .collect();

    let storage_vars: Vec<Ident> = (0..type_idents.len())
        .map(|i| syn::Ident::new(&format!("t{}", i), Span::call_site()))
        .collect();

    let (_, sv) = storage_vars.split_at(no_field_idents.len());
    let (ref_storage_vars, sv) = sv.split_at(ref_idents.len());
    let (option_ref_storage_vars, sv) = sv.split_at(option_ref_idents.len());
    let (mut_ref_storage_vars, option_mut_ref_storage_vars) = sv.split_at(mut_ref_idents.len());

    quote!(
        impl<'a, Key, #(#type_idents),*> StoreQuery<'a, (
            Key,
            #(NoField<#no_field_idents>,)*
            #(Ref<'a, #ref_idents>,)*
            #(Option<Ref<'a, #option_ref_idents>>,)*
            #(RefMut<'a, #mut_ref_idents>,)*
            #(Option<RefMut<'a, #option_mut_ref_idents>>,)*
        )> for Store<Key>
        where
            Key: Debug + Default + Copy + Ord + Hash + From<u32> + Into<u32> + 'static,
            #(
                #type_idents: 'static,
            )*
        {
            type Key = Key;

            fn get(&'a self, key: &Key) -> (
                Key,
                #(NoField<#no_field_idents>,)*
                #(Ref<'a, #ref_idents>,)*
                #(Option<Ref<'a, #option_ref_idents>>,)*
                #(RefMut<'a, #mut_ref_idents>,)*
                #(Option<RefMut<'a, #option_mut_ref_idents>>,)*
            ) {
                #(
                    assert!(!self.contains_type_key::<#no_field_idents>(key));
                )*

                #(
                    let #ref_storage_vars = self.get::<#ref_idents>(key).unwrap_or_else(|| panic!("Supplied key has no {} fields", std::any::type_name::<#ref_idents>()));
                )*
                #(
                    let #option_ref_storage_vars = self.get::<#option_ref_idents>(key);
                )*
                #(
                    let #mut_ref_storage_vars = self.get_mut::<#mut_ref_idents>(key).unwrap_or_else(|| panic!("Supplied key has no {} fields", std::any::type_name::<#mut_ref_idents>()));
                )*
                #(
                    let #option_mut_ref_storage_vars = self.get_mut::<#option_mut_ref_idents>(key);
                )*

                (*key #(, NoField::<#no_field_idents>::default())* #(, #ref_storage_vars)* #(, #option_ref_storage_vars)* #(, #mut_ref_storage_vars)* #(, #option_mut_ref_storage_vars)*)
            }

            fn iter(&'a self) -> StoreIterator<Key, (
                Key,
                #(NoField<#no_field_idents>,)*
                #(Ref<'a, #ref_idents>,)*
                #(Option<Ref<'a, #option_ref_idents>>,)*
                #(RefMut<'a, #mut_ref_idents>,)*
                #(Option<RefMut<'a, #option_mut_ref_idents>>,)*
            )> {
                let mut keys = self.keys_all();

                #(
                    keys &= &(!self.keys::<#no_field_idents>());
                )*

                #(
                    keys &= &(self.keys::<#ref_idents>());
                )*

                #(
                    keys &= &(self.keys::<#mut_ref_idents>());
                )*

                StoreIterator {
                    store: self,
                    keys: keys.into_iter(),
                    _phantom_data: PhantomData,
                }
            }

            fn iter_keys(&'a self, keys: &'a [Key]) -> StoreIterator<Key, (
                Key,
                #(NoField<#no_field_idents>,)*
                #(Ref<'a, #ref_idents>,)*
                #(Option<Ref<'a, #option_ref_idents>>,)*
                #(RefMut<'a, #mut_ref_idents>,)*
                #(Option<RefMut<'a, #option_mut_ref_idents>>,)*
            )> {
                let mut bit_set = BitSet::new();
                for key in keys {
                    bit_set.add((*key).into());
                }

                StoreIterator {
                    store: self,
                    keys: bit_set.into_iter(),
                    _phantom_data: PhantomData,
                }
            }
        }

        impl<'a, Key, #(#type_idents),*> Iterator for StoreIterator<'a, Key, (
           Key,
            #(NoField<#no_field_idents>,)*
            #(Ref<'a, #ref_idents>,)*
            #(Option<Ref<'a, #option_ref_idents>>,)*
            #(RefMut<'a, #mut_ref_idents>,)*
            #(Option<RefMut<'a, #option_mut_ref_idents>>,)*
        )>
        where
            Key: Debug + Default + Copy + Ord + Hash + From<u32> + Into<u32> + 'static,
            #(
                #type_idents: 'static,
            )*
        {
            type Item = (
                Key,
                #(NoField<#no_field_idents>,)*
                #(Ref<'a, #ref_idents>,)*
                #(Option<Ref<'a, #option_ref_idents>>,)*
                #(RefMut<'a, #mut_ref_idents>,)*
                #(Option<RefMut<'a, #option_mut_ref_idents>>,)*
            );

            fn next(&mut self) -> Option<Self::Item> {
                if let Some(key) = self.keys.next() {
                    let key: Key = key.into();

                    #(
                        let #ref_storage_vars = self.store.get::<#ref_idents>(&key).unwrap_or_else(|| panic!("Supplied key has no {} fields", std::any::type_name::<#ref_idents>()));
                    )*
                    #(
                        let #option_ref_storage_vars = self.store.get::<#option_ref_idents>(&key);
                    )*
                    #(
                        let #mut_ref_storage_vars = self.store.get_mut::<#mut_ref_idents>(&key).unwrap_or_else(|| panic!("Supplied key has no {} fields", std::any::type_name::<#mut_ref_idents>()));
                    )*
                    #(
                        let #option_mut_ref_storage_vars = self.store.get_mut::<#option_mut_ref_idents>(&key);
                    )*

                    Some((key #(, NoField::<#no_field_idents>::default())* #(, #ref_storage_vars)* #(, #option_ref_storage_vars)* #(, #mut_ref_storage_vars)* #(, #option_mut_ref_storage_vars)*))
                } else {
                    None
                }
            }
        }
    )
}
