mod agent;
mod assets;
mod dispatcher;
mod map;
mod menu;
mod rendering;
mod shader;
mod surface;

use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Instant;

use luminance::context::GraphicsContext;
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

    let mut surface = surface::Sdl2Surface::build_with(sdl, |video| {
        video.gl_attr().set_stencil_size(8);
        let mut wb = video.window(
            "MAPP",
            constants::WINDOW_SIZE as u32,
            constants::WINDOW_SIZE as u32,
        );
        wb.fullscreen_desktop().resizable();
        wb
    })
    .expect("Could not create rendering surface");

    let mut glyph_brush = {
        let ttf = include_bytes!("../resources/yoster.ttf");
        let font = ab_glyph::FontArc::try_from_slice(ttf).expect("Could not load font");
        luminance_glyph::GlyphBrushBuilder::using_font(font).build(&mut surface)
    };

    let rect_tess = surface
        .new_tess()
        .set_vertex_nb(4)
        .set_mode(luminance::tess::Mode::TriangleFan)
        .build()
        .unwrap();

    let mut rect_program = {
        let vs = include_str!("../shaders/rect.vert");
        let fs = include_str!("../shaders/rect.frag");
        shader::compile_shader::<(), (), rendering::RectInterface>(&mut surface, vs, fs)
    };

    'mainloop: loop {
        let menu_state = &mut MenuState::new(my_id);

        video_subsystem.text_input().start();
        menu_state.name = name;

        let player_type;

        {
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

                let messages_to_send =
                    menu_state.update(&mut reader, current_mouse_click, surface.window().size());
                for message in messages_to_send {
                    send_client_message(&message, &mut reader.stream);
                }

                let my_player_type = menu_state
                    .game_state
                    .get_player_by_id(my_id)
                    .map(|player| player.player_type);

                menu_state
                    .draw(&mut surface, &mut glyph_brush, &rect_tess, &mut rect_program, my_player_type)
                    .unwrap();

                if let Some(player) = menu_state.game_state.get_player_by_id(my_id) {
                    player_type = player.player_type;
                    break 'menuloop;
                }
            }
        };

        video_subsystem.text_input().stop();

        name = menu_state.name.clone();

        match player_type {
            libplen::player::PlayerType::Agent => {
                surface.sdl().mouse().set_relative_mouse_mode(true);
                let result = agent::gameloop(
                    &mut surface,
                    &mut event_pump,
                    &mut reader,
                    my_id,
                );
                surface.sdl().mouse().set_relative_mouse_mode(false);

                match result {
                    StateResult::Quit => break 'mainloop,
                    StateResult::Continue => continue,
                    StateResult::GotoNext => (),
                }
            }
            libplen::player::PlayerType::Dispatcher => {
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
                    dispatcher_state.update(&mut reader, &event_pump.keyboard_state());

                    // rendering::setup_coordinates(&mut canvas)?;
                    // canvas.set_draw_color(constants::MENU_BACKGROUND_COLOR);
                    // canvas.clear();

                    // dispatcher_state.draw(&mut canvas, &assets).unwrap();

                    // canvas.present();
                };

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
