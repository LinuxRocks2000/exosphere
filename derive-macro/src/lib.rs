// HERE BE DRAGONS
// This works. Sorta. Sometimes bugs crop up that take literal divine inspiration to solve.
// Don't anger it and it won't exact a toll in blood.
// Seriously tho, nobody has a clue how this works and nobody really *wants* to know. It's a black box. Admittedly an old, banged-up, moldy, and water-damaged black box.
// With some holes in it. None of that shiny carbonan box hootenanny. This box has *seen* places.
// what was I talking about again?

// [about an hour later] at this point I'd just like to pretend this file doesn't exist and get on with my game development.
// buuuuuut I have error handling to do *sigh*

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(ProtocolRoot)]
pub fn protocol_derive(input : TokenStream) -> TokenStream {
    let ast : syn::DeriveInput = syn::parse(input).unwrap();
    let name = ast.ident.to_string();

    let mut sizes_function : String = "fn size(&self) -> usize {match self {".to_string();
    let mut encode_into_function : String = "fn encode_into(&self, buffer : &mut [u8]) {match self {".to_string();
    let mut decode_from_function : String = "fn decode_from(data : &[u8]) -> Result<Self, DecodeError> {match data[0] {".to_string();
    let mut report_tag_function : String = "fn report_tag(&self) -> u16 {match self {".to_string();

    if let syn::Data::Enum(data) = ast.data {
        let mut variant_tag : u8 = 0;
        for variant in data.variants.iter() {
            let mut fields_string = "".to_string();
            let mut size_string = "".to_string();
            let mut encode_string = format!("let mut position = 1;buffer[0] = {};", variant_tag);
            let mut decode_string_create = "let mut position = 1;".to_string(); // set up the components
            let mut decode_string_assign = format!("Ok({}::{}", &name, variant.ident.to_string()); // generate and return the enum
            match &variant.fields {
                syn::Fields::Unit => {
                    size_string += "1"
                },
                syn::Fields::Unnamed (fields) => {
                    decode_string_assign += "(";
                    let mut f_count = 0;
                    for field in fields.unnamed.iter() {
                        if f_count == 0 {
                            fields_string += "(";
                        }
                        size_string += &format!("field_{f_count}.size() + ");
                        fields_string += &format!("field_{f_count}, ");
                        encode_string += &format!("field_{f_count}.encode_into(&mut buffer[position..]);position += field_{f_count}.size();");
                        let ty = &field.ty;
                        decode_string_create += &format!("let field_{} = {}::decode_from(&data[position..])?;position += field_{}.size();", f_count, quote!{#ty}.to_string(), f_count);
                        decode_string_assign += &format!("field_{f_count}, ");
                        f_count += 1;
                    }
                    if fields_string.len() > 0 {
                        fields_string += ")";
                    }
                    size_string += "1";
                    decode_string_assign += ")";
                },
                _ => {
                    panic!("whoops! (todo: better errors)");
                }
            }
            decode_string_assign += ")";
            sizes_function += &(format!("{}::{}{} => {{{}}}", &name, variant.ident.to_string(), fields_string, size_string));
            encode_into_function += &format!("{}::{}{} => {{{}}}", &name, variant.ident.to_string(), fields_string, encode_string);
            report_tag_function += &format!("{}::{}{} => {{{}}}", &name, variant.ident.to_string(), fields_string, variant_tag);
            decode_from_function += &format!("{} => {{{}{}}}", variant_tag, decode_string_create, decode_string_assign);

            variant_tag += 1;
        }
    }
    else {
        panic!("error: attempt to derive Protocol on a non-enum!3
4
5 (todo: better errors)");
    }

    sizes_function += "}}";
    encode_into_function += "}}";
    report_tag_function += "}}";
    decode_from_function += "_ => {return Err(DecodeError {});}}}";

    let out = format!("impl Protocol for {name} {{{sizes_function} \n\n\n {encode_into_function} \n\n\n {decode_from_function} }} impl ProtocolRoot for {name} {{ \n\n\n {report_tag_function} }}");
    out.parse().unwrap()
}