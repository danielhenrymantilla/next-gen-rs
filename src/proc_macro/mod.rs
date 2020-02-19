#![cfg_attr(feature = "external_doc",
    feature(external_doc),
)]
#![doc(test(attr(deny(warnings))))]

#[macro_use]
extern crate fstrings;
extern crate proc_macro;

use ::proc_macro::TokenStream;
use quote::{
    quote,
    quote_spanned,
};
use ::syn::{*,
    spanned::Spanned,
};

#[macro_use]
mod macros;

const IDENT_SUFFIX: &'static str = "__hack__";
/// Transforms a function with `yield_!` calls into a generator.
#[cfg_attr(feature = "external_doc",
    doc = "",
    doc = "# Example",
    doc = "",
    doc = "```rust",
    doc = "# macro_rules! ignore {($($tt:tt)*) => ()} ignore! {",
    doc(include = "doc_examples/generator.rs"),
    doc = "# }",
    doc = "```",
)]
///
/// # Expansion
///
/// The above example expands to:
#[cfg_attr(feature = "external_doc",
    doc = "",
    doc = "```rust",
    doc = "# macro_rules! ignore {($($tt:tt)*) => ()} ignore! {",
    doc(include = "doc_examples/generator_desugared.rs"),
    doc = "# }",
    doc = "```",
)]
#[proc_macro_attribute] pub
fn generator (params: TokenStream, input: TokenStream)
  -> TokenStream
{
    #[allow(unused)]
    const FUNCTION_NAME: &str = "generator";

    let yield_type: Type = parse_macro_input!(params as Type);

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
                enum __yield_slot____hack__ {}

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
            | Some(ty) => return {
                Error::new(ty.span(), &f!(
                    "`#[{FUNCTION_NAME}]` does not support `self` receivers yet"
                )).to_compile_error().into()
            },

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
            __yield_slot__: next_gen::__Internals_YieldSlot_DoNotUse__<'_, #yield_type>,
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
    let ident: String = input.ident.to_string();
    let ident: &str = &ident[.. ident.len() - IDENT_SUFFIX.len()];
    let yield_slot = Ident::new(ident, input.ident.span());
    TokenStream::from(quote_spanned! { input.span() =>
        macro_rules! yield_ {(
            $value:expr
        ) => ({
            let () = #yield_slot.put($value).await;
        })}
    })
}
