use proc_macro::TokenStream;
use quote::quote;
use std::fs::File;
use std::io::Read;
use syn::parse::{Parse, ParseStream};
use syn::token::parsing;
use syn::{parenthesized, token::Static, Ident, LitStr, Token};

struct Arguments {
    static_k: Static,
    varname: Ident,
    eq: Token![=],
    string_bytes: Vec<u8>,
    semicolon: Token![;],
}

impl Parse for Arguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let static_k = input.parse()?;
        parsing::keyword(input, "progmem")?;
        parsing::keyword(input, "string")?;
        let varname = input.parse()?;
        let eq: Token![=] = input.parse()?;
        let string_bytes: Vec<u8> = if input.lookahead1().peek(LitStr) {
            let literal = input.parse::<LitStr>()?;
            literal.value().as_bytes().to_vec()
        } else {
            custom_include_str(input)?
        };

        let semicolon: Token![;] = input.parse()?;

        Ok(Arguments {
            static_k,
            varname,
            eq,
            string_bytes,
            semicolon,
        })
    }
}

fn custom_include_str(input: ParseStream) -> syn::Result<Vec<u8>> {
    let content;
    parsing::keyword(input, "include_str")?;
    let _bang: Token![!] = input.parse()?;
    parenthesized!(content in input);
    let fname = content.parse::<LitStr>()?;
    /*for (k, v) in std::env::vars() {
        if v.contains("string1") {
            println!("{} = {}", k, v);
        }
    }*/
    let pwd = std::env::var("PWD").unwrap_or(String::from("."));
    let fname2 = format!("{}/src/{}", pwd, fname.value()); // this is probably very clumsy, and probably does not match include_str! for subdirs
    slurp(fname2).map_err(|_i_should_probably_do_something_with_this| {
        syn::Error::new_spanned(fname, "failed to read file")
    })
}

fn slurp<P: AsRef<std::path::Path>>(fname: P) -> Result<Vec<u8>, std::io::Error> {
    println!("trying to read file {}", fname.as_ref().to_str().unwrap());
    let mut f = File::open(fname)?;
    let mut rval: Vec<u8> = vec![];
    let mut buf = [0u8; 4 << 10];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        rval.extend_from_slice(&buf[..n]);
    }
    Ok(rval)
}

#[proc_macro]
pub fn avr_progmem_str(t_stream: TokenStream) -> TokenStream {
    let macro_args = syn::parse_macro_input!(t_stream as Arguments);

    let Arguments {
        static_k,
        varname,
        eq,
        string_bytes,
        semicolon,
    } = macro_args;

    let tokens = string_bytes
        .iter()
        .map(|b| quote!(#b , ))
        .collect::<Vec<_>>();
    let count = string_bytes.len();

    quote!(
        #[cfg_attr(target_arch = "avr", link_section = ".progmem.data")]
        #static_k #varname : avr_progmem::string::PmString<#count> #eq unsafe { avr_progmem::string::PmString::from_array([ #(#tokens)* ]) } #semicolon
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
