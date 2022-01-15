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
    struct Params {
        yield_ty: Type,
        resume: Option<(Type, Option<Pat>)>,
    }
    impl Parse for Params {
        fn parse (input: ParseStream<'_>)
          -> Result<Params>
        {
            mod kw {
                ::syn::custom_keyword!(resume);
            }
            let mut yield_ty: Option<Type> = None;
            let mut resume: Option<(Type, Option<Pat>)> = None;
            while input.is_empty().not() {
                let snoopy = input.lookahead1();
                match () {
                    | _case if snoopy.peek(Token![yield]) => {
                        if yield_ty.is_some() {
                            return Err(input.error("already provided"));
                        }
                        let _: Token![yield] = input.parse().unwrap();
                        let parenthesized; parenthesized!(parenthesized in input);
                        yield_ty.replace(parenthesized.parse()?);
                        let _: Option<Token![,]> = parenthesized.parse()?;
                    },
                    | _case if snoopy.peek(kw::resume) => {
                        if resume.is_some() {
                            return Err(input.error("already provided"));
                        }
                        let _: kw::resume = input.parse().unwrap();
                        let resume_ty: Type = {
                            let parenthesized; parenthesized!(parenthesized in input);
                            let it = parenthesized.parse()?;
                            let _: Option<Token![,]> = parenthesized.parse()?;
                            it
                        };
                        let mut resume_pat = None;
                        if input.parse::<Option<Token![as]>>()?.is_some() {
                            resume_pat.replace(input.parse()?);
                        }
                        resume.replace((resume_ty, resume_pat));
                    },
                    // Slightly improve the error message for extraneous
                    // trailing stuff.
                    | _case if yield_ty.is_some() && resume.is_some() => break,
                    | _default => return Err(snoopy.error()),
                }
                let _: Option<Token![,]> = input.parse()?;
            }
            let yield_ty = if let Some(it) = yield_ty { it } else {
                return Err(input.error("missing `yield(<yield type>)`"));
            };
            Ok(Self { yield_ty, resume })
        }
    }

    let Params {
        yield_ty: YieldTy @ _,
        resume,
    } = parse2(params)?;
    let mut fun: ItemFn = parse2(input)?;
    let ItemFn {
        ref mut block,
        ref mut sig,
        ..
    } = fun;

    let __yield_slot__ = Ident::new(
        "__yield_slot__",
        ::proc_macro::Span::mixed_site().into(),
    );

    // Handle the signature
    let resume_arg_pat = {
        sig.asyncness = parse_quote!( async );

        if let Some(receiver) = sig.receiver() {
            return Err(Error::new_spanned(
                receiver,
                "`self` receivers are not supported yet",
            ));
        }

        let (/* mut */ each_pat, /* mut */ EachTy @ _): (Vec<_>, Vec<_>) =
            ::core::mem::take(&mut sig.inputs)
                .into_iter()
                .map(|fn_arg| match fn_arg {
                    | FnArg::Receiver(_) => unreachable!(),
                    | FnArg::Typed(PatType { pat, ty, .. }) => (*pat, ty),
                })
                .unzip()
        ;
        let (resume_arg_pat, ResumeArg @ _) = match resume {
            | Some((ResumeArg @ _, initial_resume_arg_pat)) => (
                initial_resume_arg_pat.unwrap_or_else(|| parse_quote!( _ )),
                ResumeArg.into_token_stream(),
            ),
            | None => (
                parse_quote!( _ ),
                quote!(),
            ),
        };
        sig.inputs = parse_quote!(
            #__yield_slot__: ::next_gen::__::__Internals_YieldSlot_DoNotUse__<'_, #YieldTy, #ResumeArg>,
            (
                #(#each_pat ,)*
            ): (
                #(#EachTy ,)*
            ),
        );
        resume_arg_pat
    };

    // Update block to generate `yield_!` macro.
    {
        *block = parse_quote!({
            macro_rules! yield_ {(
                $value:expr $(,)?
            ) => (
                #__yield_slot__.__put($value).await
            )}

            let #resume_arg_pat = #__yield_slot__.__take_initial_arg();

            #block
        });
    }

    Ok(fun.into_token_stream())
}
