mod lib;
mod rustycraft;

use std::{io::{BufRead, BufReader, LineWriter, Lines}, sync::{Arc, Mutex, mpsc::Receiver}, thread, time::Duration};
use std::net::{TcpListener};

use lib::{event::{RustyCraftEvent, serialize_event}, events::RustyCraftMessage, state::State};
use rustycraft::{chunk_utils::to_serialized, world::{self, World}};
use thread::JoinHandle;
use crate::lib::{client::Client, clients::Clients};

const PORT: u32 = 9810;

fn create_read_thread(mut client: Client, state: State) -> JoinHandle<()> {
    thread::spawn(move|| {
        loop {
            let result = client.read();
            match result {
                None => break,
                Some(data) => {
                    println!("Received {:?} from client {}", data, client.id);
                    match &data {
                        RustyCraftMessage::GetChunks { coords } => {
                            let mut world = state.world.lock().unwrap();
                            let mut chunks = Vec::new();
                            for (chunk_x, chunk_z) in coords.iter() {
                                let chunk = world.get_or_insert_chunk(*chunk_x, *chunk_z);
                                let serialized = to_serialized(&chunk.blocks_in_mesh, &chunk.blocks);
                                println!("Size of chunk: {}", std::mem::size_of::<char>() * serialized.len());
                                chunks.push((*chunk_x, *chunk_z, serialized));
                            }

                             // sender is irrelevent so send as empty string
                             let message = serialize_event(String::new(), RustyCraftMessage::ChunkData { chunks });
                             client.send(&message);
                        },
                        RustyCraftMessage::PlayerPosition { x, y, z } => {
                            client.x = *x;
                            client.y = *y;
                            client.z = *z;
                            state.clients.broadcast_to_peers(&data, &client.id);
                        },
                        RustyCraftMessage::PlayerJoin { name } => {
                            // 30 char name limit
                            if client.name.lock().unwrap().is_none() && name.len() < 30 {
                                client.set_name(name.clone());
                                println!("{:?}", client.name);
                                let join_y = state.world.lock().unwrap().highest_in_column(0, 0).unwrap();
                                let message_to_broadcast = RustyCraftMessage::PlayerInit { name: name.clone(), x: 0.0, y: join_y as f32, z: 0.0 };
                                state.clients.broadcast_to_peers(&message_to_broadcast, &client.id);

                                let connection_data = RustyCraftMessage::ConnectionData { 
                                    id: client.id.clone(), 
                                    players: state.clients.clients().iter().map(|c| {
                                        println!("Client: {} - {:?}", c.id, c.name); 
                                        (
                                        c.id.clone(), 
                                        // tentative, add handling
                                        match &*c.name.lock().unwrap() { 
                                            Some(name) => name.clone(), 
                                            None => String::from("Unnamed Player") 
                                        }, 
                                        c.x, 
                                        c.y, 
                                        c.z, 
                                        c.yaw, 
                                        c.pitch
                                    )}).collect()
                                };
                                let id_message = serialize_event(String::new(), connection_data);
                                client.send(&id_message)
                            }
                        },
                        RustyCraftMessage::ChatMessage { content: _ } => {
                            state.clients.broadcast(&data, &client.id);
                        },
                        RustyCraftMessage::SetBlock { world_x, world_y, world_z, block } => {
                            let mut world = state.world.lock().unwrap();
                            world.set_block(*world_x, *world_y, *world_z, *block);
                            state.clients.broadcast(&data, &client.id);
                        },
                        RustyCraftMessage::PlayerMouseMove { x_offset, y_offset } => {
                            client.yaw += x_offset;
                            client.pitch -= y_offset;
                            state.clients.broadcast_to_peers(&data, &client.id);
                        },
                        RustyCraftMessage::Disconnect => break, 
                        _ => {
                            state.clients.broadcast_to_peers(&data, &client.id);
                        }
                    }
                }
            }
        }

        state.clients.remove(&client.id);
        state.clients.broadcast(&RustyCraftMessage::Disconnect, &client.id);
    })
}

fn main() {
    // start server
    let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT)).unwrap();
    println!("Server listening on port {}", PORT);

    // initialize server state
    let state = State::new();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                let client = Client::new(stream);
                let client_copy = client.clone();
                state.clients.add(client);
                let state = state.clone();
                thread::spawn(move || {
                    create_read_thread(client_copy, state);
                });
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}

