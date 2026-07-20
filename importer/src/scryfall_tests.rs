const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
use crate::card::SubCard;
use bevy::tasks::block_on;
use reqwest::Client;
use std::str::FromStr;
use uuid::Uuid;
#[tokio::test(flavor = "multi_thread")]
async fn test() {
    let client = Client::builder().user_agent(USER_AGENT).build().unwrap();
    let (kiki, _) = block_on(SubCard::get(
        &client,
        Uuid::from_str("0e6fc996-17ba-4090-bf82-0c2eba93a81e").unwrap(),
    ))
    .unwrap();
    println!("{kiki:#?}");
    let (reaper, _) = block_on(SubCard::get(
        &client,
        Uuid::from_str("502740bf-0bff-4358-8996-1a27e5f0343f").unwrap(),
    ))
    .unwrap();
    println!("{reaper:#?}");
}
