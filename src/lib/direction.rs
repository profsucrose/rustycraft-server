use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum Direction {
    Right,
    Left,
    Forward,
    Backward
}