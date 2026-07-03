use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod async_shutdown;
mod common;
mod event_reconciler;
mod event_sleeper;

#[proc_macro_derive(AsyncShutdown, attributes(async_shutdown, shutdown_func, shutdown, shutdown_pinned))]
pub fn derive_async_shutdown(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match async_shutdown::expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(
    EventSleeper,
    attributes(event_sleeper, sleep, sleep_pinned, sleep_fut, slot, slot_value, slot_map)
)]
pub fn derive_event_sleeper(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match event_sleeper::expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(
    EventReconciler,
    attributes(event_reconciler, reconcile_error, reconcile_func, reconcile, reconcile_pop)
)]
pub fn derive_event_reconciler(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match event_reconciler::expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
