const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
use crate::card::SubCard;
use reqwest::Client;
use uuid::uuid;
#[tokio::test(flavor = "multi_thread")]
async fn test() {
    let client = Client::builder().user_agent(USER_AGENT).build().unwrap();
    let kiki_uuid = uuid!("0e6fc996-17ba-4090-bf82-0c2eba93a81e");
    let reaper_uuid = uuid!("502740bf-0bff-4358-8996-1a27e5f0343f");
    let tamiyo_uuid = uuid!("222a736e-d819-452d-aeda-eb848c4b2302");
    let charred_uuid = uuid!("a128e6d1-b90f-45a1-b587-f8c29bd0ec8c");
    let erayo_uuid = uuid!("0b61d772-2d8b-4acf-9dd2-b2e8b03538c8");
    let aclazotz_uuid = uuid!("627c392c-4d18-4eb2-a4e8-c668f61f5487");
    let bruce_uuid = uuid!("e0dbbdcf-84e1-494f-8b8c-0a094f603fa9");
    let gisela_uuid = uuid!("04506bad-3856-4184-8dda-941ded60f41a");
    let tmr = std::time::Instant::now();
    let uuids = [[
        kiki_uuid,
        reaper_uuid,
        tamiyo_uuid,
        charred_uuid,
        erayo_uuid,
        aclazotz_uuid,
        bruce_uuid,
        gisela_uuid,
    ]; 64];
    let list = SubCard::get_list(client, uuids.as_flattened()).await;
    println!("{} {}", uuids.as_flattened().len(), list.len());
    for (card, _) in list {
        //println!("{card:#?}");
    }
    println!("{}", tmr.elapsed().as_nanos());
}
