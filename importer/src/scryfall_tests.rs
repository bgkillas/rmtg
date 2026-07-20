const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
use crate::scryfall::download_card;
use bevy::tasks::block_on;
use reqwest::Client;
use std::str::FromStr;
use uuid::Uuid;
#[tokio::test(flavor = "multi_thread")]
async fn test() {
    let client = Client::builder().user_agent(USER_AGENT).build().unwrap();
    let card = block_on(download_card(
        &client,
        Uuid::from_str("0e6fc996-17ba-4090-bf82-0c2eba93a81e").unwrap(),
    ));
    println!("{card:#?}");
}
