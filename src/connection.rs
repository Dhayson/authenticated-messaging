use bytes::Bytes;
use mini_redis::client::Client;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender as SenderMpsc;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Sender as SenderOneshot;

#[derive(Debug)]
pub enum Actions
{
    Get
    {
        key: String,
        sender: SenderOneshot<Result<Option<Bytes>, Box<dyn std::error::Error + Send + Sync>>>,
    },
    Set
    {
        key: String,
        value: Bytes,
        sender: SenderOneshot<Result<(), Box<dyn std::error::Error + Send + Sync>>>,
    },
}

pub struct Connection
{
    sender: SenderMpsc<Actions>,
}

impl Clone for Connection
{
    fn clone(&self) -> Self
    {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl Connection
{
    pub async fn with_buffer(mut client: Client, buffer: usize) -> Connection
    {
        let (sender, mut receiver) = mpsc::channel::<Actions>(buffer);
        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await
            {
                match message
                {
                    Actions::Get {
                        key,
                        sender,
                    } =>
                    {
                        println!("getting value in key {}", key);
                        let value = client.get(&key).await;
                        sender.send(value).unwrap();
                    }
                    Actions::Set {
                        key,
                        value,
                        sender,
                    } =>
                    {
                        println!("setting value {:?} in key {}", value, key);
                        let value = client.set(&key, value).await;
                        sender.send(value).unwrap();
                    }
                };
            }
        });
        Connection {
            sender,
        }
    }

    pub async fn new(client: Client) -> Connection
    {
        Connection::with_buffer(client, 32).await
    }

    pub async fn get(
        &self,
        key: String,
    ) -> Result<Option<Bytes>, Box<dyn std::error::Error + Send + Sync>>
    {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .send(Actions::Get {
                key,
                sender,
            })
            .await
            .unwrap();

        let result = receiver.await;
        return result.unwrap();
    }

    pub async fn set(
        &self,
        key: String,
        value: Bytes,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    {
        let (os_sender, os_receiver) = oneshot::channel();

        self.sender
            .send(Actions::Set {
                key,
                value,
                sender: os_sender,
            })
            .await
            .unwrap();

        return os_receiver.await.unwrap();
    }
}
