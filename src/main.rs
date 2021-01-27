mod lib;
mod rustycraft;
use std::thread;
use std::net::{TcpListener};
use std::env;
use lib::{event::serialize_event, events::RustyCraftMessage, state::State};
use rustycraft::{chunk_utils::to_serialized};
use thread::JoinHandle;
use crate::lib::client::Client;

const DEFAULT_PORT: u16 = 25566;

fn create_read_thread(mut client: Client, state: State) -> JoinHandle<()> {
    thread::spawn(move|| {
        loop {
            let result = client.read();
            match result {
                None => break,
                Some(data) => {
                    match &data {
                        RustyCraftMessage::GetChunks { coords } => {
                            let mut world = state.world.lock().unwrap();
                            let mut chunks = Vec::new();
                            for (chunk_x, chunk_z) in coords.iter() {
                                let chunk = world.get_or_insert_chunk(*chunk_x, *chunk_z);
                                let serialized = to_serialized(&chunk.blocks_in_mesh, &chunk.blocks);
                                chunks.push((*chunk_x, *chunk_z, serialized));
                            }

                             // sender is irrelevent so send as empty string
                             let message = serialize_event(String::new(), RustyCraftMessage::ChunkData { chunks });
                             client.send(&message);
                        },
                        RustyCraftMessage::PlayerPosition { x, y, z } => {
                            *client.x.lock().unwrap() = *x;
                            *client.y.lock().unwrap() = *y;
                            *client.z.lock().unwrap() = *z;
                            state.clients.broadcast_to_peers(&data, &client.id);
                        },
                        RustyCraftMessage::PlayerJoin { name } => {
                            // 30 char name limit
                            if client.name.lock().unwrap().is_none() && name.len() < 30 {
                                client.set_name(name.clone());
                                println!(
                                    "\u{001b}[33m{} joined the server\u{001b}[0m", 
                                    match client.name.lock().unwrap().clone() {
                                        Some(name) => name,
                                        None => String::from("[Unnamed Player]")
                                    }
                                );
                                let join_y = state.world.lock().unwrap().highest_in_column(0, 0).unwrap();
                                let message_to_broadcast = RustyCraftMessage::PlayerInit { name: name.clone(), x: 0.0, y: join_y as f32, z: 0.0 };
                                state.clients.broadcast(&message_to_broadcast, &client.id);

                                let connection_data = RustyCraftMessage::ConnectionData { 
                                    id: client.id.clone(), 
                                    players: state.clients.clients().iter().map(|c| (
                                        c.id.clone(), 
                                        // tentative, add handling
                                        match &*c.name.lock().unwrap() { 
                                            Some(name) => name.clone(), 
                                            None => String::from("Unnamed Player") 
                                        }, 
                                        *c.x.lock().unwrap(), 
                                        *c.y.lock().unwrap(), 
                                        *c.z.lock().unwrap(), 
                                        *c.yaw.lock().unwrap(), 
                                        *c.pitch.lock().unwrap()
                                    )).collect()
                                };
                                let id_message = serialize_event(String::new(), connection_data);
                                client.send(&id_message)
                            }
                        },
                        RustyCraftMessage::ChatMessage { content } => {
                            println!(
                                "\u{001b}[33m<{}> {}\u{001b}[0m", 
                                match client.name.lock().unwrap().clone() {
                                    Some(name) => name,
                                    None => String::from("[Unnamed Player]")
                                },
                                content
                            );
                            state.clients.broadcast(&data, &client.id);
                        },
                        RustyCraftMessage::SetBlock { world_x, world_y, world_z, block } => {
                            let mut world = state.world.lock().unwrap();
                            world.set_block(*world_x, *world_y, *world_z, *block);
                            state.clients.broadcast(&data, &client.id);
                        },
                        RustyCraftMessage::PlayerDirection { yaw, pitch } => {
                            *client.yaw.lock().unwrap() = *yaw;
                            *client.pitch.lock().unwrap() = *pitch;
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

        println!(
            "\u{001b}[33m{} left the server\u{001b}[0m", 
            match client.name.lock().unwrap().clone() {
                Some(name) => name,
                None => String::from("[Unnamed Player]")
            }
        );
        state.clients.remove(&client.id);
        state.clients.broadcast(&RustyCraftMessage::Disconnect, &client.id);
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut port_to_host = DEFAULT_PORT;
    if args.len() > 1 {
        let port = args[1].parse::<u16>();
        match port {
            Ok(port) => {
                port_to_host = port;
            },
            Err(_) => {
                println!("\u{001b}[31;1mInvalid port number! Please enter a valid number from 0 to 65535\u{001b}[0m");
                return
            }
        }
    }

    // start server
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port_to_host)).unwrap();
    println!("\u{001b}[32;1mSuccessfully started RustyCraft server!\u{001b}[0m");
    println!("\u{001b}[37;1mListening on port {}\u{001b}[0m", port_to_host);

    // initialize server state
    let state = State::new();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
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

