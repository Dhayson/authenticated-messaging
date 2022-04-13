use bytes::{Buf, BytesMut};
use serde::{self, Deserialize, Serialize};
use std::io::{Error, ErrorKind};
use tokio::io::Result;
use tokio::net::TcpStream;

use super::encryption::{self, RsaKey};
use super::log::{log, Level};
use super::message::Message;

#[derive(Debug, Deserialize, Serialize)]
pub enum Frame
{
    String(String),
    Vec(Vec<Frame>),
    Message(Message),
}

pub struct Connection
{
    stream: TcpStream,
    buffer: BytesMut,
    key: RsaKey, // ... other fields here
}

impl Connection
{
    pub fn new(stream: TcpStream, key: RsaKey) -> Connection
    {
        Connection {
            stream,
            // Allocate the buffer with 4kb of capacity.
            buffer: BytesMut::with_capacity(4096),
            key,
        }
    }
    /// Read a frame from the connection.
    ///
    /// Returns `None` if EOF is reached
    pub async fn read_frame(&mut self) -> Result<Frame>
    {
        loop
        {
            //try get frame from buffer
            if let Some(frame) = self.parse_frame()
            {
                return Ok(frame);
            }

            self.stream.readable().await?;
            match self.stream.try_read_buf(&mut self.buffer)
            {
                Ok(0) => return Err(Error::from(ErrorKind::ConnectionReset)),

                Ok(_) => log(
                    Level::Debug,
                    &format!("current buffer: {}", String::from_utf8_lossy(&self.buffer)),
                ),

                Err(error) if error.kind() == ErrorKind::WouldBlock => continue,

                Err(error) => unimplemented!("{:?}", error),
            };
        }
    }

    /// Write a frame to the connection.
    pub async fn write_frame(&mut self, frame: &Frame) -> Result<()>
    {
        let parse_frame = ron::to_string(frame).unwrap();

        let parse_frame_encrypted = match encryption::aes_encrypt(&self.key, parse_frame.as_bytes())
        {
            Ok(result) => result,
            Err(err) => panic!("could not encrypt the frame with respective key"),
        };

        //there's a better solution. This is only needed because the write buffer has
        //to end with '\0' with the current logic
        let final_buf = ron::to_string(&parse_frame_encrypted).unwrap() + "\0";

        loop
        {
            self.stream.writable().await?;
            let mut bytes_written = 0;
            match self
                .stream
                .try_write(&final_buf[bytes_written..].as_bytes())
            {
                Ok(n) =>
                {
                    bytes_written += n;
                    if bytes_written == final_buf.len()
                    {
                        break;
                    }
                }

                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,

                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
        // implementation here
    }

    fn parse_frame(&mut self) -> Option<Frame>
    {
        let mut frame_len = None;
        let mut index = 0;

        for i in self.buffer.chunk()
        {
            index += 1;
            //frame comes invalid if there's \0 where it shouldn't
            if *i == '\0' as u8
            {
                frame_len = Some(index);
                break;
            }
        }
        if frame_len.is_none()
        {
            return None; //unfinished
        }

        let frame_len = frame_len.unwrap();

        let (frame, _) = self.buffer.chunk().split_at(frame_len - 1);

        let res = (|| {
            let frame =
                &ron::from_str::<(Vec<u8>, Vec<u8>)>(&String::from_utf8_lossy(&frame)).ok()?; //wrong parse

            let frame = encryption::aes_decrypt(&self.key, &frame.0, &frame.1).ok()?; //wrong encryption

            ron::from_str(&String::from_utf8_lossy(&frame)).ok() //invalid frame
        })();

        self.buffer.advance(frame_len);

        return res;
        //NOTE: returns none if frame is invalid
    }
}
