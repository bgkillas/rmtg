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
    let literal = ty.to_ascii_lowercase();
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
fn split(s: &str) -> Vec<String> {
    s.rsplit_once(" are ")
        .unwrap()
        .1
        .replace(" and ", " ")
        .replace(".", "")
        .split(", ")
        .map(|a| {
            if let Some((a, _)) = a.split_once(" (") {
                a.to_string()
            } else {
                a.to_string()
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
        .replace(".", "")
        .split(", ")
        .map(|a| a.to_string())
        .collect::<Vec<String>>();
    let split = creature_types.split_once(": ").unwrap();
    let two_word_creature_types = split
        .1
        .replace(" and ", " ")
        .split_once(".")
        .unwrap()
        .0
        .split(", ")
        .map(|a| a.to_string())
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
        vec!["Undercity".to_string()],
        vec!["Siege".to_string()],
    ];
    let idents = types.iter().flatten().map(|s| ident(s));
    let matchs = types.iter().flatten().map(|s| match_arms(s));
    let matchs_to = types.iter().flatten().map(|s| match_arms_to(s));
    let super_types = ["Basic", "Legendary", "Ongoing", "Snow", "World"];
    let sidents = super_types.iter().map(|s| ident(s));
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
    let nidents = types.iter().map(|s| ident(s));
    let nmatchs = types.iter().map(|s| match_arms(s));
    let nmatchs_to = types.iter().map(|s| match_arms_to(s));
    quote! {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode)]
        pub enum SubType {
            #( #idents, )*
        }
        impl FromStr for SubType {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_ascii_lowercase().as_str() {
                    #( #matchs, )*
                    _ => Err(()),
                }
            }
        }
        impl fmt::Display for SubType {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                write!(f, "{}", match self {
                    #( #matchs_to, )*
                })
            }
        }
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode)]
        pub enum Type {
            #( #nidents, )*
        }
        impl FromStr for Type {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_ascii_lowercase().as_str() {
                    #( #nmatchs, )*
                    _ => Err(()),
                }
            }
        }
        impl fmt::Display for Type {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                write!(f, "{}", match self {
                    #( #nmatchs_to, )*
                })
            }
        }
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode)]
        pub enum SuperType {
            #( #sidents, )*
        }
        impl FromStr for SuperType {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_ascii_lowercase().as_str() {
                    #( #smatchs, )*
                    _ => Err(()),
                }
            }
        }
        impl fmt::Display for SuperType {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                write!(f, "{}", match self {
                    #( #smatchs_to, )*
                })
            }
        }
    }
    .into()
}
