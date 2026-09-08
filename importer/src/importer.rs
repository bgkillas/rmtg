use crate::card::{SetCn, SubCard};
use crate::card_cache::Identifier;
use crate::scryfall::Quality;
use reqwest::Client;
impl SubCard {
    pub async fn parse_list(
        client: &Client,
        list: String,
        quality: Quality,
    ) -> Option<Vec<SubCard>> {
        let mut cards = Vec::with_capacity(128);
        for card in list.lines() {
            let (val, after_number) = card.split_once(|c: char| !c.is_numeric())?;
            let amount = val.parse().ok()?;
            let (_, set_start) = after_number.split_once('(')?;
            let (set, set_after) = set_start.split_once(')')?;
            let cn = set_after[1..]
                .split_once(' ')
                .map_or(&set_after[1..], |(a, _)| a);
            let set_cn = SetCn::new(set, cn);
            for _ in 0..amount {
                cards.push(Identifier::SetCn(set_cn.clone()));
            }
        }
        let vec = SubCard::get_list(client, cards, quality)
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
}
