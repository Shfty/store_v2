extern crate proc_macro;

mod store_fields;
mod assemblage;
mod tuple;

use proc_macro::TokenStream;

#[proc_macro]
pub fn impl_store_fields_iterator(input: TokenStream) -> TokenStream {
    store_fields::impl_store_fields_iterator(input)
}

#[proc_macro]
pub fn impl_assemble(input: TokenStream) -> TokenStream {
    assemblage::impl_assemble(input)
}

#[proc_macro]
pub fn impl_assemble_chain(input: TokenStream) -> TokenStream {
    assemblage::impl_assemble_chain(input)
}

#[proc_macro]
pub fn impl_disassemble(input: TokenStream) -> TokenStream {
    assemblage::impl_disassemble(input)
}
