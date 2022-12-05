extern crate derive_syn_parse;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    FieldValue, Ident, LitBool, LitStr, Token,
};

struct RawArgs {
    args: Punctuated<FieldValue, Token![,]>,
}

impl Parse for RawArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(RawArgs {
            args: Punctuated::<FieldValue, Token![,]>::parse_terminated(input)?,
        })
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct ParsedArgs {
    signature: Option<String>,
    timestamp: Option<String>,
    signed_by: Option<String>,
    public_key: Option<String>,
    allow_use: bool,
}

const DEFAULT_ARGS: ParsedArgs = ParsedArgs {
    signature: None,
    timestamp: None,
    signed_by: None,
    public_key: None,
    allow_use: false,
};

#[proc_macro_attribute]
pub fn audited(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut parsed = DEFAULT_ARGS;
    let args = parse_macro_input!(attr as RawArgs).args;
    for arg in args.clone().into_iter() {
        let field = arg.member.to_token_stream().into();
        let field = parse_macro_input!(field as Ident).to_string();
        let value = arg.expr.to_token_stream().into();
        if field.as_str() == "allow_use" {
            let value = parse_macro_input!(value as LitBool).value();
            parsed.allow_use = value;
            continue;
        }
        let value = parse_macro_input!(value as LitStr).value();
        match field.as_str() {
            "sig" => parsed.signature = Some(value),
            "timestamp" => parsed.timestamp = Some(value),
            "signed_by" => parsed.signed_by = Some(value),
            "public" => parsed.public_key = Some(value),
            _ => {
                let span = proc_macro2::TokenStream::from(arg.to_token_stream()).span();
                return syn::Error::new(span, "invalid attribute")
                    .to_compile_error()
                    .into();
            }
        }
    }
    if parsed.signature == None {
        let span = proc_macro2::TokenStream::from(args.to_token_stream()).span();
        return syn::Error::new(span, "sig must be specified")
            .to_compile_error()
            .into();
    }
    if parsed.public_key == None {
        let span = proc_macro2::TokenStream::from(args.to_token_stream()).span();
        return syn::Error::new(span, "public must be specified")
            .to_compile_error()
            .into();
    }
    println!("{:#?}", parsed);
    item
}
