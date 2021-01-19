mod lib;
mod rustycraft;

use std::{io::{BufRead, BufReader, LineWriter, Lines}, sync::{Arc, Mutex, mpsc::Receiver}, thread, time::Duration};
use std::net::{TcpListener};

use lib::{event::serialize_event, events::RustyCraftMessage};
use rustycraft::world::World;
use thread::JoinHandle;
use crate::lib::{client::Client, clients::Clients};

const PORT: u32 = 9810;

fn create_read_thread(mut client: Client, clients: Clients, world: Arc<Mutex<World>>) -> JoinHandle<()> {
    thread::spawn(move|| {
        loop {
            let result = client.read();
            match result {
                None => break,
                Some(data) => {
                    println!("Received {:?} from client {}", data, client.id);
                    match data {
                        RustyCraftMessage::GetChunks { coords } => {
                            let mut world = world.lock().unwrap();
                            let mut chunks = Vec::new();
                            for (chunk_x, chunk_z) in coords.iter() {
                                let chunk = world.get_or_insert_chunk(*chunk_x, *chunk_z);
                                println!("Size of chunk: {}", std::mem::size_of::<char>() * chunk.serialized_blocks.len());
                                chunks.push((*chunk_x, *chunk_z, chunk.serialized_blocks.clone()));
                            }

                             // sender is irrelevent so send as empty string
                             let message = serialize_event(String::new(), RustyCraftMessage::ChunkData { chunks });
                             client.send(&message);
                        },
                        RustyCraftMessage::SetBlock { world_x, world_y, world_z, block } => {
                            let mut world = world.lock().unwrap();
                            world.set_block(world_x, world_y, world_z, block);
                            let message = serialize_event(client.id.clone(), data);
                            clients.broadcast(&message);
                        },
                        RustyCraftMessage::Disconnect => {
                            break;
                        }, 
                        _ => {
                            clients.broadcast_to_peers(&data, &client.id);
                        }
                    }
                }
            }
        }

        println!("Client disconnected");
        clients.remove(&client.id);
        clients.broadcast(&format!("{} disconnected", client.id));
    })
}

fn main() {
    // start server
    let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT)).unwrap();
    println!("Server listening on port {}", PORT);

    // create world
    let world = World::new("world");
    let world = Arc::new(Mutex::new(world));

    let clients = Clients::new();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                let client = Client::new(stream);
                let client_copy = client.clone();
                let id = client.id.clone();
                clients.add(client);
                let clients = clients.clone();
                let world = world.clone();
                thread::spawn(move || {
                    clients.broadcast(&format!("{} connected to the server", id));
                    create_read_thread(client_copy, clients, world.clone());
                });
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}

