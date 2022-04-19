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

    let mut key_ver_list = Vec::new();

    for path in fs::read_dir(".")?
    {
        if let Ok(entry) = path
        {
            if let Some(file) = entry.file_name().to_str()
            {
                if file.ends_with(".verify")
                {
                    key_ver_list.push(file.to_string());
                    println!("{}", file);
                }
            }
        }
    }

    let keys = encryption::get_verify_keys(&key_ver_list);

    while let Ok((stream, _addr)) = listen.accept().await
    {
        log::log(
            log::Level::Info,
            &format!("listen completed with {:?}\n", stream),
        );

        let priv_key = priv_key.clone();
        let keys = keys.clone();
        tokio::spawn(async move {
            let mut con = frame::Connection::new(
                stream,
                RsaKey::Private(priv_key),
                SignVerify::MultiVerify(keys),
            )
            .authenticate(frame::Auth::Host)
            .await
            .unwrap();

            loop
            {
                let res = con.read_frame().await.unwrap();
                frame_handler(res, &con);
            }
        });
    }
    Ok(())
}

fn frame_handler(frame: frame::Frame, con: &frame::Connection)
{
    match frame
    {
        frame::Frame::String(s, _) =>
        {
            log::log(log::Level::Normal, &format!("received string: {}", s));
        }
        frame::Frame::Vec(vec, _) =>
        {
            for frame in vec
            {
                frame_handler(frame, con);
            }
        }
        frame::Frame::Message(m, sig) =>
        {
            m.write_to_file().unwrap();
            log::log(
                log::Level::Info,
                &format!("signature is {:?} / status {}", sig, sig == con.session_id),
            );
        }
        frame::Frame::KeyShare(_) => panic!("element only used for authentication"),
        frame::Frame::SessionId(_) => panic!("element only used for authentication"),
    };
}
