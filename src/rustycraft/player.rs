use cgmath::{Angle, Deg, InnerSpace, Vector3};

use crate::lib::direction::Direction;

#[derive(Debug)]
pub struct Player {
    position: Vector3<f32>,
    front: Vector3<f32>,
    up: Vector3<f32>,
    yaw: f32,
    pitch: f32,
    pub name: String,
    speed: f32
}

impl Player {
    pub fn new(name: String, position: Vector3<f32>) -> Player {
        Player { 
            position, 
            front: Vector3::new(1.0, 0.0, 0.0), 
            up: Vector3::new(0.0, 1.0, 0.0), 
            yaw: -90.0,
            pitch: 45.0,
            name, 
            speed: 0.08
        }
    }

    pub fn mouse_callback(&mut self, x_offset: f32, y_offset: f32) {
        let sensitivity = 0.3;
        let x_offset = x_offset * sensitivity;
        let y_offset = y_offset * sensitivity;

        self.yaw += x_offset;
        self.pitch += y_offset;

        // clamp pitch
        if self.pitch > 89.0 {
            self.pitch = 89.0;
        } else if self.pitch < -89.0 {
            self.pitch = -89.0;
        }

        let direction = Vector3::new(
            Deg(self.yaw).cos() * Deg(self.pitch).cos(),
            Deg(self.pitch).sin(),
            Deg(self.yaw).sin() * Deg(self.pitch).cos()
        );
        self.front = direction.normalize();
    }

    pub fn move_player(&mut self, direction: Direction) {
        let old_y = self.position.y;
        let front_horiz = Vector3::new(self.front.x, 0.0, self.front.z).normalize();
        match direction {
            Direction::Forward => self.position += self.speed * front_horiz,
            Direction::Backward => self.position -= self.speed * front_horiz,
            Direction::Right => self.position += self.speed * self.front.cross(self.up).normalize(),
            Direction::Left => self.position -= self.speed * self.front.cross(self.up).normalize()
        }
        self.position.y = old_y;
    }
}