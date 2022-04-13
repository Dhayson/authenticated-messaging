use serde::{self, Deserialize, Serialize};
use std::fs;
use std::io::Result;
use std::io::Write;
use std::time;

#[derive(Debug, Deserialize, Serialize)]
pub struct Message
{
    date: time::SystemTime,
    title: String,
    autor: String,
    content: String,
}

impl Message
{
    pub fn new(title: String, autor: String, content: String) -> Message
    {
        Message {
            date: time::SystemTime::now(),
            title,
            autor,
            content,
        }
    }
    pub fn write_to_file(&self) -> Result<()>
    {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.title)
            .unwrap();
        file.write_all(
            format!(
                "autor: {}\n\n{}\ndate:{:?}\n============================\n",
                self.autor, self.content, self.date
            )
            .as_bytes(),
        )
    }
}
