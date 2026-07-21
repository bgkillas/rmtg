const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
use crate::card::SubCard;
use reqwest::Client;
use std::str::FromStr;
use uuid::Uuid;
#[tokio::test(flavor = "multi_thread")]
async fn test() {
    let client = Client::builder().user_agent(USER_AGENT).build().unwrap();
    let uuid = Uuid::from_str("0e6fc996-17ba-4090-bf82-0c2eba93a81e").unwrap();
    let (kiki, _) = SubCard::get(&client, uuid).await.unwrap();
    println!("{kiki:#?}");
    let uuid = Uuid::from_str("502740bf-0bff-4358-8996-1a27e5f0343f").unwrap();
    let (reaper, _) = SubCard::get(&client, uuid).await.unwrap();
    println!("{reaper:#?}");
    let uuid = Uuid::from_str("222a736e-d819-452d-aeda-eb848c4b2302").unwrap();
    let (tamiyo, _) = SubCard::get(&client, uuid).await.unwrap();
    println!("{tamiyo:#?}");
    let uuid = Uuid::from_str("a128e6d1-b90f-45a1-b587-f8c29bd0ec8c").unwrap();
    let (charred, _) = SubCard::get(&client, uuid).await.unwrap();
    println!("{charred:#?}");
    let uuid = Uuid::from_str("0b61d772-2d8b-4acf-9dd2-b2e8b03538c8").unwrap();
    let (erayo, _) = SubCard::get(&client, uuid).await.unwrap();
    println!("{erayo:#?}");
    let uuid = Uuid::from_str("627c392c-4d18-4eb2-a4e8-c668f61f5487").unwrap();
    let (aclazotz, _) = SubCard::get(&client, uuid).await.unwrap();
    println!("{aclazotz:#?}");
    let uuid = Uuid::from_str("e0dbbdcf-84e1-494f-8b8c-0a094f603fa9").unwrap();
    let (bruce, _) = SubCard::get(&client, uuid).await.unwrap();
    println!("{bruce:#?}");
    let uuid = Uuid::from_str("04506bad-3856-4184-8dda-941ded60f41a").unwrap();
    let (gisela, _) = SubCard::get(&client, uuid).await.unwrap();
    println!("{gisela:#?}");
}
