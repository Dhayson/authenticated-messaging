#[allow(unused_imports)]
use bytes::Bytes;
use mini_redis::{client, Result};
#[allow(unused_imports)]
use tokio::{select, time};

#[path = "../src/connection.rs"]
mod connection;
use connection::WrapClient;

#[tokio::main]
async fn main() -> Result<()>
{
    let hello = "hello";

    // Open a connection to the mini-redis address.
    let client = client::connect("192.168.1.129:8080").await.unwrap();

    let con = WrapClient::new(client).await;
    println!("connected");

    let con2 = con.clone();
    let con3 = con2.clone();
    tokio::spawn(async move {
        con3.get("Lol".to_string()).await.unwrap();
        con3.set("Lol".to_string(), "yasuone".into()).await.unwrap();
    })
    .await
    .ok();
    Ok(())
}
