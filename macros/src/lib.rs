extern crate derive_syn_parse;
extern crate ed25519_dalek;
extern crate quote;
extern crate syn;

use std::{
    collections::{hash_map::DefaultHasher, HashSet},
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
    visit::{self, Visit},
    ExprArray, FieldValue, Ident, Item, ItemMod, ItemUse, LitBool, LitStr, Path, Token,
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
    allow_unaudited_foreign_paths: bool,
    allowed_foreign_paths: HashSet<String>,
}

impl ParsedArgs {
    pub fn new() -> ParsedArgs {
        ParsedArgs {
            signature: None,
            timestamp: None,
            signed_by: None,
            public_key: None,
            allow_use: false,
            allow_extern_crate: false,
            allow_unaudited_foreign_paths: false,
            allowed_foreign_paths: HashSet::new(),
        }
    }
}

struct PathVisitor<'ast> {
    paths: Vec<&'ast Path>,
}

impl<'ast> PathVisitor<'ast> {
    pub fn new() -> PathVisitor<'ast> {
        PathVisitor { paths: Vec::new() }
    }
}

impl<'ast> Visit<'ast> for PathVisitor<'ast> {
    fn visit_path(&mut self, path: &'ast Path) {
        self.paths.push(path);
        visit::visit_path(self, path);
    }
}

fn scan_module_for_foreign_paths<'ast>(item: &ItemMod) -> Vec<Path> {
    // get all paths in mod
    let mut path_visitor = PathVisitor::new();
    path_visitor.visit_item_mod(item);

    // get top level item names
    let mut top_level_item_names: HashSet<String> = HashSet::new();
    if let Some(content) = &item.content {
        for child in &content.1 {
            if let Some(ident) = match child {
                Item::Const(child) => Some(&child.ident),
                Item::Enum(child) => Some(&child.ident),
                Item::ExternCrate(child) => Some(&child.ident),
                Item::Fn(child) => Some(&child.sig.ident),
                Item::Macro(child) => {
                    if let Some(ident) = &child.ident {
                        Some(ident)
                    } else {
                        None
                    }
                }
                Item::Macro2(child) => Some(&child.ident),
                Item::Mod(child) => Some(&child.ident),
                Item::Static(child) => Some(&child.ident),
                Item::Struct(child) => Some(&child.ident),
                Item::Trait(child) => Some(&child.ident),
                Item::TraitAlias(child) => Some(&child.ident),
                Item::Type(child) => Some(&child.ident),
                Item::Union(child) => Some(&child.ident),
                _ => None,
            } {
                top_level_item_names.insert(ident.to_string());
            }
        }
    }

    // find paths that don't have a top level mod name as their first segment
    let mut foreign_paths: Vec<Path> = Vec::new();
    for path in path_visitor.paths {
        if let Some(segment) = path.segments.first() {
            let name = segment.ident.to_string();
            if !top_level_item_names.contains(&name) {
                foreign_paths.push(path.clone());
            }
        }
    }
    foreign_paths
}

#[proc_macro_attribute]
pub fn audited(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut parsed = ParsedArgs::new();
    let args = parse_macro_input!(attr as RawArgs).args;
    for arg in args.clone().into_iter() {
        let field = arg.member.to_token_stream().into();
        let field = parse_macro_input!(field as Ident).to_string();
        let value = arg.expr.to_token_stream().into();
        match field.as_str() {
            "allowed_foreign_paths" => {
                let arr = parse_macro_input!(value as ExprArray);
                for lit in arr.elems {
                    let lit = lit.to_token_stream().into();
                    let path = parse_macro_input!(lit as Path)
                        .to_token_stream()
                        .to_string();
                    parsed.allowed_foreign_paths.insert(path);
                }
            }
            // bool args
            "allow_use" | "allow_extern_crate" | "allow_unaudited_foreign_paths" => {
                *match field.as_str() {
                    "allow_use" => &mut parsed.allow_use,
                    "allow_extern_crate" => &mut parsed.allow_extern_crate,
                    "allow_unaudited_foreign_paths" => &mut parsed.allow_unaudited_foreign_paths,
                    _ => panic!("unreachable"),
                } = parse_macro_input!(value as LitBool).value();
            }
            // string args
            "sig" | "timestamp" | "signed_by" | "public" => {
                *match field.as_str() {
                    "sig" => &mut parsed.signature,
                    "timestamp" => &mut parsed.timestamp,
                    "signed_by" => &mut parsed.signed_by,
                    "public" => &mut parsed.public_key,
                    _ => panic!("unreachable"),
                } = Some(parse_macro_input!(value as LitStr).value());
            }
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
            if let Some(content) = &item.content {
                for child in &content.1 {
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
            if !parsed.allow_unaudited_foreign_paths {
                let foreign_paths = scan_module_for_foreign_paths(&item);
                if let Some(foreign_path) = foreign_paths.first() {
                    return emit_error(&foreign_path.to_token_stream(), "This path has not been marked as audited \
                        and unaudited foreign paths have been disabled for this module. Please annotate a `use` \
                        statement that brings this path into scope with `#[audited_use]` or add this path to the \
                        `allowed_foreign_paths` list for this module.");
                }
            }
        }
        _ => {
            return emit_error(&item, "can only be applied to a module");
        }
    }
    println!("hash: {}", hash);
    println!("parsed: {:#?}", parsed);
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
