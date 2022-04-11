use mini_redis::Result;
use rsa::pkcs1::DecodeRsaPublicKey;
use std::fs;
use std::io::stdin;
use std::net::SocketAddr;
use tokio::net::TcpStream;

#[path = "../src/main.rs"]
mod main;
use main::encryption;
use main::frame;

#[tokio::main]
async fn main() -> Result<()>
{
    //std::env::set_var("RUST_BACKTRACE", "1");
    let local_network = false;

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

    let mut con = frame::Connection::new(stream, encryption::Key::Public(key));
    loop
    {
        let mut buf_vec = vec![];
        for i in 0..3
        {
            let mut buf = String::new();
            stdin().read_line(&mut buf).unwrap();
            buf_vec.push(frame::Frame::String(buf));
        }
        println!("{:?}", con.write_frame(&frame::Frame::Vec(buf_vec)).await?);
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
