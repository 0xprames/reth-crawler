use chrono::{Days, NaiveDateTime, Utc};
use ethers::{
    abi::FixedBytes,
    providers::{Http, Middleware, Provider},
};
use futures::future;
use reth_crawler_db::{save_peer, AwsPeerDB};
use std::str::FromStr;
use std::sync::Arc;
use tracing::info;
const SYNCED_THRESHOLD: u64 = 100;
use ethers::types::H256;
use web3_dater::Web3Dater;

use chrono::{DateTime, FixedOffset};
use ipgeolocate::{Locator, Service};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let db = AwsPeerDB::new().await;
    let peers = db.all_nonexistent_isp_peers(None).await.unwrap();
    // println!("{:#?}", peers);
    let mut handles = vec![];
    let captured_db = Arc::from(db);
    for mut peer in peers {
        let db: Arc<AwsPeerDB> = captured_db.clone();
        handles.push(tokio::spawn(async move {
            // get peer location
            let service = Service::IpApi;
            let ip_addr = peer.address.to_string();

            let mut isp = String::default();

            match Locator::get(&ip_addr, service).await {
                Ok(loc) => {
                    isp = loc.isp;
                }
                Err(_) => {
                    // leave `country` and `city` empty if not able to get them
                }
            }
            peer.isp = isp;
            save_peer(peer, db).await;
        }));
    }
    future::join_all(handles).await;
}
