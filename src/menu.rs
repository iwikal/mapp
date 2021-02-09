use luminance::context::GraphicsContext;
use luminance::pipeline::{PipelineState, PipelineError};
use luminance::render_state::RenderState;
use luminance::shading_gate::ShadingGate;
use luminance::tess::Tess;
use luminance_gl::GL33;
use luminance_glyph::{GlyphBrush, HorizontalAlign, Layout, VerticalAlign};

use crate::rendering;
use crate::surface::Sdl2Surface;

use libplen::constants;
use libplen::gamestate::GameState;
use libplen::math::{vec2, Vec2};
use libplen::messages::{ClientMessage, MessageReader, ServerMessage};
use libplen::player::{Player, PlayerType};

pub enum ButtonAction {
    SetAgent(u64), // team id
    SetDispatcher(u64),
}

pub struct Button {
    pub pos: Vec2,
    pub size: Vec2,
    pub text: String,
    pub color: [f32; 4],
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
        let button_size = vec2(constants::MENU_BUTTON_WIDTH, constants::MENU_BUTTON_HEIGHT);

        let red_button_x = 1./4. - button_size.x / 2.;
        let blue_button_x = 3./4. - button_size.x / 2.;

        let red_disp_pos = vec2(red_button_x, constants::MENU_BUTTON_JOIN_DISPATCHER_Y);
        let blue_disp_pos = vec2(blue_button_x, constants::MENU_BUTTON_JOIN_DISPATCHER_Y);
        let red_ag_pos = vec2(red_button_x, constants::MENU_BUTTON_JOIN_AGENT_Y);
        let blue_ag_pos = vec2(blue_button_x, constants::MENU_BUTTON_JOIN_AGENT_Y);

        let red_disp_btn = Button {
            pos: red_disp_pos,
            size: button_size,
            text: String::from("Join as Dispatcher"),
            color: constants::MENU_RED_BUTTON_COLOR,
            action: ButtonAction::SetDispatcher(constants::TEAM_RED_ID),
        };

        let blue_disp_btn = Button {
            pos: blue_disp_pos,
            size: button_size,
            text: String::from("Join as Dispatcher"),
            color: constants::MENU_BLUE_BUTTON_COLOR,
            action: ButtonAction::SetDispatcher(constants::TEAM_BLUE_ID),
        };

        let red_agent_btn = Button {
            pos: red_ag_pos,
            size: button_size,
            text: String::from("Join as Agent"),
            color: constants::MENU_RED_BUTTON_COLOR,
            action: ButtonAction::SetAgent(constants::TEAM_RED_ID),
        };

