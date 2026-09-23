use crate::card::{Colors, SubCard};
use crate::card_cache::Identifier;
use crate::scryfall::Quality;
use crate::warn_if;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::{DecodeSliceError, Engine as _};
use bevy::prelude::Event;
use futures::future::join_all;
use jzon::{JsonValue, parse};
use reqwest::Client;
use std::fmt::{Debug, Display, Formatter};
use std::iter;
use std::ops::Deref;
use std::str::FromStr;
use uuid::Uuid;
const URL: &str = "api2.moxfield.com";
#[derive(Clone, Copy)]
pub struct DeckId {
    pub bytes: [u8; 16],
}
impl FromStr for DeckId {
    type Err = DecodeSliceError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bytes = [0; 16];
        BASE64_URL_SAFE_NO_PAD.decode_slice(s, &mut bytes)?;
        Ok(Self { bytes })
    }
}
impl Display for DeckId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut buffer = [0; 24];
        BASE64_URL_SAFE_NO_PAD
            .encode_slice(self.bytes, &mut buffer)
            .unwrap();
        let str = str::from_utf8(&buffer).unwrap();
        write!(f, "{str}")
    }
}
impl Debug for DeckId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}
#[derive(Debug, Clone, Event)]
pub struct Boards {
    pub commanders: Option<Vec<SubCard>>,
    pub mainboard: Option<Vec<SubCard>>,
    pub sideboard: Option<Vec<SubCard>>,
    pub companions: Option<Vec<SubCard>>,
    pub signature_spells: Option<Vec<SubCard>>,
    pub attractions: Option<Vec<SubCard>>,
    pub stickers: Option<Vec<SubCard>>,
    pub contraptions: Option<Vec<SubCard>>,
    pub planes: Option<Vec<SubCard>>,
    pub schemes: Option<Vec<SubCard>>,
}
impl Boards {
    pub fn iter(&self) -> impl Iterator<Item = &[SubCard]> {
        [
            &self.commanders,
            &self.mainboard,
            &self.sideboard,
            &self.companions,
            &self.signature_spells,
            &self.attractions,
            &self.stickers,
            &self.contraptions,
            &self.planes,
            &self.schemes,
        ]
        .into_iter()
        .filter_map(Option::as_ref)
        .map(Deref::deref)
    }
}
#[derive(Debug, Clone)]
pub enum MaybeBoards {
    None,
    Waiting,
    Full(Box<Boards>),
}
#[derive(Debug, Clone)]
pub struct MoxfieldDeck {
    pub id: DeckId,
    pub colors: Colors,
    pub name: Box<str>,
    pub boards: MaybeBoards,
}
impl MaybeBoards {
    pub fn is_some(&self) -> bool {
        !matches!(self, Self::None | Self::Waiting)
    }
    pub fn is_waiting(&self) -> bool {
        !matches!(self, Self::Waiting)
    }
    pub fn unwrap(self) -> Box<Boards> {
        match self {
            MaybeBoards::Full(board) => board,
            MaybeBoards::None | MaybeBoards::Waiting => {
                panic!()
            }
        }
    }
}
impl MoxfieldDeck {
    pub async fn get_deck(&mut self, client: &Client, quality: Quality) -> Result<(), DeckId> {
        async fn get_deck(client: &Client, id: DeckId) -> Option<JsonValue> {
            let request = warn_if(
                client
                    .get(format!("https://{URL}/v3/decks/all/{id}"))
                    .send()
                    .await,
            )?;
            let json_raw = warn_if(request.text().await)?;
            warn_if(parse(&json_raw))
        }
        let deck = get_deck(client, self.id).await.ok_or(self.id)?;
        self.parse_json(client, deck, quality)
            .await
            .ok_or(self.id)?;
        Ok(())
    }
    pub async fn get_decks(client: &Client, user: &str) -> Option<Vec<Self>> {
        async fn get_page(client: &Client, user: &str, page: usize) -> Option<JsonValue> {
            let request = warn_if(
                client
                    .get(format!("https://{URL}/v2/decks/search"))
                    .query(&(
                        ("authorUserNames", user),
                        ("sortType", "updated"),
                        ("sortDirection", "descending"),
                        ("pageSize", 100),
                        ("pageNumber", page),
                        ("showIllegal", true),
                    ))
                    .send()
                    .await,
            )?;
            let json_raw = warn_if(request.text().await)?;
            warn_if(parse(&json_raw))
        }
        let mut vec = Vec::new();
        for page in 1.. {
            let json = get_page(client, user, page).await?;
            if vec.capacity() == 0 {
                vec.reserve_exact(json["totalResults"].as_usize()?);
            }
            for deck in json["data"].as_array()?.iter().map(Self::from_json) {
                vec.push(deck?);
            }
            if json["totalPages"].as_usize()? == page {
                break;
            }
        }
        Some(vec)
    }
    pub fn from_json(json: &JsonValue) -> Option<Self> {
        Some(Self {
            id: warn_if(json["publicId"].as_str()?.parse())?,
            colors: Colors::parse(
                json["colors"]
                    .as_array()?
                    .iter()
                    .map(|j| j.as_str().unwrap_or_default()),
            ),
            name: json["name"].as_str()?.into(),
            boards: MaybeBoards::None,
        })
    }
    pub async fn parse_json(
        &mut self,
        client: &Client,
        mut json: JsonValue,
        quality: Quality,
    ) -> Option<()> {
        let boards = Boards::from_json(client, json.remove("boards"), quality).await;
        self.boards = MaybeBoards::Full(Box::new(boards));
        Some(())
    }
}
impl Boards {
    pub async fn from_json(client: &Client, mut json: JsonValue, quality: Quality) -> Self {
        async fn get_board(
            client: &Client,
            mut board: JsonValue,
            quality: Quality,
        ) -> Option<Vec<SubCard>> {
            let cards = board.remove("cards");
            let JsonValue::Object(object) = cards else {
                return None;
            };
            let iter = object
                .into_iter()
                .flat_map(|(_, j)| {
                    let id = warn_if(Uuid::parse_str(
                        j["card"]["scryfall_id"].as_str().unwrap_or_default(),
                    ))
                    .unwrap_or_default();
                    let count = j["quantity"].as_usize().unwrap_or_default();
                    iter::repeat_n(Identifier::Uuid(id), count)
                })
                .collect();
            let vec = SubCard::get_list(client, iter, quality)
                .await?
                .into_iter()
                .map(Result::ok)
                .collect::<Option<Vec<_>>>();
            if vec.as_ref().is_some_and(Vec::is_empty) {
                None
            } else {
                vec
            }
        }
        let [
            commanders,
            mainboard,
            sideboard,
            companions,
            signature_spells,
            attractions,
            stickers,
            contraptions,
            planes,
            schemes,
        ] = *join_all(
            [
                "commanders",
                "mainboard",
                "sideboard",
                "companions",
                "signatureSpells",
                "attractions",
                "stickers",
                "contraptions",
                "planes",
                "schemes",
            ]
            .map(|s| json.remove(s))
            .map(|j| get_board(client, j, quality)),
        )
        .await
        .into_array()
        .unwrap();
        Self {
            commanders,
            mainboard,
            sideboard,
            companions,
            signature_spells,
            attractions,
            stickers,
            contraptions,
            planes,
            schemes,
        }
    }
}
