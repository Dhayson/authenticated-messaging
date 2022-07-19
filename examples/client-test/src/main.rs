
use rsa::pkcs1::DecodeRsaPublicKey;
use std::env;
use std::fs;
use std::io::stdin;
use std::net::SocketAddr;
use tokio::net::TcpStream;

use tokiorust::encryption;
use tokiorust::frame;
use tokiorust::message::Message;

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
        rsa::RsaPublicKey::from_pkcs1_pem(&fs::read_to_string("../../public.pem").unwrap()).unwrap();

    #[allow(unused_mut)]
    let mut con = frame::Connection::new(
        stream,
        encryption::RsaKey::Public(key),
        encryption::SignVerify::Sign(encryption::get_sign_key("../../key.sign").unwrap()),
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
            con.write_frame(frame::Frame::Message(Message::new(
                "mensg.txt".to_string(),
                args[1].to_string(),
                buf
            )))
            .await
            .unwrap()
        );
    }
}