        let blue_agent_btn = Button {
            pos: blue_ag_pos,
            size: button_size,
            text: String::from("Join as Agent"),
            color: constants::MENU_BLUE_BUTTON_COLOR,
            action: ButtonAction::SetAgent(constants::TEAM_BLUE_ID),
        };
        self.buttons.push(red_disp_btn);
        self.buttons.push(blue_disp_btn);
        self.buttons.push(red_agent_btn);
        self.buttons.push(blue_agent_btn);
    }

    fn draw_player_name(&mut self, glyph_brush: &mut GlyphBrush<GL33>, win_size: (u32, u32)) {
        let (nx, ny) = constants::NAME_POS;
        glyph_brush.queue(
            luminance_glyph::Section::default()
                .with_screen_position((nx * win_size.0 as f32, ny * win_size.1 as f32))
                .add_text(
                    luminance_glyph::Text::new(&format!("Welcome to MAPP! Enter your name: {}", self.name))
                        .with_color([1.0, 1.0, 1.0, 1.0])
                        .with_scale(constants::MENU_BUTTON_WIDTH * win_size.0 as f32 * 0.1)
                )
        );
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
            let decoded_msg = bincode::deserialize(&message).unwrap();
            if let ServerMessage::GameState(state) = decoded_msg {
                self.game_state = state;
            }
        }
        if let Some(click_position) = current_mouse_click {
            self.check_buttons(click_position, &mut messages_to_send, window_size);
        }
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
        click_position: (i32, i32),
        messages_to_send: &mut Vec<ClientMessage>,
        window_size: (u32, u32),
    ) {
        dbg!(click_position);

        let (width, height) = window_size;
        let win_size = vec2(width as f32, height as f32);
        let (px, py) = click_position;

        for button in &self.buttons {
            let rx = (button.pos.x * win_size.x) as i32;
            let ry = (button.pos.y * win_size.y) as i32;

            dbg!((rx, ry));

            let button_screen_size = button.size * win_size;

            if (px >= rx && px <= rx + button_screen_size.x as i32)
                && (py >= ry && py <= ry + button_screen_size.y as i32)
            {
                println!("detect");
                self.perform_button_action(&button.action, messages_to_send);
            }
        }
        println!();
    }

    pub fn draw(
        &mut self,
        surface: &mut Sdl2Surface,
        glyph_brush: &mut GlyphBrush<GL33>,
        rect_tess: &Tess<GL33, ()>,
        rect_shader: &mut rendering::RectShader,
        player_type: Option<PlayerType>,
    ) -> Result<(), String> {
        let back_buffer = surface.back_buffer().expect("Could not get back buffer");

        glyph_brush.process_queued(surface);

        let win_size = surface.window().size();

        self.draw_player_name(glyph_brush, win_size);

        if let Some(player_type) = player_type {
            self.draw_player_status(glyph_brush, player_type, 0);
        }

        self.draw_button_texts(glyph_brush, win_size);

        // Create a new dynamic pipeline that will render to the back buffer and must clear it
        // with pitch black prior to do any render to it.
        surface
            .new_pipeline_gate()
            .pipeline(
                &back_buffer,
                &PipelineState::default(),
                |mut pipeline, mut shd_gate| {
                    self.draw_background(rect_tess, &mut shd_gate, rect_shader)?;

                    self.draw_button_backgrounds(rect_tess, &mut shd_gate, rect_shader)?;

                    // Draw text.
                    glyph_brush.draw_queued(&mut pipeline, &mut shd_gate, win_size.0, win_size.1)?;

                    Ok(())
                },
            )
            .assume()
            .into_result()
            .expect("Failed to render");

        surface.window().gl_swap_window();

        Ok(())
    }

    fn draw_player_status(
        &mut self,
        glyph_brush: &mut GlyphBrush<GL33>,
        player_type: PlayerType,
        team_id: u64,
    ) {
        let (nx, ny) = constants::STATUS_TEXT_POS;
        let disp_ag_text = match player_type {
            PlayerType::Agent => "agent",
            PlayerType::Dispatcher => "dispatcher",
        };
        let team_text = if team_id == constants::TEAM_RED_ID {
            "RED"
        } else {
            "BLUE"
        };
        glyph_brush.queue(
            luminance_glyph::Section::default().add_text(
                luminance_glyph::Text::new(&format!("You are {} in team {}", disp_ag_text, team_text))
                    .with_color([1.0, 1.0, 1.0, 1.0])
            )
                .with_screen_position((nx as f32, ny as f32))
        );
    }

    fn draw_button_backgrounds(
        &mut self,
        rect_tess: &Tess<GL33, ()>,
        shd_gate: &mut ShadingGate<GL33>,
        rect_shader: &mut rendering::RectShader,
    ) -> Result<(), PipelineError> {
        for button in &self.buttons {
            shd_gate.shade(rect_shader, |mut iface, uni, mut rdr_gate| {
                iface.set(&uni.position, [button.pos.x, button.pos.y]);
                iface.set(&uni.size, [button.size.x, button.size.y]);
                iface.set(&uni.color, button.color);

                let render_state = RenderState::default()
                    .set_depth_test(None);
                rdr_gate.render(&render_state, |mut tess_gate| {
                    tess_gate.render(rect_tess)
                })
            })?;
        }

        Ok(())
    }

    fn draw_button_texts(
        &mut self,
        glyph_brush: &mut GlyphBrush<GL33>,
        (win_width, win_height): (u32, u32),
    ) {
        let win_size = vec2(win_width as f32, win_height as f32);
        let half_button_size = vec2(
            constants::MENU_BUTTON_WIDTH / 2.,
            constants::MENU_BUTTON_HEIGHT / 2.
        );

        for button in &self.buttons {
            let pos = (button.pos + half_button_size) * win_size;
            let bounds = button.size * win_size;

            glyph_brush.queue(
                luminance_glyph::Section::default()
                    .with_screen_position((pos.x, pos.y))
                    .with_bounds((bounds.x, bounds.y))
                    .with_layout(
                        Layout::default_single_line()
                            .h_align(HorizontalAlign::Center)
                            .v_align(VerticalAlign::Center)
                    )
                    .add_text(
                        luminance_glyph::Text::new(&button.text)
                            .with_color([1.0, 1.0, 1.0, 1.0])
                            .with_scale(constants::MENU_BUTTON_WIDTH * win_size.x * 0.07)
                    )
            );
        }
    }

    fn draw_background(
        &mut self,
        rect_tess: &Tess<GL33, ()>,
        shd_gate: &mut ShadingGate<GL33>,
        rect_shader: &mut rendering::RectShader,
    ) -> Result<(), PipelineError> {
        let line_pos = [
            0.5 - constants::MENU_CENTER_LINE_WIDTH / 2.,
            0.,
        ];
        let line_size = [
            constants::MENU_CENTER_LINE_WIDTH,
            1.,
        ];

        let side_size = [
            0.5 - constants::MENU_CENTER_LINE_WIDTH / 2.,
            1.,
        ];

        let right_pos = [
            line_pos[0] + line_size[0],
            0.,
        ];

        let rects = [
            ([0., 0.], side_size, constants::MENU_RED_BACKGROUND_COLOR),
            (line_pos, line_size, constants::MENU_CENTER_LINE_COLOR),
            (right_pos, side_size, constants::MENU_BLUE_BACKGROUND_COLOR),
        ];

        for &(pos, size, color) in rects.iter() {
            shd_gate.shade(rect_shader, |mut iface, uni, mut rdr_gate| {
                iface.set(&uni.position, pos);
                iface.set(&uni.size, size);
                iface.set(&uni.color, color);

                let render_state = RenderState::default()
                    .set_depth_test(None);
                rdr_gate.render(&render_state, |mut tess_gate| {
                    tess_gate.render(rect_tess)
                })
            })?;
        }

        Ok(())
    }
}
