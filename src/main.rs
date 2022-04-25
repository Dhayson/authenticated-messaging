//#![allow(unused)]
use encryption::{RsaKey, SignVerify};
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
async fn main()
{
    let add_listen: SocketAddr = "192.168.1.129:8080".parse().unwrap();

    let listen = TcpListener::bind(add_listen).await.unwrap();

    let priv_key = fs::read_to_string("private.pem").unwrap();
    let priv_key = RsaPrivateKey::from_pkcs1_pem(&priv_key).unwrap();

    let mut key_ver_list = Vec::new();

    for path in fs::read_dir(".").unwrap()
    {
        if let Ok(entry) = path
        {
            if let Some(file) = entry.file_name().to_str()
            {
                if file.ends_with(".verify")
                {
                    key_ver_list.push(file.to_string());
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
                let res = con.read_frame().await;
                if let Err(err) = frame_handler(res, &con)
                {
                    log::log(log::Level::Info, "frame handling failed");
                    panic!("{}", err);
                }
            }
        });
    }
}

fn frame_handler(
    frame_auth: (std::io::Result<frame::Frame>, bool),
    con: &frame::Connection,
) -> std::io::Result<()>
{
    let frame = frame_auth.0?;
    let auth = frame_auth.1;
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
                frame_handler((Ok(frame), auth), con).ok();
            }
        }
        frame::Frame::Message(m) =>
        {
            m.write_to_file().unwrap();
            log::log(
                log::Level::Info,
                &format!("session id is {:?} / status {}", con.session_id, auth),
            );
        }
    };
    Ok(())
}
