use mini_redis::{client, Result};
use tokio::{select, time};

#[tokio::main]
async fn main() -> Result<()>
{
    let hello = "hello";
    // Open a connection to the mini-redis address.
    let mut client = client::connect("192.168.1.129:6379").await.unwrap();
    println!("connected");
    //let mut client = client::connect("0.0.0.0:6379").await?;
    client.set(hello, "sima".into()).await?;
    loop
    {
        select! {
            // Print every 3 secs if nothing happens
            _ = time::sleep(time::Duration::from_secs_f32(3f32)) =>{
                println!("got value from the server; result={:?}", client.get(hello).await);
            }
        };
    }
}
