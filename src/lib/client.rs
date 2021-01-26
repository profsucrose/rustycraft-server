use std::{io::{BufRead, BufReader, LineWriter, Write}, net::TcpStream, sync::{Arc, Mutex}};
use uuid::Uuid;

use super::events::RustyCraftMessage;

// struct to represent player-server connection read/writing 
// and server player data
pub struct Client {
    pub id: String,
    pub name: Arc<Mutex<Option<String>>>,
    stream: TcpStream,
    reader: BufReader<TcpStream>,
    writer: LineWriter<TcpStream>,
    pub x: Arc<Mutex<f32>>,
    pub y: Arc<Mutex<f32>>,
    pub z: Arc<Mutex<f32>>,
    pub pitch: Arc<Mutex<f32>>,
    pub yaw: Arc<Mutex<f32>> 
}

impl Clone for Client {
    fn clone(&self) -> Self {
        let reader = BufReader::new(self.stream.try_clone().unwrap());
        let writer = LineWriter::new(self.stream.try_clone().unwrap());
        Client { 
            id: self.id.clone(), 
            stream: self.stream.try_clone().unwrap(), 
            reader, 
            writer, 
            name: self.name.clone(), 
            x: self.x.clone(), 
            y: self.y.clone(), 
            z: self.z.clone(), 
            pitch: self.pitch.clone(), 
            yaw: self.yaw.clone() 
        }
    }
}

impl Client {
    pub fn new(stream: TcpStream) -> Client {
        let reader = BufReader::new(stream.try_clone().unwrap());
        let writer = LineWriter::new(stream.try_clone().unwrap());

        // name is not set until SetName packet is received
        let name = Arc::new(Mutex::new(None));
        let x = Arc::new(Mutex::new(0.0));
        let y = Arc::new(Mutex::new(0.0));
        let z = Arc::new(Mutex::new(0.0));
        let pitch = Arc::new(Mutex::new(0.0));
        let yaw = Arc::new(Mutex::new(-90.0));
        Client { id: Uuid::new_v4().to_string(), name, stream, reader, writer, x, y, z, pitch, yaw }
    }
    
    pub fn set_name(&mut self, name: String) {
        *self.name.lock().unwrap() = Some(name);
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
}