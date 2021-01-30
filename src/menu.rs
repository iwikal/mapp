use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::rect::Rect;
use sdl2::pixels::Color;

use crate::assets::Assets;
use crate::rendering;
use libplen::constants;
use libplen::math::vec2;
use libplen::player::PlayerType;
use libplen::messages::{MessageReader, ServerMessage, ClientMessage};
use libplen::gamestate::GameState;


pub enum ButtonAction {
    SetAgent(u64), // team id
    SetDispatcher(u64),
}


pub struct Button {
    pub rect: Rect,
    pub text: String,
    pub color: Color,
    pub action: ButtonAction,
}

pub struct MenuState {
    pub name: String,
    pub player_type: PlayerType,
    pub game_state: GameState,
    pub team_id: u64,
    pub my_id: u64,
    pub buttons: Vec<Button>,
}

impl MenuState {
    pub fn new(my_id: u64) -> MenuState {
        MenuState {
            name: String::new(),
            game_state: GameState::new(),
            player_type: PlayerType::Agent,
            team_id: 0,
            buttons: vec!(),
            my_id: my_id
        }
    }
}

impl MenuState {
    fn draw_player_name(
        &mut self,
        canvas: &mut Canvas<Window>,
        assets: &Assets,
    ) -> Result<(), String> {
        let (nx, ny) = constants::NAME_POS;
        let text = assets
            .font
            .render(&format!("Welcome to MAPP! Enter your name: {}", self.name))
            .blended((255, 255, 255))
            .expect("Could not render text");

        let texture_creator = canvas.texture_creator();
        let text_texture = texture_creator.create_texture_from_surface(text).unwrap();

        let res_offset = rendering::calculate_resolution_offset(canvas);
        rendering::draw_texture(canvas, &text_texture, vec2(nx + 10., ny + 10.) + res_offset)
    }

    pub fn update(
        &mut self,
        server_reader: &mut MessageReader,
        current_mouse_click: Option<(i32, i32)>
    ) -> Vec<ClientMessage> {
        let mut messages_to_send = vec!();
        // update game state
        server_reader.fetch_bytes().unwrap();
        for message in server_reader.iter() {
            match bincode::deserialize(&message).unwrap() {
                ServerMessage::GameState(state) => self.game_state = state,
                _ => {}
            }
        }
        self.check_buttons(current_mouse_click, &mut messages_to_send);
        messages_to_send
    }

    fn perform_button_action(
        &self, action: &ButtonAction, messages_to_send: &mut Vec<ClientMessage>
    ) {
        match action {
            ButtonAction::SetAgent(team_id) => {
                messages_to_send.push(
                    ClientMessage::JoinTeam {
                        team_id: *team_id,
                        player_type: PlayerType::Agent,
                        name: self.name.clone(),
                    }
                );
            }
            ButtonAction::SetDispatcher(team_id) => {
                messages_to_send.push(
                    ClientMessage::JoinTeam {
                        team_id: *team_id,
                        player_type: PlayerType::Dispatcher,
                        name: self.name.clone(),
                    }
                );
            }
        }
    }

    fn check_buttons(
        &mut self,
        current_mouse_click: Option<(i32, i32)>,
        messages_to_send: &mut Vec<ClientMessage>
    ) {
        match current_mouse_click {
            None => {},
            Some(position) => {
                for button in &self.buttons {
                    let rect = *button.rect;
                    let (px, py) = position;
                    if (px >= rect.x && px <= rect.x + rect.w) &&
                       (py >= rect.y && py <= rect.y + rect.h) {
                        self.perform_button_action(&button.action, messages_to_send);
                    }
                }
            }
        }
    }

    pub fn draw(&mut self, canvas: &mut Canvas<Window>, assets: &Assets) -> Result<(), String> {
        let (width, height) = canvas.logical_size();
        canvas.set_draw_color(constants::MENU_BACKGROUND_COLOR);
        canvas.clear();

        self.draw_player_name(canvas, assets)?;

        canvas.present();
        Ok(())
    }
}
