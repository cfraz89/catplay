use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{Attribute, Data, DeriveInput, Error, Fields, Ident, LitStr, Meta, Path, Result, parse_quote, spanned::Spanned};

use crate::common::add_self_send_bound;

pub fn expand(input: &DeriveInput) -> Result<TokenStream2> {
    let ident = &input.ident;
    let crate_path = parse_crate_path(&input.attrs)?;
    let mut steps = parse_shutdown_funcs(&input.attrs)?;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(Error::new_spanned(
                    input,
                    "AsyncShutdown can only be derived for structs with named fields",
                ));
            }
        },
        _ => return Err(Error::new_spanned(input, "AsyncShutdown can only be derived for structs")),
    };

    for field in fields {
        let Some(field_ident) = &field.ident else {
            continue;
        };

        for attr in &field.attrs {
            let Some(attr_kind) = parse_shutdown_attr(attr)? else {
                continue;
            };

            let step = match attr_kind {
                ShutdownAttr::Shutdown => quote_spanned! { field_ident.span()=>
                        #crate_path::AsyncShutdown::shutdown(&mut self.#field_ident).await;
                },
                ShutdownAttr::ShutdownTake => quote_spanned! { field_ident.span()=>
                    if let Some(mut __shutdown_value) = self.#field_ident.take() {
                        #crate_path::AsyncShutdown::shutdown(&mut __shutdown_value).await;
                    }
                },
                ShutdownAttr::ShutdownPinned => quote_spanned! { field_ident.span()=>
                        #crate_path::AsyncShutdownDyn::shutdown_pinned(&mut self.#field_ident).await;
                },
                ShutdownAttr::ShutdownPinnedTake => quote_spanned! { field_ident.span()=>
                    if let Some(mut __shutdown_value) = self.#field_ident.take() {
                        #crate_path::AsyncShutdownDyn::shutdown_pinned(&mut __shutdown_value).await;
                    }
                },
            };

            steps.push(step);
        }
    }

    if steps.is_empty() {
        return Err(Error::new_spanned(
            input,
            "AsyncShutdown derive requires at least one #[shutdown], #[shutdown_pinned], or #[shutdown_func(...)]",
        ));
    }

    let mut generics = input.generics.clone();
    add_self_send_bound(ident, &mut generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #crate_path::AsyncShutdown for #ident #ty_generics #where_clause {
            async fn shutdown(&mut self) {
                #(#steps)*
            }
        }
    })
}

fn parse_crate_path(attrs: &[Attribute]) -> Result<Path> {
    let mut crate_path: Path = parse_quote!(::catplay_util);

    for attr in attrs {
        if !attr.path().is_ident("async_shutdown") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("crate") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                crate_path = lit.parse()?;
                Ok(())
            } else {
                Err(meta.error("unsupported async_shutdown option"))
            }
        })?;
    }

    Ok(crate_path)
}

fn parse_shutdown_funcs(attrs: &[Attribute]) -> Result<Vec<TokenStream2>> {
    let mut steps = Vec::new();

    for attr in attrs {
        if !attr.path().is_ident("shutdown_func") {
            continue;
        }

        let func: Ident = attr.parse_args()?;
        steps.push(quote_spanned! { attr.span()=>
            Self::#func(self).await;
        });
    }

    Ok(steps)
}

enum ShutdownAttr {
    Shutdown,
    ShutdownTake,
    ShutdownPinned,
    ShutdownPinnedTake,
}

fn parse_shutdown_attr(attr: &Attribute) -> Result<Option<ShutdownAttr>> {
    if attr.path().is_ident("shutdown") {
        return parse_shutdown_take_arg(attr, ShutdownAttr::Shutdown, ShutdownAttr::ShutdownTake);
    }

    if attr.path().is_ident("shutdown_pinned") {
        return parse_shutdown_take_arg(attr, ShutdownAttr::ShutdownPinned, ShutdownAttr::ShutdownPinnedTake);
    }

    Ok(None)
}

fn parse_shutdown_take_arg(attr: &Attribute, plain: ShutdownAttr, take: ShutdownAttr) -> Result<Option<ShutdownAttr>> {
    match &attr.meta {
        Meta::Path(_) => Ok(Some(plain)),
        Meta::List(_) => {
            let ident: Ident = attr.parse_args()?;
            if ident == "take" {
                Ok(Some(take))
            } else {
                Err(Error::new_spanned(ident, "expected `take`"))
            }
        }
        _ => Err(Error::new_spanned(
            attr,
            "supported forms are #[shutdown], #[shutdown(take)], #[shutdown_pinned], and #[shutdown_pinned(take)]",
        )),
    }
}
