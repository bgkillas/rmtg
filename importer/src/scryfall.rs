use crate::card::SubCard;
use json::parse;
use reqwest::Client;
use uuid::Uuid;
pub async fn download_card(client: &Client, uuid: Uuid) -> Result<SubCard, reqwest::Error> {
    let request = client
        .get(format!("https://api.scryfall.com/cards/{uuid}"))
        .send()
        .await?;
    let json_raw = request.text().await?;
    let json = parse(&json_raw).unwrap();
    Ok(SubCard::default())
}
