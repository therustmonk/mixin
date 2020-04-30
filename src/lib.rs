use once_cell::sync::Lazy;
use proc_macro::{TokenStream, TokenTree};
use quote::quote;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use syn::parse_macro_input::ParseMacroInput;
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, Attribute, AttributeArgs, Data,
    DeriveInput, Field, Fields, Meta, NestedMeta,
};
use thiserror::Error;

#[derive(Error, Debug)]
enum Error {
    #[error("global data unavailable")]
    GlobalUnavailable,
    #[error("token stream has been depleted")]
    StreamDepleted,
    #[error("can't find mixin with name: {0}")]
    NoMixin(String),
    #[error("syn error: {0}")]
    SynError(#[from] syn::Error),
    #[error("lex error: {0}")]
    LexError(#[from] proc_macro::LexError),
}

impl Error {
    fn to_compile_error(self) -> TokenStream {
        todo!()
    }
}

fn rec_printer(stream: TokenStream) {
    for tt in stream.into_iter() {
        match tt {
            TokenTree::Group(group) => {
                println!("GROUP");
                rec_printer(group.stream());
                println!("END GROUP");
            }
            _ => {
                println!("USE {:?}", tt);
            }
        }
    }
}

#[proc_macro_attribute]
pub fn mixin(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    mixin_impl(args, input).unwrap_or_else(Error::to_compile_error)
}

fn mixin_impl(args: AttributeArgs, input: TokenStream) -> Result<TokenStream, Error> {
    let mut mixin_names = HashSet::new();
    for nested_meta in args {
        if let NestedMeta::Meta(meta) = nested_meta {
            if let Meta::Path(path) = meta {
                for path_segment in path.segments.iter() {
                    mixin_names.insert(path_segment.ident.to_string());
                }
            }
        }
    }
    let data = GLOBAL_DATA.lock().map_err(|_| Error::GlobalUnavailable)?;
    let mut punctuated = None;
    for mixin_name in mixin_names {
        let mixin_source = data
            .get(&mixin_name)
            .ok_or_else(|| Error::NoMixin(mixin_name))?;
        let input: TokenStream = mixin_source.parse()?;
        let the_mixin: DeriveInput = syn::parse(input)?;
        if let Data::Struct(st) = the_mixin.data {
            if let Fields::Named(named) = st.fields {
                //let punctuated: Punctuated<Field, Comma> = named.named;
                punctuated = Some(named.named);
            }
        }
    }

    let mut the_struct: DeriveInput = syn::parse(input)?;
    if let Data::Struct(ref mut st) = the_struct.data {
        if let Fields::Named(ref mut named) = st.fields {
            if let Some(punc) = punctuated {
                named.named.extend(punc.into_pairs());
            }
        }
    }

    let output = quote! {
        #the_struct
    };

    //println!("DATA: {}", output.to_string());

    Ok(TokenStream::from(output))
}

static GLOBAL_DATA: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[proc_macro_attribute]
pub fn mixin_new(attribute: TokenStream, input: TokenStream) -> TokenStream {
    mixin_new_impl(attribute, input).unwrap_or_else(Error::to_compile_error)
}

fn mixin_new_impl(attribute: TokenStream, input: TokenStream) -> Result<TokenStream, Error> {
    // Consume the struct
    let s = input.to_string();
    let input: DeriveInput = syn::parse(input).unwrap();
    let name = input.ident.to_string();
    let mut data = GLOBAL_DATA.lock().map_err(|_| Error::GlobalUnavailable)?;
    data.insert(name, s);
    // And give the empty output back
    Ok(TokenStream::new())
}

/*
    let input = parse_macro_input!(input as DeriveInput);
    if let Data::Struct(st) = input.data {
        if let Fields::Named(named) = st.fields {
            let punctuated: Punctuated<Field, Comma> = named.named;
        }
    }
*/
