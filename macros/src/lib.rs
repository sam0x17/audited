extern crate derive_syn_parse;
extern crate ed25519_dalek;
extern crate quote;
extern crate syn;

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use derive_syn_parse::Parse;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    FieldValue, Ident, Item, ItemUse, LitBool, LitStr, Token,
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

fn generate_hash<T: Into<Item> + Clone>(item: &T) -> u64 {
    let item = item.clone();
    let mut hasher = DefaultHasher::new();
    let item = Into::<Item>::into(item);
    item.hash(&mut hasher);
    hasher.finish()
}

fn emit_error<T: Into<TokenStream> + Clone, S: Into<String>>(item: &T, message: S) -> TokenStream {
    let item = Into::<TokenStream>::into(item.clone());
    let message = Into::<String>::into(message);
    let span = proc_macro2::TokenStream::from(item).span();
    return syn::Error::new(span, message).to_compile_error().into();
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct ParsedArgs {
    signature: Option<String>,
    timestamp: Option<String>,
    signed_by: Option<String>,
    public_key: Option<String>,
    allow_use: bool,
    allow_extern_crate: bool,
}

const DEFAULT_ARGS: ParsedArgs = ParsedArgs {
    signature: None,
    timestamp: None,
    signed_by: None,
    public_key: None,
    allow_use: false,
    allow_extern_crate: false,
};

#[proc_macro_attribute]
pub fn audited(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut parsed = DEFAULT_ARGS;
    let args = parse_macro_input!(attr as RawArgs).args;
    for arg in args.clone().into_iter() {
        let field = arg.member.to_token_stream().into();
        let field = parse_macro_input!(field as Ident).to_string();
        let value = arg.expr.to_token_stream().into();
        match field.as_str() {
            // bool args
            "allow_use" | "allow_extern_crate" => {
                let value = parse_macro_input!(value as LitBool).value();
                match field.as_str() {
                    "allow_use" => parsed.allow_use = value,
                    "allow_extern_crate" => parsed.allow_extern_crate = value,
                    _ => panic!("invalid state"),
                }
                continue;
            }
            _ => {}
        }
        let value = parse_macro_input!(value as LitStr).value();
        match field.as_str() {
            // string args
            "sig" => parsed.signature = Some(value),
            "timestamp" => parsed.timestamp = Some(value),
            "signed_by" => parsed.signed_by = Some(value),
            "public" => parsed.public_key = Some(value),
            _ => {
                return emit_error(&arg.to_token_stream(), "invalid attribute");
            }
        }
    }
    if parsed.signature == None {
        return emit_error(&args.to_token_stream(), "sig is required");
    }
    if parsed.public_key == None {
        return emit_error(&args.to_token_stream(), "public is required");
    }
    let tmp = item.clone();
    let item_parsed = parse_macro_input!(tmp as Item);
    let hash = generate_hash(&item_parsed);
    match item_parsed {
        Item::Mod(item) => {
            if let Some(content) = item.content {
                for child in content.1 {
                    match child {
                        Item::ExternCrate(_) => {
                            if !parsed.allow_extern_crate {
                                return emit_error(
                                    &child.to_token_stream(),
                                    "`extern crate` has been disabled for this module by the audited crate",
                                );
                            }
                        }
                        Item::Use(_) => {
                            if !parsed.allow_use {
                                return emit_error(
                                    &child.to_token_stream(),
                                    "`use` has been disabled for this module by the audited crate",
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        _ => {
            let span = proc_macro2::TokenStream::from(item).span();
            return syn::Error::new(span, "can only be applied to a module.")
                .to_compile_error()
                .into();
        }
    }
    println!("hash: {}", hash);
    item
}

#[derive(Parse)]
struct AuditedUseArgs {
    hashcode: Option<LitStr>,
}

#[proc_macro_attribute]
pub fn audited_use(args: TokenStream, item: TokenStream) -> TokenStream {
    let item2 = item.clone();
    parse_macro_input!(item2 as ItemUse);
    let args = parse_macro_input!(args as AuditedUseArgs);
    let hashcode: Option<String> = match args.hashcode {
        None => None,
        Some(lit) => Some(lit.value()),
    };
    if let Some(code) = hashcode {
        println!("hashcode: {}", code);
    } else {
        println!("no hashcode");
    }
    item
}
