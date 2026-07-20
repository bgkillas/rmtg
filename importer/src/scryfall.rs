use crate::card::{CardData, CardInfo, Layout, MaybeImage};
use crate::card::{Colors, Cost, SubCard, Types};
use crate::id::Id;
use json::{JsonValue, parse};
use reqwest::Client;
use uuid::Uuid;
impl SubCard {
    #[must_use]
    pub async fn get(client: &Client, uuid: Uuid) -> Option<Self> {
        let request = client
            .get(format!("https://api.scryfall.com/cards/{uuid}"))
            .send()
            .await
            .ok()?;
        let json_raw = request.text().await.ok()?;
        let json = parse(&json_raw).unwrap();
        Self::from_scryfall(json, uuid)
    }
    #[must_use]
    pub fn from_scryfall(json: JsonValue, uuid: Uuid) -> Option<Self> {
        let [name_raw, mana_cost_raw, type_line_raw, oracle_text_raw] =
            ["name", "mana_cost", "type_line", "oracle_text"].try_map(|s| json[s].as_str())?;
        let [colors, color_identity] = ["colors", "color_identity"]
            .map(|s| json[s].members().map(|c| c.as_str().unwrap_or_default()))
            .map(Colors::parse);
        let [power, toughness, loyalty] = ["power", "toughness", "loyalty"]
            .map(|s| json[s].as_str().and_then(|l| l.parse().ok()));
        let name = name_raw.to_owned();
        let oracle_text = oracle_text_raw.to_owned();
        let mana_cost = Cost::from(mana_cost_raw);
        let type_line = Types::from(type_line_raw);
        let face = CardInfo {
            name,
            mana_cost,
            type_line,
            oracle_text,
            colors,
            color_identity,
            power,
            toughness,
            loyalty,
            image: MaybeImage::default(),
        };
        let data = CardData {
            face,
            back: None,
            layout: Layout::default(),
        };
        Some(Self {
            id: Id::from(uuid),
            tokens: vec![],
            data,
            flipped: false,
        })
    }
}
