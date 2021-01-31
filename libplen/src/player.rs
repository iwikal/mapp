use serde_derive::{Serialize, Deserialize};
use crate::constants::PLAYER_SPEED;
use crate::math::{Vec2, vec2};
use crate::messages::ClientInput;
use ultraviolet::Rotor2;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum PlayerType {
    Dispatcher,
    Agent
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub position: Vec2,
    pub rotation: f32,
    pub player_type: PlayerType,
}


impl Player {
    pub fn new(
        id: u64,
        name: String,
        player_type: PlayerType
    ) -> Player {
        Player {
            id,
            name,
            position: vec2(0., 0.),
            rotation: 0.,
            player_type,
        }
    }

    pub fn update(&mut self, delta_time: f32, input: &ClientInput) {
        let &ClientInput {
            rotation,
            x_input,
            y_input,
        } = input;
        self.rotation -= rotation; // No delta time factor here!
        let input_movement = vec2(x_input, y_input)
            .rotated_by(Rotor2::from_angle(-self.rotation));
        self.position += input_movement * PLAYER_SPEED * delta_time;
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Team {
    pub id: u64,
    pub name: String,
    pub color: (u8, u8, u8),
    pub dispatcher: Option<Player>,
    pub agents: Vec<Player>,
}

impl Team {
    pub fn new(id: u64, name: String, color: (u8, u8, u8)) -> Team {
        Team {
            id,
            name,
            color,
            dispatcher: None,
            agents: vec!()
        }
    }

    pub fn try_add_player(&mut self, player_id: u64, name: String, player_type: PlayerType) {
        match player_type {
            PlayerType::Dispatcher => {
                match self.dispatcher {
                    None => {
                        self.dispatcher = Some(Player::new(player_id, name, player_type));
                    },
                    Some(_) => { }, // don't add dispatcher if already taken
                }
            },
            PlayerType::Agent => {
                self.agents.push(Player::new(player_id, name, player_type));
            }
        }
    }

    pub fn has_player(&self, id: u64) -> bool {
        match &self.dispatcher {
            None => { },
            Some(d) => {
                if d.id == id {
                    return true;
                }
            }
        };
        for agent in &self.agents {
            if agent.id == id {
                return true;
            }
        }
        false
    }

    pub fn remove_player(&mut self, id: u64) {
        match &mut self.dispatcher {
            None => { },
            Some(d) => {
                if d.id == id {
                    self.dispatcher = None;
                    return;
                }
            }
        };
        self.agents.retain(|p| p.id != id);
    }
}
