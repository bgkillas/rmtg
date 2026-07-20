use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
fn fix(ty: &str) -> String {
    ty.replace(['-', '’', ' '], "")
}
fn ident(ty: &str) -> proc_macro2::TokenStream {
    let ident = Ident::new(&fix(ty), Span::call_site());
    quote! {#ident}
}
fn match_arms(ty: &str) -> proc_macro2::TokenStream {
    let ident = Ident::new(&fix(ty), Span::call_site());
    let literal = ty.to_ascii_lowercase();
    quote! {
        #literal => Ok(Self::#ident)
    }
}
fn match_arms_to(ty: &str) -> proc_macro2::TokenStream {
    let ident = Ident::new(&fix(ty), Span::call_site());
    let literal = ty.to_owned();
    quote! {
        Self::#ident => #literal.to_owned()
    }
}
fn split(s: &str) -> Vec<String> {
    s.rsplit_once(" are ")
        .unwrap()
        .1
        .replace(" and ", " ")
        .replace('.', "")
        .split(", ")
        .map(|a| {
            if let Some((v, _)) = a.split_once(" (") {
                v.to_owned()
            } else {
                a.to_owned()
            }
        })
        .collect::<Vec<String>>()
}
#[proc_macro]
pub fn generate_types(_item: TokenStream) -> TokenStream {
    let rules = include_str!("../rules.txt");
    let plane_types = split(rules.lines().find(|l| l.starts_with("205.3n")).unwrap());
    let spell_types = split(rules.lines().find(|l| l.starts_with("205.3k")).unwrap());
    let plane_walkers_types = split(rules.lines().find(|l| l.starts_with("205.3j")).unwrap());
    let land_types = split(
        rules
            .lines()
            .find(|l| l.starts_with("205.3i"))
            .unwrap()
            .rsplit_once(" Of that list")
            .unwrap()
            .0,
    );
    let enchantment_types = split(rules.lines().find(|l| l.starts_with("205.3h")).unwrap());
    let artifact_types = split(rules.lines().find(|l| l.starts_with("205.3g")).unwrap());
    let creature_types = rules.lines().find(|l| l.starts_with("205.3m")).unwrap();
    let split = creature_types.rsplit_once(": ").unwrap();
    let one_word_creature_types = split
        .1
        .replace(" and ", " ")
        .replace('.', "")
        .split(", ")
        .map(ToOwned::to_owned)
        .collect::<Vec<String>>();
    let split = creature_types.split_once(": ").unwrap();
    let two_word_creature_types = split
        .1
        .replace(" and ", " ")
        .split_once('.')
        .unwrap()
        .0
        .split(", ")
        .map(ToOwned::to_owned)
        .collect::<Vec<String>>();
    let types = [
        plane_types
            .into_iter()
            .filter(|a| a != "Spacecraft")
            .collect(),
        spell_types,
        plane_walkers_types,
        land_types,
        enchantment_types,
        artifact_types,
        one_word_creature_types,
        two_word_creature_types,
        vec!["Undercity".to_owned()],
        vec!["Siege".to_owned()],
    ];
    let idents: Vec<_> = types.iter().flatten().map(|s| ident(s)).collect();
    let matchs = types.iter().flatten().map(|s| match_arms(s));
    let matchs_to = types.iter().flatten().map(|s| match_arms_to(s));
    let super_types = ["Basic", "Legendary", "Ongoing", "Snow", "World"];
    let sidents: Vec<_> = super_types.iter().map(|s| ident(s)).collect();
    let smatchs = super_types.iter().map(|s| match_arms(s));
    let smatchs_to = super_types.iter().map(|s| match_arms_to(s));
    let types = [
        "Land",
        "Phenomenon",
        "Plane",
        "Scheme",
        "Vanguard",
        "Creature",
        "Artifact",
        "Enchantment",
        "PlanesWalker",
        "Conspiracy",
        "Battle",
        "Dungeon",
        "Instant",
        "Sorcery",
        "Kindred",
    ];
    let nidents: Vec<_> = types.iter().map(|s| ident(s)).collect();
    let nmatchs = types.iter().map(|s| match_arms(s));
    let nmatchs_to = types.iter().map(|s| match_arms_to(s));
    quote! {
        #[derive(Debug, Encode, Decode, enumset::EnumSetType)]
        pub enum SubType {
            #( #idents, )*
        }
        pub const SUBTYPES: &[SubType] = &[#( SubType::#idents, )*];
        impl TryFrom<&str> for SubType {
            type Error = ();
            fn try_from(s: &str) -> Result<Self, Self::Error> {
                match s.to_ascii_lowercase().as_str() {
                    #( #matchs, )*
                    _ => Err(()),
                }
            }
        }
        impl std::fmt::Display for SubType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                write!(f, "{}", match self {
                    #( #matchs_to, )*
                })
            }
        }
        #[derive(Debug, Encode, Decode, enumset::EnumSetType)]
        pub enum MainType {
            #( #nidents, )*
        }
        pub const TYPES: &[MainType] = &[#( MainType::#nidents, )*];
        impl TryFrom<&str> for MainType {
            type Error = ();
            fn try_from(s: &str) -> Result<Self, Self::Error> {
                match s.to_ascii_lowercase().as_str() {
                    #( #nmatchs, )*
                    _ => Err(()),
                }
            }
        }
        impl std::fmt::Display for MainType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                write!(f, "{}", match self {
                    #( #nmatchs_to, )*
                })
            }
        }
        #[derive(Debug, Encode, Decode, enumset::EnumSetType)]
        pub enum SuperType {
            #( #sidents, )*
        }
        pub const SUPERTYPES: &[SuperType] = &[#( SuperType::#sidents, )*];
        impl TryFrom<&str> for SuperType {
            type Error = ();
            fn try_from(s: &str) -> Result<Self, Self::Error> {
                match s.to_ascii_lowercase().as_str() {
                    #( #smatchs, )*
                    _ => Err(()),
                }
            }
        }
        impl std::fmt::Display for SuperType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                write!(f, "{}", match self {
                    #( #smatchs_to, )*
                })
            }
        }
    }
    .into()
}
