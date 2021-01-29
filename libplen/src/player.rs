use serde_derive::{Serialize, Deserialize};
use crate::constants::PLAYER_SPEED;
use crate::math::{Vec2, vec2};
use crate::messages::ClientInput;


#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub position: Vec2,
}


impl Player {
    pub fn new(
        id: u64,
        name: String
    ) -> Player {
        Player {
            id: id,
            name: name,
            position: vec2(0., 0.),
        }
    }

    pub fn update(&mut self, delta_time: f32, input: &ClientInput) {
        self.position.x += delta_time * input.x_input * PLAYER_SPEED;
        self.position.y += delta_time * input.y_input * PLAYER_SPEED;
    }
}
