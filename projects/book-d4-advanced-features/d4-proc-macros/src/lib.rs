//! Small procedural macros implemented directly with the stable proc_macro API.

use proc_macro::{TokenStream, TokenTree};
use std::str::FromStr;

fn compile_error(message: &str) -> TokenStream {
    TokenStream::from_str(&format!("compile_error!({message:?});"))
        .expect("compile_error expansion must parse")
}

fn named_type(input: &TokenStream) -> Result<String, &'static str> {
    let mut tokens = input.clone().into_iter();
    while let Some(token) = tokens.next() {
        if let TokenTree::Ident(keyword) = token {
            let kind = keyword.to_string();
            if kind == "struct" || kind == "enum" {
                let Some(TokenTree::Ident(name)) = tokens.next() else {
                    return Err("D4 macros require a named struct or enum");
                };
                if matches!(tokens.next(), Some(TokenTree::Punct(punct)) if punct.as_char() == '<') {
                    return Err("D4 teaching macros currently support non-generic types only");
                }
                return Ok(name.to_string());
            }
        }
    }
    Err("D4 macros require a named struct or enum")
}

/// Derives Describe for a named, non-generic struct or enum.
#[proc_macro_derive(Describe)]
pub fn derive_describe(input: TokenStream) -> TokenStream {
    let name = match named_type(&input) {
        Ok(name) => name,
        Err(message) => return compile_error(message),
    };
    TokenStream::from_str(&format!(
        "impl ::rustapplab_book_d4_lab::Describe for {name} {{ \
             fn describe(&self) -> &'static str {{ \"{name}\" }} \
         }}"
    ))
    .unwrap_or_else(|_| compile_error("D4 derive macro could not parse its expansion"))
}

/// Adds a documentation marker and an associated AUDIT_TAG constant to a named type.
///
/// Use an identifier argument, for example audit_tag(telemetry).
#[proc_macro_attribute]
pub fn audit_tag(attribute: TokenStream, item: TokenStream) -> TokenStream {
    let tag = attribute.to_string();
    if tag.is_empty() {
        return compile_error("audit_tag requires one non-empty identifier");
    }
    let name = match named_type(&item) {
        Ok(name) => name,
        Err(message) => return compile_error(message),
    };
    let doc = format!("D4 procedural attribute applied: {tag}");
    let expansion = format!(
        "#[doc = {doc:?}] {item} \
         impl {name} {{ pub const AUDIT_TAG: &'static str = {tag:?}; }}"
    );
    TokenStream::from_str(&expansion)
        .unwrap_or_else(|_| compile_error("D4 attribute macro could not parse its expansion"))
}

/// Expands to the number of top-level tokens in its input.
#[proc_macro]
pub fn token_count(input: TokenStream) -> TokenStream {
    let count = input.into_iter().count();
    TokenStream::from_str(&format!("{count}usize"))
        .expect("integer literal expansion must parse")
}
