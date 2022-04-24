use rsa::pkcs1::DecodeRsaPublicKey;
use std::env;
use std::fs;
use std::io::stdin;
use std::net::SocketAddr;
use tokio::net::TcpStream;

#[path = "../src/main.rs"]
mod main;
use main::encryption;
use main::frame;
use main::message::Message;

#[tokio::main]
async fn main()
{
    let args: Vec<String> = env::args().collect();
    //env::set_var("RUST_BACKTRACE", "1");
    let local_network = true;

    let add_connect: SocketAddr = if local_network
    {
        "192.168.1.129:8080"
    }
    else
    {
        "179.73.194.65:8083"
    }
    .parse()
    .unwrap();

    let stream = TcpStream::connect(add_connect).await.unwrap();
    println!("connect completed first with {:?}", stream);

    let key =
        rsa::RsaPublicKey::from_pkcs1_pem(&fs::read_to_string("public.pem").unwrap()).unwrap();

    #[allow(unused_mut)]
    let mut con = frame::Connection::new(
        stream,
        encryption::RsaKey::Public(key),
        encryption::SignVerify::Sign(encryption::get_sign_key("key.sign").unwrap()),
    )
    .authenticate(frame::Auth::Client)
    .await
    .unwrap();

    loop
    {
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        println!(
            "{:?}",
            con.write_frame(&frame::Frame::Message(
                Message::new("mensg.txt".to_string(), args[1].to_string(), buf),
                con.session_id
            ))
            .await
            .unwrap()
        );
    }
    /*
    let mut stream = TcpStream::connect("127.0.0.1:6142").await?;
    stream.write_all(b"neymar melhor do mundo").await?;

    let mut buf = vec![];
    stream.readable().await?;
    stream.read_buf(&mut buf).await?;
    println!("{:?}", String::from_utf8(buf));

    let mut stream = TcpStream::connect("127.0.0.1:6142").await?;
    stream.write_all(b"pele melhor do universo").await?;

    let mut buf2 = vec![];
    stream.readable().await?;
    stream.read_buf(&mut buf2).await?;
    println!("{:?}", String::from_utf8(buf2));
    */
    //Ok(())
}
