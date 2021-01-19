use std::{io::{BufRead, BufReader, LineWriter, Write}, net::TcpStream};

use uuid::Uuid;

use crate::lib::event::serialize_event;

use super::events::RustyCraftMessage;

pub struct Client {
    pub id: String,
    stream: TcpStream,
    reader: BufReader<TcpStream>,
    writer: LineWriter<TcpStream>
}

impl Client {
    pub fn new(stream: TcpStream) -> Client {
        let reader = BufReader::new(stream.try_clone().unwrap());
        let writer = LineWriter::new(stream.try_clone().unwrap());
        Client { id: Uuid::new_v4().to_string(), stream, reader, writer }
    }

    pub fn send(&mut self, message: &String) {
        self.writer.write(&message.as_bytes()).unwrap();
        self.writer.write(&[b'\n']).unwrap();
    }

    pub fn read(&mut self) -> Option<RustyCraftMessage> {
        let mut buffer = String::new();
        let bytes_read = self.reader.read_line(&mut buffer).unwrap();
        match bytes_read {
            0 => None,
            _ => {
                buffer.pop();
                Some(serde_json::from_str(buffer.as_str()).unwrap())
            }
        }
    }

    pub fn clone(&self) -> Client {
        let reader = BufReader::new(self.stream.try_clone().unwrap());
        let writer = LineWriter::new(self.stream.try_clone().unwrap());
        Client { id: self.id.clone(), stream: self.stream.try_clone().unwrap(), reader, writer }
    }
}