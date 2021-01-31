mod agent;
mod assets;
mod dispatcher;
mod map;
mod menu;
mod rendering;
mod surface;

use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Instant;

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::mouse::MouseButton;
use sdl2::render::BlendMode;
use sdl2::render::Canvas;
use sdl2::video::Window;

use assets::{Assets, SoundAssets};
use dispatcher::DispatcherState;
use libplen::constants;
use libplen::gamestate;
use libplen::level::{self, Level};
use libplen::math::{vec2, Vec2};
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
    Quit,
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

    // Allows 64 sounds to play simultaneously
    sdl2::mixer::allocate_channels(64);

    let ttf_context = sdl2::ttf::init().expect("Could not initialize SDL ttf");

    let mut name = whoami::username();

    let mut event_pump = sdl.event_pump().expect("Could not get event pump");

    let mut sdl = Some(sdl);

    // TODO: only create a window and load assets once

    'mainloop: loop {
        let menu_state = &mut MenuState::new(my_id);

        video_subsystem.text_input().start();
        menu_state.name = name;

        let sound_assets;
        let player_type;

        {
            let window = video_subsystem
                .window(
                    "MAPP",
                    constants::WINDOW_SIZE as u32,
                    constants::WINDOW_SIZE as u32,
                )
                .fullscreen_desktop()
                .resizable()
                .build()
                .expect("Could not create window");

            let mut canvas = window
                .into_canvas()
                .build()
                .expect("Could not create canvas");
            canvas.set_blend_mode(BlendMode::Blend);
            let texture_creator = canvas.texture_creator();

            let assets = Assets::new(&texture_creator, &ttf_context, SoundAssets::new());

            'menuloop: loop {
                let mut current_mouse_click: Option<(i32, i32)> = None;

                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. } => break 'mainloop,
                        Event::KeyDown {
                            keycode: Some(kc), ..
                        } => match kc {
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
                        Event::MouseButtonDown {
                            x, y, mouse_btn, ..
                        } => match mouse_btn {
                            MouseButton::Left => {
                                current_mouse_click = Some((x, y));
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
                rendering::setup_coordinates(&mut canvas)?;

                let window_size = canvas.logical_size();
                let messages_to_send =
                    menu_state.update(&mut reader, current_mouse_click, window_size);
                for message in messages_to_send {
                    send_client_message(&message, &mut reader.stream);
                }

                let my_player_type = menu_state
                    .game_state
                    .get_player_by_id(my_id)
                    .map(|player| player.player_type);

                menu_state
                    .draw(&mut canvas, &assets, my_player_type)
                    .unwrap();

                if let Some(player) = menu_state.game_state.get_player_by_id(my_id) {
                    player_type = player.player_type;
                    break 'menuloop;
                }
            }

            sound_assets = assets.sounds
        };

        video_subsystem.text_input().stop();

        name = menu_state.name.clone();

        match player_type {
            libplen::player::PlayerType::Agent => {
                let (result, returned_sdl) = agent::gameloop(
                    sdl.take().unwrap(),
                    &mut event_pump,
                    &mut reader,
                    &sound_assets,
                    my_id,
                );
                sdl = Some(returned_sdl);

                match result {
                    StateResult::Quit => break 'mainloop,
                    StateResult::Continue => continue,
                    StateResult::GotoNext => (),
                }
            }
            libplen::player::PlayerType::Dispatcher => {
                let my_sdl = sdl.take().unwrap();
                let video_subsystem = my_sdl.video().expect("Could not initialize SDL video");
                let window = video_subsystem
                    .window(
                        "MAPP",
                        constants::WINDOW_SIZE as u32,
                        constants::WINDOW_SIZE as u32,
                    )
                    .fullscreen_desktop()
                    .resizable()
                    .build()
                    .expect("Could not create window");

                let mut canvas = window
                    .into_canvas()
                    .build()
                    .expect("Could not create canvas");
                canvas.set_blend_mode(BlendMode::Blend);
                let texture_creator = canvas.texture_creator();
                let assets = Assets::new(&texture_creator, &ttf_context, SoundAssets::new());

                let dispatcher_state = &mut DispatcherState::new(my_id);

                let result = 'dispatcher_loop: loop {
                    for event in event_pump.poll_iter() {
                        match event {
                            Event::Window {
                                win_event: WindowEvent::Close,
                                ..
                            }
                            | Event::Quit { .. } => {
                                break 'dispatcher_loop StateResult::Quit;
                            }
                            _ => {}
                        }
                    }
                    dispatcher_state.update(&sound_assets, &mut reader, &event_pump.keyboard_state());

                    canvas.set_draw_color(constants::MENU_BACKGROUND_COLOR);
                    canvas.clear();

                    dispatcher_state.draw(&mut canvas, &assets).unwrap();

                    canvas.present();
                };

                sdl = Some(my_sdl);

                match result {
                    StateResult::Quit => break 'mainloop,
                    StateResult::Continue => continue,
                    StateResult::GotoNext => (),
                }
            }
        }
    }

    Ok(())
}
