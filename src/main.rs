//#![allow(unused)]
use encryption::Key;
use mini_redis::Result;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::RsaPrivateKey;
use std::fs;
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub(crate) mod encryption;
pub(crate) mod frame;
mod log;

#[tokio::main]
async fn main() -> Result<()>
{
    let add_listen: SocketAddr = "192.168.1.129:8080".parse().unwrap();

    let listen = TcpListener::bind(add_listen).await?;

    let priv_key = fs::read_to_string("private.pem").unwrap();
    let priv_key = RsaPrivateKey::from_pkcs1_pem(&priv_key).unwrap();

    while let Ok((stream, _addr)) = listen.accept().await
    {
        log::log(
            log::Level::Info,
            &format!("listen completed with {:?}\n", stream),
        );
        let priv_key = priv_key.clone();
        tokio::spawn(async move {
            let mut con = frame::Connection::new(stream, Key::Private(priv_key));
            loop
            {
                let res = con.read_frame().await.unwrap();
                log::log(log::Level::Info, &format!("received frame -> {:?}", res));
            }
        });
    }
    Ok(())
}
