#![feature(proc_macro_expand)]

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::token::parsing;
use syn::{token::Static, Ident, LitStr, Token};

struct Arguments {
    static_k: Static,
    varname: Ident,
    eq: Token![=],
    string_literal: LitStr,
    semicolon: Token![;],
}

impl Parse for Arguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let static_k = input.parse()?;
        parsing::keyword(input, "progmem")?;
        parsing::keyword(input, "string")?;
        let varname = input.parse()?;
        let eq: Token![=] = input.parse()?;
        let string_literal = input.parse()?;
        let semicolon: Token![;] = input.parse()?;

        Ok(Arguments {
            static_k,
            varname,
            eq,
            string_literal,
            semicolon,
        })
    }
}

#[proc_macro]
pub fn avr_progmem_str(t_stream: TokenStream) -> TokenStream {
    let macro_args = syn::parse_macro_input!(t_stream as Arguments);

    let Arguments {
        static_k,
        varname,
        eq,
        string_literal,
        semicolon,
    } = macro_args;

    let string_value = string_literal.value();
    let string_bytes = string_value.as_bytes();
    let tokens = string_bytes
        .iter()
        .map(|b| quote!(#b , ))
        .collect::<Vec<_>>();
    let count = string_bytes.len();

    quote!(
        #[cfg_attr(target_arch = "avr", link_section = ".progmem.data")]
        #static_k #varname : avr_progmem::ProgMem<[u8;#count]> #eq unsafe { avr_progmem::ProgMem::new([ #(#tokens)* ]) } #semicolon
    )
    .into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
