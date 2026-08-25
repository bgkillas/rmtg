pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
use crate::card::SubCard;
use crate::scryfall::{CACHE, IMAGES_IN_PROGRESS, IMAGES_TO_PROCESS, Quality, SLEEP_TIME};
use fdlimit::raise_fd_limit;
use reqwest::Client;
use std::time::Instant;
use tokio::time::sleep;
#[cfg(target_family = "wasm")]
use tokio_with_wasm as tokio;
use uuid::uuid;
pub async fn clear_images() {
    let mut progress = IMAGES_IN_PROGRESS.lock().await;
    for (uuid, _) in IMAGES_TO_PROCESS.lock().await.drain() {
        assert!(progress.remove(&uuid));
    }
}
#[tokio::test(flavor = "multi_thread")]
async fn test_list() {
    raise_fd_limit().unwrap();
    let client = Client::builder().user_agent(USER_AGENT).build().unwrap();
    let kiki_uuid = uuid!("0e6fc996-17ba-4090-bf82-0c2eba93a81e");
    let reaper_uuid = uuid!("502740bf-0bff-4358-8996-1a27e5f0343f");
    let tamiyo_uuid = uuid!("222a736e-d819-452d-aeda-eb848c4b2302");
    let charred_uuid = uuid!("a128e6d1-b90f-45a1-b587-f8c29bd0ec8c");
    let erayo_uuid = uuid!("0b61d772-2d8b-4acf-9dd2-b2e8b03538c8");
    let aclazotz_uuid = uuid!("627c392c-4d18-4eb2-a4e8-c668f61f5487");
    let bruce_uuid = uuid!("e0dbbdcf-84e1-494f-8b8c-0a094f603fa9");
    let gisela_uuid = uuid!("04506bad-3856-4184-8dda-941ded60f41a");
    let tmr = Instant::now();
    let uuids = [[
        kiki_uuid,
        reaper_uuid,
        tamiyo_uuid,
        charred_uuid,
        erayo_uuid,
        aclazotz_uuid,
        bruce_uuid,
        gisela_uuid,
    ]; 128];
    let list = SubCard::get_list(
        &client,
        uuids.as_flattened().iter().copied(),
        Quality::Normal,
    )
    .await;
    let time = tmr.elapsed().as_millis();
    let mut in_progress_images = IMAGES_IN_PROGRESS.lock().await;
    let tmr = Instant::now();
    for card in list.iter().filter_map(|c| c.as_ref().ok()) {
        card.spawn_image_getters(&client, &mut in_progress_images, Quality::Normal, |f| {
            tokio::spawn(f);
        });
    }
    println!("{}", tmr.elapsed().as_millis());
    drop(in_progress_images);
    let mut i = 0;
    for res in &list {
        if let Err(uuid) = res {
            i += 1;
            println!("{uuid}");
        }
    }
    let cache = CACHE.lock().await;
    println!(
        "{} {i} {} {} {:?} {:?}",
        list.len(),
        IMAGES_IN_PROGRESS.lock().await.len(),
        time,
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
    println!("{}", tmr.elapsed().as_millis(),);
}
#[tokio::test(flavor = "multi_thread")]
async fn test_prints() {
    raise_fd_limit().unwrap();
    let client = Client::builder().user_agent(USER_AGENT).build().unwrap();
    let forest_uuid = uuid!("b34bb2dc-c1af-4d77-b0b3-a0fb342a5fc6");
    let tmr = Instant::now();
    let vec = SubCard::get_prints(&client, forest_uuid, Quality::Normal)
        .await
        .unwrap();
    let time = tmr.elapsed().as_millis();
    let mut in_progress_images = IMAGES_IN_PROGRESS.lock().await;
    let tmr = Instant::now();
    for card in vec.iter().filter_map(|c| c.as_ref().ok()) {
        card.spawn_image_getters(&client, &mut in_progress_images, Quality::Normal, |f| {
            tokio::spawn(f);
        });
    }
    println!("{}", tmr.elapsed().as_millis());
    drop(in_progress_images);
    let mut i = 0;
    for res in &vec {
        if let Err(uuid) = res {
            i += 1;
            println!("{uuid}");
        }
    }
    let cache = CACHE.lock().await;
    println!(
        "{} {i} {} {} {:?} {:?}",
        vec.len(),
        IMAGES_IN_PROGRESS.lock().await.len(),
        time,
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
    println!("{}", tmr.elapsed().as_millis(),);
}
