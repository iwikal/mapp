use std::time::Instant;

use sdl2::keyboard::Scancode;
use sdl2::render::Canvas;
use sdl2::video::Window;

use libplen::level::{self, Level};
use libplen::messages::{ClientInput, ClientMessage, MessageReader, ServerMessage};
use libplen::player;

use crate::{assets::{Assets, SoundAssets}, gamestate, map, StateResult};

pub struct DispatcherState {
    my_id: u64,
    game_state: gamestate::GameState,
    map: map::Map,
    last_time: Instant,
}

impl DispatcherState {
    pub fn new(my_id: u64) -> DispatcherState {
        DispatcherState {
            my_id,
            game_state: gamestate::GameState::new(),
            map: map::Map::new(level::example_level()),
            last_time: Instant::now(),
        }
    }

    pub fn update(
        &mut self,
        _sounds: &SoundAssets,
        server_reader: &mut MessageReader,
        keyboard_state: &sdl2::keyboard::KeyboardState,
    ) -> StateResult {
        let elapsed = self.last_time.elapsed();
        self.last_time = Instant::now();
        let dt_duration = std::time::Duration::from_millis(1000 / 60);
        if elapsed < dt_duration {
            std::thread::sleep(dt_duration - elapsed);
        }

        server_reader.fetch_bytes().unwrap();

        for message in server_reader.iter() {
            match bincode::deserialize(&message).unwrap() {
                ServerMessage::AssignId(_) => {
                    panic!("Got new ID after intialisation")
                }
                ServerMessage::GameState(state) => self.game_state = state,
                ServerMessage::PlaySound(_sound, _pos) => {
                }
            }
        }

        let mut input = ClientInput::new();
        if keyboard_state.is_scancode_pressed(Scancode::W) {
            input.y_input -= 1.0;
        }
        if keyboard_state.is_scancode_pressed(Scancode::S) {
            input.y_input += 1.0;
        }

        if keyboard_state.is_scancode_pressed(Scancode::A) {
            input.x_input -= 1.0;
        }
        if keyboard_state.is_scancode_pressed(Scancode::D) {
            input.x_input += 1.0;
        }

        self.map
            .update(elapsed.as_secs_f32(), &self.game_state, self.my_id);

        let input_message = ClientMessage::Input(input);
        crate::send_client_message(&input_message, &mut server_reader.stream);

        StateResult::Continue
    }

    fn _myself(&self) -> &player::Player {
        let Self { my_id, game_state, .. } = self;
        game_state.get_player_by_id(*my_id).unwrap()
    }

    pub fn draw(&mut self, canvas: &mut Canvas<Window>, _assets: &Assets) -> Result<(), String> {
        self.map.draw(canvas)?;

        Ok(())
    }
}
