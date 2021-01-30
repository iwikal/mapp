mod assets;
mod map;
mod menu;
mod rendering;
mod surface;
mod agent;

use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Instant;

use sdl2::mouse::MouseButton;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::render::BlendMode;
use sdl2::render::Canvas;
use sdl2::video::Window;

use assets::Assets;
use libplen::constants;
use libplen::gamestate;
use libplen::math::Vec2;
use libplen::messages::{ClientInput, ClientMessage, MessageReader, ServerMessage, SoundEffect};
use menu::MenuState;

pub fn send_client_message(msg: &ClientMessage, stream: &mut TcpStream) {
    let data = bincode::serialize(msg).expect("Failed to encode message");
    let length = data.len() as u16;
    stream
        .write(&length.to_be_bytes())
        .expect("Failed to send message length to server");
    stream
        .write(&data)
        .expect("Failed to send message to server");
}

#[derive(PartialEq)]
pub enum StateResult {
    Continue,
    GotoNext,
// <<<<<<< HEAD
// }
// 
// struct MainState {
//     my_id: u64,
//     game_state: gamestate::GameState,
//     map: map::Map,
//     last_time: Instant,
// }
// 
// impl MainState {
//     fn new(my_id: u64) -> MainState {
//         MainState {
//             my_id,
//             game_state: gamestate::GameState::new(),
//             map: map::Map::new(),
//             last_time: Instant::now(),
//         }
//     }
// 
//     fn update(
//         &mut self,
//         assets: &Assets,
//         server_reader: &mut MessageReader,
//         keyboard_state: &sdl2::keyboard::KeyboardState,
//     ) -> StateResult {
//         let elapsed = self.last_time.elapsed();
//         self.last_time = Instant::now();
//         let dt_duration = std::time::Duration::from_millis(1000 / 60);
//         if elapsed < dt_duration {
//             std::thread::sleep(dt_duration - elapsed);
//         }
// 
//         server_reader.fetch_bytes().unwrap();
// 
//         for message in server_reader.iter() {
//             match bincode::deserialize(&message).unwrap() {
//                 ServerMessage::AssignId(_) => {
//                     panic!("Got new ID after intialisation")
//                 }
//                 ServerMessage::GameState(state) => self.game_state = state,
//                 ServerMessage::PlaySound(sound, pos) => {
//                     fn play_sound(soundeffect: &sdl2::mixer::Chunk) {
//                         if let Err(e) = sdl2::mixer::Channel::all().play(soundeffect, 0) {
//                             println!("SDL mixer error: {}", e);
//                         }
//                     }
// 
//                     match sound {
//                         SoundEffect::Powerup => {
//                             play_sound(&assets.powerup);
//                         }
//                         SoundEffect::Gun => {
//                             play_sound(&assets.gun);
//                         }
//                         SoundEffect::Explosion => {
//                             play_sound(&assets.explosion);
//                         }
//                         SoundEffect::LaserCharge => {
//                             play_sound(&assets.laser_charge_sound);
//                         }
//                         SoundEffect::LaserFire => {
//                             play_sound(&assets.laser_fire_sound);
//                         }
//                     }
//                 }
//             }
//         }
// 
//         let mut input = ClientInput::new();
//         if keyboard_state.is_scancode_pressed(Scancode::W) {
//             input.y_input -= 1.0;
//         }
//         if keyboard_state.is_scancode_pressed(Scancode::S) {
//             input.y_input += 1.0;
//         }
// 
//         if keyboard_state.is_scancode_pressed(Scancode::A) {
//             input.x_input -= 1.0;
//         }
//         if keyboard_state.is_scancode_pressed(Scancode::D) {
//             input.x_input += 1.0;
//         }
// 
//         self.map
//             .update(elapsed.as_secs_f32(), &self.game_state, self.my_id);
// 
//         let input_message = ClientMessage::Input(input);
//         send_client_message(&input_message, &mut server_reader.stream);
// 
//         StateResult::Continue
//     }
// 
//     fn draw(&mut self, canvas: &mut Canvas<Window>, assets: &mut Assets) -> Result<(), String> {
//         self.map.draw(self.my_id, canvas)?;
// 
//         // for player in &self.game_state.players {
//         //     let w = 10;
//         //     let h = 10;
// 
//         //     let dest_rect = sdl2::rect::Rect::new(
//         //         player.position.x as i32 - w as i32 / 2,
//         //         player.position.y as i32 - h as i32 / 2,
//         //         w as u32,
//         //         h as u32,
//         //     );
//         //     canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 25, 25));
// 
//         //     canvas.fill_rect(dest_rect);
//         // }
// 
//         Ok(())
//     }
// =======
    Quit,
// >>>>>>> c164495aa2f4562c5fc18eb5dcf22a46971776c1
}

