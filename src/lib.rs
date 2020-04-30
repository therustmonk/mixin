use once_cell::sync::Lazy;
use proc_macro::{TokenStream, TokenTree};
use proc_macro2::Span;
use quote::ToTokens;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use syn::{parse_macro_input, AttributeArgs, Data, DeriveInput, Fields, Meta, NestedMeta};
use thiserror::Error;

#[derive(Error, Debug)]
enum Error {
    #[error("global data unavailable")]
    GlobalUnavailable,
    #[error("can't find mixin with name: {0}")]
    NoMixin(String),
    #[error("invalid expansion of the mixin")]
    InvalidExpansion,
    #[error("syn error: {0}")]
    SynError(#[from] syn::Error),
    #[error("lex error: {0}")]
    LexError(#[from] proc_macro::LexError),
}

impl Error {
    fn to_compile_error(self) -> TokenStream {
        let txt = self.to_string();
        let err = syn::Error::new(Span::call_site(), txt).to_compile_error();
        TokenStream::from(err)
    }
}

#[proc_macro_attribute]
pub fn insert(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    insert_impl(args, input).unwrap_or_else(Error::to_compile_error)
}

fn insert_impl(args: AttributeArgs, input: TokenStream) -> Result<TokenStream, Error> {
    let mut the_struct: DeriveInput = syn::parse(input)?;
    let the_struct_name = the_struct.ident.to_string();

    // Get names of mixins to append
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
    let mut mixed_fields = Vec::new();
    let mut mixed_impls = Vec::new();
    for mixin_name in mixin_names {
        let mixin = data
            .get(&mixin_name)
            .ok_or_else(|| Error::NoMixin(mixin_name.clone()))?;
        let input: TokenStream = mixin.declaration.parse()?;
        let the_mixin: DeriveInput = syn::parse(input)?;
        if let Data::Struct(st) = the_mixin.data {
            if let Fields::Named(named) = st.fields {
                mixed_fields.push(named.named);
            }
        }
        for extension in &mixin.extensions {
            let source = extension.replace(&mixin_name, &the_struct_name);
            let stream: TokenStream = source.parse()?;
            mixed_impls.push(stream);
        }
    }

    if let Data::Struct(ref mut st) = the_struct.data {
        if let Fields::Named(ref mut named) = st.fields {
            for fields in mixed_fields {
                named.named.extend(fields.into_pairs());
            }
        }
    }

    let mut stream = TokenStream::from(the_struct.into_token_stream());
    for impls in mixed_impls {
        stream.extend(impls);
    }
    Ok(stream)
}

struct Mixin {
    declaration: String,
    extensions: Vec<String>,
}

impl Mixin {
    fn from(input: &TokenStream) -> Self {
        Self {
            declaration: input.to_string(),
            extensions: Vec::new(),
        }
    }
}

static GLOBAL_DATA: Lazy<Mutex<HashMap<String, Mixin>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[proc_macro_attribute]
pub fn declare(_attribute: TokenStream, input: TokenStream) -> TokenStream {
    declare_impl(input).unwrap_or_else(Error::to_compile_error)
}

fn declare_impl(input: TokenStream) -> Result<TokenStream, Error> {
    // Keep it just to let the compiler check it
    let mut output: TokenStream = "#[allow(dead_code)]".parse()?;
    output.extend(input.clone().into_iter());

    // Consume the struct
    let mixin = Mixin::from(&input);
    let input: DeriveInput = syn::parse(input).unwrap();
    let name = input.ident.to_string();
    let mut data = GLOBAL_DATA.lock().map_err(|_| Error::GlobalUnavailable)?;
    data.insert(name, mixin);
    // And give the empty output back
    Ok(output)
}

#[proc_macro_attribute]
pub fn expand(_attribute: TokenStream, input: TokenStream) -> TokenStream {
    expand_impl(input).unwrap_or_else(Error::to_compile_error)
}

fn expand_impl(input: TokenStream) -> Result<TokenStream, Error> {
    // Keep it just to let the compiler check it
    let mut output: TokenStream = "#[allow(dead_code)]".parse()?;
    output.extend(input.clone().into_iter());

    let code = input.to_string();
    let ident = input.into_iter().skip(1).next();
    let name;
    match ident {
        Some(TokenTree::Ident(ident)) => {
            name = ident.to_string();
        }
        _ => {
            return Err(Error::InvalidExpansion);
        }
    }
    let mut data = GLOBAL_DATA.lock().map_err(|_| Error::GlobalUnavailable)?;
    let mixin = data.get_mut(&name).ok_or_else(|| Error::NoMixin(name))?;
    mixin.extensions.push(code);
    // Drops the original impl
    Ok(output)
}
