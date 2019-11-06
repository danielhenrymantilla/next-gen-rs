#[macro_use]
extern crate fstrings;
extern crate proc_macro;

use ::proc_macro::TokenStream;
use ::proc_macro2::{
    Span,
    // TokenStream as TokenStream2,
};
use quote::{
    // ToTokens,
    quote,
    quote_spanned,
};
use ::syn::{*,
    spanned::Spanned,
    // parse::Parse,
    // punctuated::Punctuated,
    // error::Error,
};

#[macro_use]
mod macros;

const IDENT_SUFFIX: &'static str = "__hack__";

#[proc_macro_attribute] pub
fn generator (params: TokenStream, input: TokenStream)
  -> TokenStream
{
    #[allow(unused)]
    const FUNCTION_NAME: &str = "generator";

    let yield_type = {
        let params = parse_macro_input!(params as AttributeArgs);
        match params.len() {
            | 0 => return Error::new(
                Span::call_site(),
                &f!("Missing parameter: expected `#[{FUNCTION_NAME}(<type>)]`"),
            ).to_compile_error().into(),

            | 1 => {
                let arg = {params}.pop().unwrap();
                match arg {
                    | NestedMeta::Meta(Meta::Path(path)) => path,
                    | _ => return Error::new(
                        arg.span(),
                        "Expected a type",
                    ).to_compile_error().into(),
                }
            },

            | _ => return Error::new(
                {params}.pop().unwrap().span(),
                &f!("Too many parameters"),
            ).to_compile_error().into(),
        }
    };

    debug_input!(&input);
    let mut function: ItemFn = parse_macro_input!(input);
    let ItemFn {
        ref mut block,
        ref mut sig,
        ..
    } = function;

    // Update block to generate `yield_!` macro.
    {
        *block = parse_quote! {
            {
                #[derive(next_gen::next_gen_hack)]
                enum __coroutine____hack__ {}

                #block
            }
        };
    }

    // Handle the signature
    {
        sig.asyncness = parse_quote! { async };

        match sig
                .inputs
                .iter()
                .find(|&fn_arg| match *fn_arg {
                    | FnArg::Receiver(_) => true,
                    | _ => false,
                })
        {
            | Some(ty) => return Error::new(
                ty.span(),
                &f!("`#[{FUNCTION_NAME}]` does not support `self` receivers yet"),
            ).to_compile_error().into(),

            | _ => {},
        }

        let (pats, tys): (Vec<_>, Vec<_>) =
            ::core::mem::replace(&mut sig.inputs, Default::default())
                .into_iter()
                .map(|fn_arg| match fn_arg {
                    | FnArg::Receiver(_) => unreachable!(),
                    | FnArg::Typed(PatType { pat, ty, .. }) => (pat, ty),
                })
                .unzip()
        ;
        sig.inputs = parse_quote! {
            __coroutine__: next_gen::Coroutine<'_, #yield_type >,
            ( #(#pats ,)* ): ( #(#tys ,)* ),
        };
    }

    TokenStream::from(debug_output!(quote! {
        #function
    }))
}


#[doc(hidden)]
#[proc_macro_derive(next_gen_hack)] pub
fn hack (input: TokenStream) -> TokenStream
{
    let input: DeriveInput = parse_macro_input!(input);
    let ident = input.ident.to_string();
    let ident = &ident[.. ident.len() - IDENT_SUFFIX.len()];
    let co = Ident::new(ident, input.ident.span());
    TokenStream::from(quote_spanned! { input.span() =>
        macro_rules! yield_ {(
            $value:expr
        ) => (
            #co._yield($value).await
        )}
    })
}
