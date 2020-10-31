use syn::{
    export::{Span, TokenStream2},
    Ident,
};

pub fn impl_tuple(arity: usize, f: fn(&[Ident]) -> TokenStream2) -> Vec<TokenStream2> {
    let type_keys: Vec<Ident> = (0..arity)
        .map(|arity| Ident::new(&format!("T{}", arity), Span::call_site()))
        .collect();

    (0..arity)
        .map(|arity| {
            let type_keys = &type_keys[0..arity];
            f(type_keys)
        })
        .collect()
}
