use syn::{Generics, Ident, parse_quote};

pub fn add_self_send_bound(ident: &Ident, generics: &mut Generics) {
    let predicate = {
        let (_, ty_generics, _) = generics.split_for_impl();
        parse_quote!(#ident #ty_generics: ::core::marker::Send)
    };
    generics.make_where_clause().predicates.push(predicate);
}