pub fn main() -> Result<(), String> {
    let host = std::env::var("SERVER").unwrap_or(String::from("localhost:4444"));
    let stream = TcpStream::connect(host).expect("Could not connect to server");
    println!("Connected to server");

    stream
        .set_nonblocking(true)
        .expect("Could not set socket as nonblocking");
    let mut reader = MessageReader::new(stream);

    let msg = loop {
        reader.fetch_bytes().unwrap();
        if let Some(msg) = reader.iter().next() {
            break bincode::deserialize(&msg).unwrap();
        }
    };

    let my_id = if let ServerMessage::AssignId(id) = msg {
        println!("Received the id {}", id);
        id
    } else {
        panic!("Expected to get an id from server")
    };

    let sdl = sdl2::init().expect("Could not initialize SDL");
    let video_subsystem = sdl.video().expect("Could not initialize SDL video");

    let _audio = sdl.audio().expect("Could not initialize SDL audio");
    let frequency = 44_100;
    let format = sdl2::mixer::AUDIO_S16LSB; // signed 16 bit samples, in little-endian byte order
    let channels = sdl2::mixer::DEFAULT_CHANNELS; // Stereo
    let chunk_size = 1_024;
    sdl2::mixer::open_audio(frequency, format, channels, chunk_size)
        .expect("Could not open SDL mixer audio");
    let _mixer_context =
        sdl2::mixer::init(sdl2::mixer::InitFlag::OGG).expect("Could not initialize SDL mixer");

    let mut name = whoami::username();

    let mut event_pump = sdl.event_pump().expect("Could not get event pump");

    let mut sdl = Some(sdl);

    'mainloop: loop {
        let menu_state = &mut MenuState::new(my_id);

        video_subsystem.text_input().start();
        menu_state.name = name;

// <<<<<<< HEAD
//         'menuloop: loop {
//             let mut current_mouse_click: Option<(i32, i32)> = None;
// 
//             for event in event_pump.poll_iter() {
//                 match event {
//                     Event::Quit { .. } => break 'mainloop,
//                     Event::KeyDown {
//                         keycode: Some(kc), ..
//                     } => match kc {
//                         Keycode::Return => {
//                             break 'menuloop;
//                         }
//                         Keycode::Backspace => {
//                             menu_state.name.pop();
// =======
        {
            let window = video_subsystem
                .window(
                    "very nice gem",
                    constants::WINDOW_SIZE as u32,
                    constants::WINDOW_SIZE as u32,
                )
                .resizable()
                .build()
                .expect("Could not create window");

            let mut canvas = window
                .into_canvas()
                .build()
                .expect("Could not create canvas");
            canvas.set_blend_mode(BlendMode::Blend);
            let texture_creator = canvas.texture_creator();

            // Allows 64 sounds to play simultaneously
            sdl2::mixer::allocate_channels(64);

            let ttf_context = sdl2::ttf::init().expect("Could not initialize SDL ttf");

            let mut assets = Assets::new(&texture_creator, &ttf_context);

            'menuloop: loop {
                let mut current_mouse_click: Option<(i32, i32)> = None;

                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. } => break 'mainloop,
                        Event::KeyDown {
                            keycode: Some(kc), ..
                        } => match kc {
                            Keycode::Return => {
                                break 'menuloop;
                            }
                            Keycode::Backspace => {
                                menu_state.name.pop();
                            }
                            _ => {}
                        },
                        Event::TextInput { text, .. } => {
                            if menu_state.name.chars().count() < 20 {
                                menu_state.name += &text;
                            }
                        }
                        Event::MouseButtonDown { x, y, mouse_btn, .. } => {
                            match mouse_btn {
                                MouseButton::Left => {
                                    current_mouse_click = Some((x, y));
                                },
                                _ => { }
                            }
                        }
                        _ => {}
                    }
                }
                rendering::setup_coordinates(&mut canvas)?;

                let messages_to_send = menu_state.update(&mut reader, current_mouse_click);
                for message in messages_to_send {
                    send_client_message(&message, &mut reader.stream);
                }

                menu_state.draw(&mut canvas, &assets).unwrap();
            }
        }

        video_subsystem.text_input().stop();

        name = menu_state.name.clone();

        // send_client_message(
        //     &ClientMessage::JoinGame {
        //         name: menu_state.name.clone(),
        //     },
        //     &mut reader.stream,
        // );

        let (result, returned_sdl) = agent::gameloop(sdl.take().unwrap(), &mut event_pump, my_id);
        sdl = Some(returned_sdl);

        match result {
            StateResult::Quit => break 'mainloop,
            StateResult::Continue => continue,
            StateResult::GotoNext => (),
        }
    }

    Ok(())
}
