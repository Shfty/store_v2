use proc_macro::TokenStream;
use syn::{
    export::{quote::quote, TokenStream2},
    parse_macro_input, Ident, Index, LitInt,
};

use crate::tuple::impl_tuple;

pub fn impl_assemble(input: TokenStream) -> TokenStream {
    let arity: usize = parse_macro_input!(input as LitInt)
        .base10_parse()
        .expect("Macro input is not an integer literal");

    let tokens = impl_tuple(arity, impl_assemble_inner);

    quote!(
        #(#tokens)*
    )
    .into()
}

fn impl_assemble_inner(type_keys: &[Ident]) -> TokenStream2 {
    let tuple_indices: Vec<Index> = type_keys
        .iter()
        .enumerate()
        .map(|(i, _)| Index::from(i))
        .collect();

    quote!(
        impl<Key, #(#type_keys),*> Assemble<(#(#type_keys,)*)> for Key
        where
            Key: StoreKey,
            #(
                #type_keys: Debug + 'static,
            )*
        {
            fn assemble(self, component_store: &mut Store<Key>, tuple: (#(#type_keys,)*)) -> Key {
                #(
                    component_store.insert(self, tuple.#tuple_indices);
                )*

                self
            }
        }
    )
}

pub fn impl_assemble_chain(input: TokenStream) -> TokenStream {
    let arity: usize = parse_macro_input!(input as LitInt)
        .base10_parse()
        .expect("Macro input is not an integer literal");

    let tokens = impl_tuple(arity, impl_assemble_chain_inner);

    quote!(
        #(#tokens)*
    )
    .into()
}

fn impl_assemble_chain_inner(type_keys: &[Ident]) -> TokenStream2 {
    let tuple_indices: Vec<Index> = type_keys
        .iter()
        .enumerate()
        .map(|(i, _)| Index::from(i))
        .collect();

    quote!(
        impl<Key, #(#type_keys),*> AssembleChain<Key> for (#(#type_keys,)*)
        where
            Key: StoreKey,
            #(
                #type_keys: Assemble<Key>,
            )*
        {
            fn assemble_chain(self, store: &mut Store<Key>, key: Key) -> Key {
                #(
                    self.#tuple_indices.assemble(store, key);
                )*
                key
            }
        }
    )
}

pub fn impl_disassemble(input: TokenStream) -> TokenStream {
    let arity: usize = parse_macro_input!(input as LitInt)
        .base10_parse()
        .expect("Macro input is not an integer literal");

    let tokens = impl_tuple(arity, impl_disassemble_inner);

    quote!(
        #(#tokens)*
    )
    .into()
}

fn impl_disassemble_inner(type_idents: &[Ident]) -> TokenStream2 {
    quote!(
        impl<Key, #(#type_idents),*> Disassemble<Key> for (#(#type_idents,)*)
        where
            Key: StoreKey,
            #(
                #type_idents: Debug + Clone + 'static,
            )*
        {
            fn disassemble(component_store: &mut Store<Key>, key: &Key) {
                #(
                    component_store.remove::<#type_idents>(key);
                )*
            }
        }
    )
}
