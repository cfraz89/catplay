use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{
    Attribute, Data, DeriveInput, Error, Expr, Fields, Ident, LitStr, Meta, Path, Result, Token, Type,
    parse::{Parse, ParseStream},
    parse_quote,
    spanned::Spanned,
};

use crate::common::add_self_send_bound;

pub fn expand(input: &DeriveInput) -> Result<TokenStream2> {
    let ident = &input.ident;
    let crate_path = parse_crate_path(&input.attrs)?;
    let error_ty = parse_reconcile_error(&input.attrs)?;
    let mut steps = Vec::new();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(Error::new_spanned(
                    input,
                    "EventReconciler can only be derived for structs with named fields",
                ));
            }
        },
        _ => return Err(Error::new_spanned(input, "EventReconciler can only be derived for structs")),
    };

    for field in fields {
        let Some(field_ident) = &field.ident else {
            continue;
        };

        for attr in &field.attrs {
            let step = if attr.path().is_ident("reconcile") {
                match &attr.meta {
                    Meta::Path(_) => {
                        quote_spanned! { field_ident.span()=>
                            #crate_path::EventReconciler::reconcile(&mut self.#field_ident).await?;
                        }
                    }
                    Meta::List(_) => {
                        let mapper: Expr = attr.parse_args()?;
                        quote_spanned! { field_ident.span()=>
                            #crate_path::EventReconciler::reconcile(&mut self.#field_ident)
                                .await
                                .map_err(|e| (#mapper)(e.into()))?;
                        }
                    }
                    _ => {
                        return Err(Error::new_spanned(
                            attr,
                            "field-level #[reconcile] accepts either no arguments or one error mapper expression",
                        ));
                    }
                }
            } else if attr.path().is_ident("reconcile_pop") {
                match &attr.meta {
                    Meta::Path(_) => {
                        quote_spanned! { field_ident.span()=>
                            if let Some(err) = self.#field_ident.take() {
                                return Err(err.into());
                            }
                        }
                    }
                    Meta::List(_) => {
                        let mode: Ident = attr.parse_args()?;
                        if mode != "clone" {
                            return Err(Error::new_spanned(
                                mode,
                                "field-level #[reconcile_pop] only supports the `clone` option",
                            ));
                        }

                        quote_spanned! { field_ident.span()=>
                            if let Some(err) = self.#field_ident.as_ref() {
                                return Err(err.clone().into());
                            }
                        }
                    }
                    _ => {
                        return Err(Error::new_spanned(
                            attr,
                            "field-level #[reconcile_pop] accepts either no arguments or `clone`",
                        ));
                    }
                }
            } else {
                continue;
            };
            steps.push(step);
        }
    }

    steps.extend(parse_reconcile_funcs(&input.attrs)?);

    if steps.is_empty() {
        return Err(Error::new_spanned(
            input,
            "EventReconciler derive requires at least one #[reconcile] field, #[reconcile_pop] field, or #[reconcile_func(...)]",
        ));
    }

    let mut generics = input.generics.clone();
    add_self_send_bound(ident, &mut generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #crate_path::EventReconciler for #ident #ty_generics #where_clause {
            type Error = #error_ty;

            #[allow(clippy::useless_conversion, clippy::redundant_closure_call)]
            async fn reconcile(&mut self) -> Result<(), Self::Error> {
                #(#steps)*
                Ok(())
            }
        }
    })
}

fn parse_crate_path(attrs: &[Attribute]) -> Result<Path> {
    let mut crate_path: Path = parse_quote!(::catplay_util);

    for attr in attrs {
        if !attr.path().is_ident("event_reconciler") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("crate") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                crate_path = lit.parse()?;
                Ok(())
            } else {
                Err(meta.error("unsupported event_reconciler option"))
            }
        })?;
    }

    Ok(crate_path)
}

fn parse_reconcile_error(attrs: &[Attribute]) -> Result<Type> {
    for attr in attrs {
        if attr.path().is_ident("reconcile_error") {
            return attr.parse_args();
        }
    }

    Err(Error::new(
        proc_macro2::Span::call_site(),
        "EventReconciler derive requires #[reconcile_error(ErrorType)]",
    ))
}

fn parse_reconcile_funcs(attrs: &[Attribute]) -> Result<Vec<TokenStream2>> {
    let mut steps = Vec::new();

    for attr in attrs {
        if !attr.path().is_ident("reconcile_func") {
            continue;
        }

        let ReconcileFuncArgs { func, mapper } = attr.parse_args()?;
        let step = match mapper {
            Some(mapper) => quote_spanned! { attr.span()=>
                Self::#func(self)
                    .await
                    .map_err(|e| (#mapper)(e.into()))?;
            },
            None => quote_spanned! { attr.span()=>
                Self::#func(self).await?;
            },
        };
        steps.push(step);
    }

    Ok(steps)
}

struct ReconcileFuncArgs {
    func: Ident,
    mapper: Option<Expr>,
}

impl Parse for ReconcileFuncArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let func = input.parse()?;
        let mapper = if input.is_empty() {
            None
        } else {
            input.parse::<Token![,]>()?;
            Some(input.parse()?)
        };

        Ok(Self { func, mapper })
    }
}
