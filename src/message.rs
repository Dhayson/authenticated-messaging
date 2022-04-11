use serde::{self, Deserialize, Serialize};
use std::fs;
use std::io::Result;
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
        fs::write(
            &self.title,
            format!(
                "autor: {}\n\n{}\n\ndate:{:?}\n",
                self.autor, self.content, self.date
            ),
        )
    }
}
