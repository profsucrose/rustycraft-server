use std::{collections::HashMap, sync::{Arc, Mutex}};

use super::{client::Client, event::serialize_event, events::RustyCraftMessage};

pub struct Clients {
    // thread-safe hashmap of UUIDs to Clients
    clients: Arc<Mutex<HashMap<String, Arc<Mutex<Client>>>>>
}

impl Clients {
    pub fn new() -> Clients {
        Clients { clients: Arc::new(Mutex::new(HashMap::new())) }
    }

    pub fn clone(&self) -> Clients {
        Clients { clients: self.clients.clone() }
    }

    pub fn add(&self, client: Client) {
        self.clients.lock().unwrap().insert(client.id.clone(), Arc::new(Mutex::new(client)));
    }

    pub fn remove(&self, id: &String) {
        self.clients.lock().unwrap().remove(id)
            .expect("Failed to remove client");
    }

    pub fn get(&self, id: &String) -> Arc<Mutex<Client>> {
        let client = self.clients.lock().unwrap().get(id).unwrap().clone();
        client.clone()
    }

    pub fn broadcast(&self, message: &String) {
        for (id, client) in self.clients.lock().unwrap().iter() {
            println!("Broadcasting to {}", id);
            client.lock().unwrap().send(message);
        }
    }

    // broadcast to all clients except one 
    pub fn broadcast_to_peers(&self, message: &RustyCraftMessage, sender_id: &String) {
        let message = message.clone();
        let event = serialize_event(sender_id.clone(), message);
        for (id, client) in self.clients.lock().unwrap().iter() {
            if id == sender_id {
                continue;
            }
            client.lock().unwrap().send(&event);
        }
    }
}