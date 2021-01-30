mod assets;
mod map;
mod menu;
mod rendering;
mod surface;

use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Instant;

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::render::BlendMode;
use sdl2::render::Canvas;
use sdl2::video::Window;

use luminance::context::GraphicsContext as _;
use luminance::pipeline::PipelineState;
use luminance::shader::BuiltProgram;
use luminance::render_state::RenderState;

use luminance_derive::{Vertex, Semantics};

use luminance_glyph::{GlyphBrushBuilder, Section, Text};

use ultraviolet::{Vec4, Mat4};

use assets::Assets;
use libplen::constants;
use libplen::gamestate;
use libplen::math::{vec2, Vec2};
use libplen::messages::{ClientInput, ClientMessage, MessageReader, ServerMessage, SoundEffect};
use menu::MenuState;

fn send_client_message(msg: &ClientMessage, stream: &mut TcpStream) {
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
enum StateResult {
    Continue,
    GotoNext,
}

struct MainState {
    my_id: u64,
    game_state: gamestate::GameState,
    map: map::Map,
    last_time: Instant,
}

impl MainState {
    fn new(my_id: u64) -> MainState {
        MainState {
            my_id,
            game_state: gamestate::GameState::new(),
            map: map::Map::new(),
            last_time: Instant::now(),
        }
    }

    fn update(
        &mut self,
        assets: &Assets,
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
                ServerMessage::PlaySound(sound, pos) => {
                    fn play_sound(soundeffect: &sdl2::mixer::Chunk) {
                        if let Err(e) = sdl2::mixer::Channel::all().play(soundeffect, 0) {
                            println!("SDL mixer error: {}", e);
                        }
                    }

                    match sound {
                        SoundEffect::Powerup => {
                            play_sound(&assets.powerup);
                        }
                        SoundEffect::Gun => {
                            play_sound(&assets.gun);
                        }
                        SoundEffect::Explosion => {
                            play_sound(&assets.explosion);
                        }
                        SoundEffect::LaserCharge => {
                            play_sound(&assets.laser_charge_sound);
                        }
                        SoundEffect::LaserFire => {
                            play_sound(&assets.laser_fire_sound);
                        }
                    }
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
        send_client_message(&input_message, &mut server_reader.stream);

        StateResult::Continue
    }

    fn draw(&mut self, canvas: &mut Canvas<Window>, assets: &mut Assets) -> Result<(), String> {
        self.map.draw(self.my_id, canvas)?;

        for player in &self.game_state.players {
            let w = 10;
            let h = 10;

            let dest_rect = sdl2::rect::Rect::new(
                player.position.x as i32 - w as i32 / 2,
                player.position.y as i32 - h as i32 / 2,
                w as u32,
                h as u32,
            );
            canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 25, 25));

            canvas.fill_rect(dest_rect)?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Semantics)]
pub enum Semantics {
    #[sem(name = "co", repr = "[f32; 3]", wrapper = "VertexPosition")]
    Position,
    #[sem(name = "color", repr = "[u8; 3]", wrapper = "VertexColor")]
    Color,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics")]
struct Vertex {
    pos: VertexPosition,
    #[vertex(normalized = "true")]
    rgb: VertexColor,
}

// The vertices. We define two triangles.
const TRI_VERTICES: [Vertex; 6] = [
    // First triangle – an RGB one.
    Vertex::new(
        VertexPosition::new([0.5, -0.5, 0.]),
        VertexColor::new([0, 255, 0]),
    ),
    Vertex::new(
        VertexPosition::new([0.0, 0.5, 0.]),
        VertexColor::new([0, 0, 255]),
    ),
    Vertex::new(
        VertexPosition::new([-0.5, -0.5, 0.]),
        VertexColor::new([255, 0, 0]),
    ),
    // Second triangle, a purple one, positioned differently.
    Vertex::new(
        VertexPosition::new([-0.5, 0.5, 0.]),
        VertexColor::new([255, 51, 255]),
    ),
    Vertex::new(
        VertexPosition::new([0.0, -0.5, 0.]),
        VertexColor::new([51, 255, 255]),
    ),
    Vertex::new(
        VertexPosition::new([0.5, 0.5, 0.]),
        VertexColor::new([51, 51, 255]),
    ),
];

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

    let mut surface = surface::Sdl2Surface::build_with(|video| {
        let mut wb = video.window(
            "very nice gem",
            constants::WINDOW_SIZE as u32,
            constants::WINDOW_SIZE as u32,
        );
        wb.resizable();
        wb
    })
    .expect("Could not create rendering surface");

    let sdl = surface.sdl();
    let video_subsystem = sdl.video().unwrap();

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

    let mut name = whoami::username();

    let mut event_pump = sdl.event_pump().expect("Could not get event pump");
    let mut back_buffer = surface.back_buffer().expect("Could not get back buffer");

    let mut program = {
        let vs = include_str!("../shaders/triangle.vert");
        let fs = include_str!("../shaders/triangle.frag");
        let BuiltProgram { program, warnings } = surface
            .new_shader_program::<Semantics, (), ()>()
            .from_strings(vs, None, None, fs)
            .expect("Failed to compile shaders");

        for warning in warnings {
            eprintln!("{}", warning);
        }

        program
    };

    // Create tessellation for direct geometry; that is, tessellation that will render vertices by
    // taking one after another in the provided slice.
    let direct_triangles = surface
        .new_tess()
        .set_vertices(&TRI_VERTICES[..])
        .set_mode(luminance::tess::Mode::Triangle)
        .build()
        .unwrap();

    let mut glyph_brush = {
        let ttf = include_bytes!("../resources/yoster.ttf");
        let font = ab_glyph::FontArc::try_from_slice(ttf).expect("Could not load font");
        GlyphBrushBuilder::using_font(font).build(&mut surface)
    };

    glyph_brush.queue(
        Section::default().add_text(
            Text::new("Font test")
            .with_color([1.0, 1.0, 1.0, 1.0])
            .with_scale(80.0),
        ),
    );

    glyph_brush.process_queued(&mut surface);

    'mainloop: loop {
        let menu_state = &mut MenuState::new();

        video_subsystem.text_input().start();
        menu_state.name = name;

        'menuloop: loop {
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
                    Event::Window { win_event: WindowEvent::SizeChanged(..), .. } => {
                        back_buffer = surface.back_buffer().expect("Could not get back buffer");
                    }
                    _ => {}
                }
            }
            // rendering::setup_coordinates(&mut canvas)?;

            // Create a new dynamic pipeline that will render to the back buffer and must clear it
            // with pitch black prior to do any render to it.
            let render = surface
                .new_pipeline_gate()
                .pipeline(
                    &back_buffer,
                    &PipelineState::default(),
                    |mut pipeline, mut shd_gate| {
                        // Start shading with our program.
                        shd_gate.shade(&mut program, |_, _, mut rdr_gate| {
                            // Start rendering things with the default render state provided by
                            // luminance.
                            rdr_gate.render(&RenderState::default(), |mut tess_gate| {
                                // Pick the right tessellation to use depending on the mode chosen
                                // and render it to the surface.
                                tess_gate.render(&direct_triangles)
                            })
                        })?;

                        glyph_brush.draw_queued(&mut pipeline, &mut shd_gate, 1024, 720)?;
                        Ok(())
                    },
                )
                .assume();

            surface.window().gl_swap_window();

            // Ignore all messages so we don't freeze the server
            reader.fetch_bytes().unwrap();
            for _ in reader.iter() {}

            menu_state.update();
        }
        video_subsystem.text_input().stop();

        name = menu_state.name.clone();

        send_client_message(
            &ClientMessage::JoinGame {
                name: menu_state.name.clone(),
            },
            &mut reader.stream,
        );

        let main_state = &mut MainState::new(my_id);
        'gameloop: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'mainloop,
                    _ => {}
                }
            }

            /*
            rendering::setup_coordinates(&mut canvas)?;

            canvas.set_draw_color(sdl2::pixels::Color::RGB(25, 25, 25));
            canvas.clear();

            let state_result =
                main_state.update(&assets, &mut reader, &event_pump.keyboard_state());
            main_state.draw(&mut canvas, &mut assets).unwrap();

            canvas.present();


            if state_result == StateResult::GotoNext {
                break 'gameloop;
            }
            */
        }
    }

    Ok(())
}
