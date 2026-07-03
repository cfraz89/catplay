use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{
    Attribute, Data, DeriveInput, Error, Expr, Fields, LitStr, Meta, Path, Result, Token, parse::Parse, parse::ParseStream, parse_quote,
    punctuated::Punctuated, spanned::Spanned,
};

use crate::common::add_self_send_bound;

pub fn expand(input: &DeriveInput) -> Result<TokenStream2> {
    let ident = &input.ident;
    let crate_path = parse_crate_path(&input.attrs)?;
    let mut branches = parse_struct_sleep_branches(&input.attrs, &crate_path)?;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(Error::new_spanned(
                    input,
                    "EventSleeper can only be derived for structs with named fields",
                ));
            }
        },
        _ => return Err(Error::new_spanned(input, "EventSleeper can only be derived for structs")),
    };

    let mut field_branches = Vec::new();
    for field in fields {
        let Some(field_ident) = &field.ident else {
            continue;
        };
        let attrs = parse_field_attrs(&field.attrs)?;

        for attr in attrs {
            let branch = match attr {
                FieldSleepAttr::Sleep => quote_spanned! { field_ident.span()=> self.#field_ident },
                FieldSleepAttr::SleepPinned => {
                    quote_spanned! { field_ident.span()=> #crate_path::dyn_sleeper(&mut self.#field_ident) }
                }
                FieldSleepAttr::Slot(expr) => {
                    quote_spanned! { field_ident.span()=> #crate_path::filling_slot(&mut self.#field_ident, #expr) }
                }
                FieldSleepAttr::SlotValue(expr) => {
                    quote_spanned! { field_ident.span()=> #crate_path::filling_slot_value(&mut self.#field_ident, #expr) }
                }
                FieldSleepAttr::SlotMap(expr, map) => {
                    quote_spanned! { field_ident.span()=> #crate_path::filling_slot_map(&mut self.#field_ident, #expr, #map) }
                }
            };
            field_branches.push(branch);
        }
    }

    let mut all_branches = field_branches;
    all_branches.append(&mut branches);

    if all_branches.is_empty() {
        return Err(Error::new_spanned(
            input,
            "EventSleeper derive requires at least one #[sleep], #[slot(...)], #[slot_value(...)], #[slot_map(...)], #[sleep(...)], or #[sleep_fut(...)] branch",
        ));
    }

    let mut generics = input.generics.clone();
    add_self_send_bound(ident, &mut generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #crate_path::EventSleeper for #ident #ty_generics #where_clause {
            async fn sleep(&mut self) -> Option<#crate_path::EventToken> {
                #crate_path::event_select!(#(#all_branches),*)
            }
        }
    })
}

fn parse_crate_path(attrs: &[Attribute]) -> Result<Path> {
    let mut crate_path: Path = parse_quote!(::catplay_util);

    for attr in attrs {
        if !attr.path().is_ident("event_sleeper") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("crate") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                crate_path = lit.parse()?;
                Ok(())
            } else {
                Err(meta.error("unsupported event_sleeper option"))
            }
        })?;
    }

    Ok(crate_path)
}

fn parse_struct_sleep_branches(attrs: &[Attribute], crate_path: &Path) -> Result<Vec<TokenStream2>> {
    let mut branches = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("sleep") {
            match &attr.meta {
                Meta::List(_) => {
                    let expr: Expr = attr.parse_args()?;
                    branches.push(quote_spanned! { attr.span()=> #expr });
                }
                _ => {
                    return Err(Error::new_spanned(
                        attr,
                        "struct-level #[sleep] must include an expression, for example #[sleep(deadline_after(duration))]",
                    ));
                }
            }
        } else if attr.path().is_ident("sleep_fut") {
            match &attr.meta {
                Meta::List(_) => {
                    let expr: Expr = attr.parse_args()?;
                    branches.push(quote_spanned! { attr.span()=> #crate_path::sleeper(#expr) });
                }
                _ => {
                    return Err(Error::new_spanned(
                        attr,
                        "struct-level #[sleep_fut] must include a future expression, for example #[sleep_fut(self.watch.changed())]",
                    ));
                }
            }
        }
    }

    Ok(branches)
}

enum FieldSleepAttr {
    Sleep,
    SleepPinned,
    Slot(Expr),
    SlotValue(Expr),
    SlotMap(Expr, Expr),
}

fn parse_field_attrs(attrs: &[Attribute]) -> Result<Vec<FieldSleepAttr>> {
    let mut parsed = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("sleep") {
            match &attr.meta {
                Meta::Path(_) => parsed.push(FieldSleepAttr::Sleep),
                _ => {
                    return Err(Error::new_spanned(
                        attr,
                        "field-level #[sleep] does not accept arguments; use struct-level #[sleep(expr)] for computed sleepers",
                    ));
                }
            }
        } else if attr.path().is_ident("sleep_fut") {
            return Err(Error::new_spanned(
                attr,
                "field-level #[sleep_fut] is not supported; use struct-level #[sleep_fut(expr)] for future-backed sleepers",
            ));
        } else if attr.path().is_ident("sleep_pinned") {
            match &attr.meta {
                Meta::Path(_) => parsed.push(FieldSleepAttr::SleepPinned),
                _ => {
                    return Err(Error::new_spanned(attr, "field-level #[sleep_pinned] does not accept arguments"));
                }
            }
        } else if attr.path().is_ident("slot") {
            parsed.push(FieldSleepAttr::Slot(attr.parse_args()?));
        } else if attr.path().is_ident("slot_value") {
            parsed.push(FieldSleepAttr::SlotValue(attr.parse_args()?));
        } else if attr.path().is_ident("slot_map") {
            let args: TwoExprArgs = attr.parse_args()?;
            parsed.push(FieldSleepAttr::SlotMap(args.expr, args.map));
        }
    }

    Ok(parsed)
}

struct TwoExprArgs {
    expr: Expr,
    map: Expr,
}

impl Parse for TwoExprArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let args = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;
        if args.len() != 2 {
            return Err(Error::new(input.span(), "expected exactly two expressions"));
        }

        let mut args = args.into_iter();
        Ok(Self {
            expr: args.next().expect("first expression must exist"),
            map: args.next().expect("second expression must exist"),
        })
    }
}
