use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
fn fix(ty: &str) -> String {
    ty.replace("-", "").replace("’", "").replace(" ", "")
}
fn ident(ty: &str) -> proc_macro2::TokenStream {
    let ident = Ident::new(&fix(ty), Span::call_site());
    quote! {#ident}
}
fn match_arms(ty: &str) -> proc_macro2::TokenStream {
    let ident = Ident::new(&fix(ty), Span::call_site());
    let literal = ty.to_string();
    quote! {
        #literal => Ok(Self::#ident)
    }
}
fn match_arms_to(ty: &str) -> proc_macro2::TokenStream {
    let ident = Ident::new(&fix(ty), Span::call_site());
    let literal = ty.to_string();
    quote! {
        Self::#ident => #literal.to_string()
    }
}
#[proc_macro]
pub fn generate_types(_item: TokenStream) -> TokenStream {
    let rules = include_str!("../rules.txt");
    let creature_types = rules.lines().find(|l| l.starts_with("205.3m")).unwrap();
    let split = creature_types.rsplit_once(": ").unwrap();
    let one_word_creature_types = split
        .1
        .replace("and ", "")
        .replace(".", "")
        .split(", ")
        .map(|a| a.to_string())
        .collect::<Vec<String>>();
    let one_idents = one_word_creature_types.iter().map(|s| ident(s));
    let one_match = one_word_creature_types.iter().map(|s| match_arms(s));
    let one_match_to = one_word_creature_types.iter().map(|s| match_arms_to(s));
    let split = creature_types.split_once(": ").unwrap();
    let two_word_creature_types = split
        .1
        .replace("and ", "")
        .split_once(".")
        .unwrap()
        .0
        .split(", ")
        .map(|a| a.to_string())
        .collect::<Vec<String>>();
    let two_idents = two_word_creature_types.iter().map(|s| ident(s));
    let two_match = two_word_creature_types.iter().map(|s| match_arms(s));
    let two_match_to = two_word_creature_types.iter().map(|s| match_arms_to(s));
    quote! {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode)]
        pub enum CreatureType {
            #( #one_idents, )*
            #( #two_idents, )*
        }
        impl FromStr for CreatureType {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #( #one_match, )*
                    #( #two_match, )*
                    _ => Err(()),
                }
            }
        }
        #[allow(clippy::to_string_trait_impl)]
        impl ToString for CreatureType {
            fn to_string(&self) -> String {
                match self {
                    #( #one_match_to, )*
                    #( #two_match_to, )*
                }
            }
        }
    }
    .into()
}
