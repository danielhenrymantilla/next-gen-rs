//! Crate not intended for direct use.
//! Use https://docs.rs/next-gen instead.
// Templated by `cargo-generate` using https://github.com/danielhenrymantilla/proc-macro-template
#![allow(nonstandard_style, unused_imports)]

use ::core::{
    mem,
    ops::Not as _,
};
use ::proc_macro::{
    TokenStream,
};
use ::proc_macro2::{
    Span,
    TokenStream as TokenStream2,
    TokenTree as TT,
};
use ::quote::{
    format_ident,
    quote,
    quote_spanned,
    ToTokens,
};
use ::syn::{*,
    parse::{Parse, Parser, ParseStream},
    punctuated::Punctuated,
    Result, // Explicitly shadow it
    spanned::Spanned,
};

mod utils;

// #[macro_use]
// mod macros;

#[proc_macro_attribute] pub
fn generator (
    attrs: TokenStream,
    input: TokenStream,
) -> TokenStream
{
    generator_impl(attrs.into(), input.into())
        .map(|ret| {
            #[cfg(feature = "verbose-expansions")] {
                utils::pretty_print_tokenstream(&ret);
            }
            ret
        })
        .unwrap_or_else(|err| {
            let mut errors =
                err .into_iter()
                    .map(|err| Error::new(
                        err.span(),
                        format_args!("`#[next_gen::generator]`: {}", err),
                    ))
            ;
            let mut err = errors.next().unwrap();
            errors.for_each(|cur| err.combine(cur));
            err.to_compile_error()
        })
        .into()
}

fn generator_impl (
    params: TokenStream2,
    input: TokenStream2,
) -> Result<TokenStream2>
{
    let yield_type: Type = parse2(params)?;

    let mut function: ItemFn = parse2(input)?;
    let ItemFn {
        ref mut block,
        ref mut sig,
        ..
    } = function;

    // Update block to generate `yield_!` macro.
    {
        *block = parse_quote!({
            macro_rules! yield_ {(
                $value:expr
            ) => (
                __yield_slot__.put($value).await
            )}

            #block
        });
    }

    // Handle the signature
    {
        sig.asyncness = parse_quote!( async );

        match
            sig .inputs
                .iter()
                .find(|&fn_arg| match *fn_arg {
                    | FnArg::Receiver(_) => true,
                    | _ => false,
                })
        {
            | Some(ty) => return Err(Error::new_spanned(
                ty,
                "`self` receivers are not supported yet",
            )),

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
        sig.inputs = parse_quote!(
            __yield_slot__: ::next_gen::__::__Internals_YieldSlot_DoNotUse__<'_, #yield_type>,
            ( #(#pats ,)* ): ( #(#tys ,)* ),
        );
    }

    Ok(function.into_token_stream())
}
