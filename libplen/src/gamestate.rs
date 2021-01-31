use std::collections::HashMap;
use std::sync::mpsc::Receiver;

use serde_derive::{Serialize, Deserialize};

use crate::math::{Vec2, vec2, wrap_around};
use crate::player::{self, Player};

#[derive(Serialize, Deserialize, Clone)]
pub struct GameState {
    pub teams: HashMap<u64, player::Team>,
    pub game_started: bool,
    // put server side game state stuff here
}

impl GameState {
    pub fn new() -> GameState {
        let mut teams = HashMap::new();
        teams.insert(0, player::Team::new(0, "RED".to_string(), (255, 0, 0)));
        teams.insert(1, player::Team::new(1, "BLUE".to_string(), (0, 0, 255)));
        GameState {
            teams,
            game_started: false,
        }
    }

    pub fn update(&mut self, delta: f32) {
        // update game state
    }

    pub fn set_player_name(&mut self, player_id: u64, name: String) {
        let mut player = self.get_mut_player_by_id(player_id).unwrap();
        player.name = name;
    }

    pub fn get_player_by_id(&self, id: u64) -> Option<&player::Player> {
        for (_, team) in &self.teams {
            if Some(id) == team.dispatcher.as_ref().map(|player| player.id) {
                return team.dispatcher.as_ref();
            }
            for agent in &team.agents {
                if id == agent.id {
                    return Some(&agent);
                }
            }
        }
        None
    }

    pub fn get_mut_player_by_id(&mut self, id: u64) -> Option<&mut player::Player> {
        for (_, team) in &mut self.teams {
            if Some(id) == team.dispatcher.as_ref().map(|player| player.id) {
                return Some(team.dispatcher.as_mut().unwrap());
            }
            for agent in &mut team.agents {
                if id == agent.id {
                    return Some(agent);
                }
            }
        }
        None
    }

    pub fn try_add_player_to_team(
        &mut self, player_id: u64, team_id: u64, player_type: player::PlayerType, name: String
     ) {
        for (id, team) in &mut self.teams {
            if team.has_player(player_id) {
                team.remove_player(player_id);
                if team_id == *id {
                    team.try_add_player(player_id, name.clone(), player_type);
                }
            } else {
                if team_id == *id {
                    team.try_add_player(player_id, name.clone(), player_type);
                }
            }
        }
    }

    pub fn remove_player(&mut self, player_id: u64) {
        for (_, team) in &mut self.teams {
            if team.has_player(player_id) {
                team.remove_player(player_id);
                return;
            }
        }
    }
}
