use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::assets::Assets;
use crate::rendering;
use libplen::constants;

use libplen::math::{vec2, Vec2};
use libplen::player::{PlayerType, Player};
use libplen::messages::{MessageReader, ServerMessage, ClientMessage};
use libplen::gamestate::GameState;

pub enum ButtonAction {
    SetAgent(u64), // team id
    SetDispatcher(u64),
}

pub struct Button {
    pub pos: Vec2,
    pub h: u32,
    pub w: u32,
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
        let mut menu = MenuState {
            name: String::new(),
            game_state: GameState::new(),
            player_type: PlayerType::Agent,
            team_id: 0,
            buttons: vec![],
            my_id,
        };
        menu.build_menu_buttons();
        menu
    }

    pub fn build_menu_buttons(&mut self) {
        let red_disp_pos = vec2(1./4., constants::MENU_BUTTON_JOIN_DISPATCHER_Y);
        let blue_disp_pos = vec2(3./4., constants::MENU_BUTTON_JOIN_DISPATCHER_Y);
        let red_ag_pos = vec2(1./4., constants::MENU_BUTTON_JOIN_AGENT_Y);
        let blue_ag_pos = vec2(3./4., constants::MENU_BUTTON_JOIN_AGENT_Y);

        let red_disp_btn = Button {
            pos: red_disp_pos,
            h: constants::MENU_BUTTON_HEIGHT,
            w: constants::MENU_BUTTON_WIDTH,
            text: String::from("Join as Dispatcher"),
            color: constants::MENU_RED_BUTTON_COLOR.into(),
            action: ButtonAction::SetDispatcher(constants::TEAM_RED_ID),
        };

        let blue_disp_btn = Button {
            pos: blue_disp_pos,
            h: constants::MENU_BUTTON_HEIGHT,
            w: constants::MENU_BUTTON_WIDTH,
            text: String::from("Join as Dispatcher"),
            color: constants::MENU_BLUE_BUTTON_COLOR.into(),
            action: ButtonAction::SetDispatcher(constants::TEAM_BLUE_ID),
        };

        let red_agent_btn = Button {
            pos: red_ag_pos,
            h: constants::MENU_BUTTON_HEIGHT,
            w: constants::MENU_BUTTON_WIDTH,
            text: String::from("Join as Agent"),
            color: constants::MENU_RED_BUTTON_COLOR.into(),
            action: ButtonAction::SetAgent(constants::TEAM_RED_ID),
        };

        let blue_agent_btn = Button {
            pos: blue_ag_pos,
            h: constants::MENU_BUTTON_HEIGHT,
            w: constants::MENU_BUTTON_WIDTH,
            text: String::from("Join as Agent"),
            color: constants::MENU_BLUE_BUTTON_COLOR.into(),
            action: ButtonAction::SetAgent(constants::TEAM_BLUE_ID),
        };
        self.buttons.push(red_disp_btn);
        self.buttons.push(blue_disp_btn);
        self.buttons.push(red_agent_btn);
        self.buttons.push(blue_agent_btn);
    }

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
        current_mouse_click: Option<(i32, i32)>,
        window_size: (u32, u32),
    ) -> Vec<ClientMessage> {
        let mut messages_to_send = vec![];
        // update game state
        server_reader.fetch_bytes().unwrap();
        for message in server_reader.iter() {
            match bincode::deserialize(&message).unwrap() {
                ServerMessage::GameState(state) => self.game_state = state,
                _ => {}
            }
        }
        self.check_buttons(current_mouse_click, &mut messages_to_send, window_size);
        messages_to_send
    }

    fn perform_button_action(
        &self,
        action: &ButtonAction,
        messages_to_send: &mut Vec<ClientMessage>,
    ) {
        match action {
            ButtonAction::SetAgent(team_id) => {
                messages_to_send.push(ClientMessage::JoinTeam {
                    team_id: *team_id,
                    player_type: PlayerType::Agent,
                    name: self.name.clone(),
                });
            }
            ButtonAction::SetDispatcher(team_id) => {
                messages_to_send.push(ClientMessage::JoinTeam {
                    team_id: *team_id,
                    player_type: PlayerType::Dispatcher,
                    name: self.name.clone(),
                });
            }
        }
    }

    fn check_buttons(
        &mut self,
        current_mouse_click: Option<(i32, i32)>,
        messages_to_send: &mut Vec<ClientMessage>,
        window_size: (u32, u32),
    ) {
        let (width, height) = window_size;
        match current_mouse_click {
            None => {}
            Some(position) => {
                for button in &self.buttons {
                    let (px, py) = position;
                    //let rect = *button.rect;
                    let rx = (button.pos.x * (width as f32)) as i32;
                    let ry = (button.pos.y * (height as f32)) as i32;
                    if (px >= rx && px <= rx + button.w as i32) &&
                       (py >= ry && py <= ry + button.h as i32) {
                        self.perform_button_action(&button.action, messages_to_send);
                    }
                }
            }
        }
    }

    pub fn draw(&mut self, canvas: &mut Canvas<Window>, assets: &Assets, player: &Player) -> Result<(), String> {
        let (width, height) = canvas.logical_size();
        canvas.set_draw_color(constants::MENU_BACKGROUND_COLOR);
        canvas.clear();

        self.draw_background(canvas);

        self.draw_player_name(canvas, assets)?;

        self.draw_buttons(canvas, assets);

        self.draw_player_status(canvas, assets, player, 0);

        canvas.present();
        Ok(())
    }

    fn draw_player_status(
        &mut self,
        canvas: &mut Canvas<Window>,
        assets: &Assets,
        player: &Player,
        team_id: u64,
    ) -> Result<(), String> {
        let (nx, ny) = constants::STATUS_TEXT_POS;
        let disp_ag_text = match player.player_type {
            PlayerType::Agent => "agent",
            PlayerType::Dispatcher => "dispatcher"
        };
        let team_text = if team_id == constants::TEAM_RED_ID { "RED" } else { "BLUE" };
        let text = assets
            .font
            .render(&format!("You are {} in team {}", disp_ag_text, team_text))
            .blended((255, 255, 255))
            .expect("Could not render text");

        let texture_creator = canvas.texture_creator();
        let text_texture = texture_creator.create_texture_from_surface(text).unwrap();

        let res_offset = rendering::calculate_resolution_offset(canvas);
        rendering::draw_texture(canvas, &text_texture, vec2(nx + 10., ny + 10.) + res_offset)
    }

    fn draw_buttons(&mut self, canvas: &mut Canvas<Window>, assets: &Assets) {
        let (width, height) = canvas.logical_size();

        for button in &self.buttons {
            let rx = (button.pos.x * (width as f32)) as i32;
            let ry = (button.pos.y * (height as f32)) as i32;

            canvas.set_draw_color(button.color);
            canvas.fill_rect(Rect::new(rx, ry, button.w, button.h));

            let text = assets
                .font
                .render(&button.text)
                .blended((255, 255, 255))
                .expect("Could not render text");

            let texture_creator = canvas.texture_creator();
            let text_texture = texture_creator.create_texture_from_surface(text).unwrap();

            //let res_offset = rendering::calculate_resolution_offset(canvas);
            rendering::draw_texture(canvas, &text_texture, vec2(rx as f32 + 10., ry as f32 + 10.));
        }
    }

    fn draw_background(&mut self, canvas: &mut Canvas<Window>) {
        let (width, height) = canvas.logical_size();
        let line = Rect::new((width/2 - constants::MENU_CENTER_LINE_WIDTH/2) as i32,
                             0, constants::MENU_CENTER_LINE_WIDTH, height);

        let left = Rect::new(0, 0, (width/2) as u32, height);
        let right = Rect::new((width/2) as i32, 0, (width/2) as u32, height);

        canvas.set_draw_color(constants::MENU_RED_BACKGROUND_COLOR);
        canvas.fill_rect(left).unwrap();

        canvas.set_draw_color(constants::MENU_BLUE_BACKGROUND_COLOR);
        canvas.fill_rect(right).unwrap();

        canvas.set_draw_color(constants::MENU_CENTER_LINE_COLOR);
        canvas.fill_rect(line).unwrap();
    }
}
