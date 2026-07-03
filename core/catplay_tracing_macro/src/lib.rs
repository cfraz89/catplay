use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn trace_time(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut f: syn::ItemFn = syn::parse_macro_input!(item);

    f.attrs.retain(|a| !a.path().is_ident("trace_time"));

    let level: syn::Ident = if attr.is_empty() {
        syn::parse_quote!(debug)
    } else {
        syn::parse_macro_input!(attr as syn::Ident)
    };

    let name = syn::LitStr::new(&f.sig.ident.to_string(), f.sig.ident.span());
    let block = &f.block;

    if cfg!(feature = "std") {
        *f.block = syn::parse_quote!({

            let __trace_start = std::time::Instant::now();

            struct __TraceGuard {
                start: std::time::Instant,
                name: &'static str,
            }

            impl Drop for __TraceGuard {
                fn drop(&mut self) {
                    log::#level!(
                        "[trace_time] {} took {:?}",
                        self.name,
                        self.start.elapsed()
                    );
                }
            }

            let _trace_guard = __TraceGuard {
                start: __trace_start,
                name: #name,
            };

            log::#level!("[trace_time] {} entering", #name);

            #block
        });
    } else {
        *f.block = syn::parse_quote!({
            #block
        });
    }

    quote::quote!(#f).into()
}
