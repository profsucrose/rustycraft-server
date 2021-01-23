use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::rustycraft::world::World;

use super::clients::Clients;

// struct for organizing server state
// for each read thread
#[derive(Clone)]
pub struct State {
    pub world: Arc<Mutex<World>>,
    pub names: Arc<Mutex<HashMap<String, String>>>,
    pub clients: Clients
}

impl State {
    pub fn new() -> State {
        let world = World::new("world");
        State {
            world: Arc::new(Mutex::new(world)),
            names: Arc::new(Mutex::new(HashMap::new())),
            clients: Clients::new()
        }
    }
}