use bytes::Bytes;
//#[allow(unused_imports)]
use mini_redis::{Connection, Frame};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
type Database = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main()
{
    // Bind the listener to the address
    let listener = TcpListener::bind("192.168.1.129:6379").await.unwrap();

    let arc = Arc::new(Mutex::new(HashMap::new()));

    loop
    {
        // The second item contains the IP and port of the new connection.
        let (socket, address) = listener.accept().await.unwrap();
        println!("{:?}", address);
        let db = arc.clone();

        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream, db: Database)
{
    use mini_redis::Command::{self, Get, Set};

    // Connection, provided by `mini-redis`, handles parsing frames from
    // the socket
    let mut connection = Connection::new(socket);

    // Use `read_frame` to receive a command from the connection.
    while let Some(frame) = connection.read_frame().await.unwrap()
    {
        let response;

        {
            let mut db = db.lock().unwrap();
            response = match Command::from_frame(frame).unwrap()
            {
                Set(cmd) =>
                {
                    // The value is stored as `Vec<u8>`
                    db.insert(cmd.key().to_string(), cmd.value().clone());
                    println!("set key {:?} to {:?}", cmd.key(), cmd.value());
                    Frame::Simple("OK".to_string())
                }
                Get(cmd) =>
                {
                    if let Some(value) = db.get(cmd.key())
                    {
                        // `Frame::Bulk` expects data to be of type `Bytes`. This
                        // type will be covered later in the tutorial. For now,
                        // `&Vec<u8>` is converted to `Bytes` using `into()`.
                        println!("got value {:?} in key {:?}", value, cmd.key());
                        Frame::Bulk(value.clone().into())
                    }
                    else
                    {
                        Frame::Null
                    }
                }
                cmd => panic!("unimplemented {:?}", cmd),
            };
        }

        // Write the response to the client
        connection.write_frame(&response).await.unwrap();
    }
}