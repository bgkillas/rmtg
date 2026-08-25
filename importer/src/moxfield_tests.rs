use crate::moxfield::MoxfieldDeck;
use crate::scryfall::{CACHE, IMAGES_IN_PROGRESS, Quality, SLEEP_TIME};
use crate::scryfall_tests::{USER_AGENT, clear_images};
use reqwest::Client;
use std::time::Instant;
use tokio::time::sleep;
#[tokio::test(flavor = "multi_thread")]
async fn get_decks() {
    let client = Client::builder().user_agent(USER_AGENT).build().unwrap();
    let tmr = Instant::now();
    let decks = MoxfieldDeck::get_decks(&client, "bgkillas").await.unwrap();
    let time = tmr.elapsed().as_millis();
    println!("{} {time}", decks.len());
    for mut deck in decks {
        if &*deck.name == "gambling 3" {
            let tmr = Instant::now();
            deck.get_deck(&client, Quality::Normal).await.unwrap();
            let time = tmr.elapsed().as_millis();
            let mut in_progress_images = IMAGES_IN_PROGRESS.lock().await;
            let tmr = Instant::now();
            for card in deck.boards.clone().unwrap().commanders.unwrap() {
                card.spawn_image_getters(&client, &mut in_progress_images, Quality::Normal, |f| {
                    tokio::spawn(f);
                });
            }
            for card in deck.boards.clone().unwrap().mainboard.unwrap() {
                card.spawn_image_getters(&client, &mut in_progress_images, Quality::Normal, |f| {
                    tokio::spawn(f);
                });
            }
            println!("{}", tmr.elapsed().as_millis());
            drop(in_progress_images);
            let cache = CACHE.lock().await;
            println!(
                "{} {} {time} {} {:?} {:?}",
                deck.boards.clone().unwrap().mainboard.unwrap().len(),
                deck.boards.unwrap().commanders.unwrap().len(),
                IMAGES_IN_PROGRESS.lock().await.len(),
                cache.in_progress,
                cache.in_progress_set_cn,
            );
            drop(cache);
            let tmr = Instant::now();
            clear_images().await;
            while IMAGES_IN_PROGRESS.lock().await.len() > 0 {
                sleep(SLEEP_TIME).await;
                clear_images().await;
            }
            println!("{}", tmr.elapsed().as_millis());
            break;
        }
    }
}
