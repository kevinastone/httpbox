use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_attribute]
pub fn async_handler(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    let vis = input.vis;
    let ident = input.sig.ident;
    let body = input.block;
    let params = input.sig.inputs;

    let tokens = quote! {
        #vis fn #ident(#params) -> Box<::gotham::handler::HandlerFuture> {
            Box::new(
                ::futures::future::TryFutureExt::compat(
                    ::futures::future::FutureExt::boxed(
                        ::futures::future::FutureExt::then(
                            async move {
                               #body
                            }, |result| {
                                ::futures::compat::Future01CompatExt::compat(
                                    ::gotham::handler::IntoHandlerFuture::into_handler_future(
                                        result
                                    )
                                )
                            }
                        )
                    )
                )
            )
        }

    };
    tokens.into()
}
