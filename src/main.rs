//#![allow(unused)]
use encryption::{RsaKey, SignVerify};
use mini_redis::Result;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::RsaPrivateKey;
use std::fs;
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub(crate) mod encryption;
pub(crate) mod frame;
pub(crate) mod log;
pub(crate) mod message;

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
            let mut con = frame::Connection::new(
                stream,
                RsaKey::Private(priv_key),
                SignVerify::Verify(encryption::get_verify_key("key.verify").unwrap()),
            );
            loop
            {
                let res = con.read_frame().await.unwrap();
                frame_handler(res);
            }
        });
    }
    Ok(())
}

fn frame_handler(frame: frame::Frame)
{
    match frame
    {
        frame::Frame::String(s) =>
        {
            log::log(log::Level::Normal, &format!("received string: {}", s));
        }
        frame::Frame::Vec(vec) =>
        {
            for frame in vec
            {
                frame_handler(frame);
            }
        }
        frame::Frame::Message(m) => m.write_to_file().unwrap(),
    };
}
