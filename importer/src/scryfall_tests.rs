use crate::scryfall::download_card;
use bevy::tasks::TaskPool;
use std::str::FromStr;
use uuid::Uuid;
#[test]
fn test() {
    let pool = TaskPool::new();
    let card = &pool.scope(|s| {
        s.spawn(download_card(
            Uuid::from_str("0e6fc996-17ba-4090-bf82-0c2eba93a81e").unwrap(),
        ));
    })[0];
    println!("{card:#?}");
}
