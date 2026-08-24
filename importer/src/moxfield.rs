use crate::card::{Colors, SubCard};
use crate::scryfall::Quality;
use crate::warn_if;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::{DecodeSliceError, Engine as _};
use bevy::prelude::Event;
use jzon::{JsonValue, parse};
use reqwest::Client;
use std::fmt::{Debug, Display, Formatter};
use std::iter;
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
}
#[derive(Debug, Clone)]
pub enum MaybeBoards {
    None,
    Waiting,
    Full(Boards),
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
    pub fn unwrap(self) -> Boards {
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
        json: JsonValue,
        quality: Quality,
    ) -> Option<()> {
        async fn get_board(
            client: &Client,
            board: &JsonValue,
            quality: Quality,
        ) -> Option<Vec<SubCard>> {
            let iter = board["cards"].entries().flat_map(|(_, j)| {
                let id = warn_if(Uuid::parse_str(
                    j["card"]["scryfall_id"].as_str().unwrap_or_default(),
                ))
                .unwrap_or_default();
                let count = j["quantity"].as_usize().unwrap_or_default();
                iter::repeat_n(id, count)
            });
            let vec = SubCard::get_list(client, iter, quality)
                .await
                .into_iter()
                .map(Result::ok)
                .collect::<Option<Vec<_>>>();
            if vec.as_ref().is_some_and(Vec::is_empty) {
                None
            } else {
                vec
            }
        }
        let boards = &json["boards"];
        self.boards = MaybeBoards::Full(Boards {
            commanders: get_board(client, &boards["commanders"], quality).await,
            mainboard: get_board(client, &boards["mainboard"], quality).await,
        });
        Some(())
    }
}
