mod sprite;

use std::time::Instant;

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Scancode;
use sdl2::EventPump;

use luminance::context::GraphicsContext as _;
use luminance::pipeline::PipelineState;
use luminance::render_state::RenderState;
use luminance::shader::BuiltProgram;
use luminance_derive::{Semantics, Vertex};
use luminance_glyph::{GlyphBrushBuilder, Section, Text};

use ultraviolet::Mat4;

use libplen::messages::{ClientInput, ClientMessage, MessageReader, ServerMessage, SoundEffect};

use crate::{assets::SoundAssets, constants, gamestate, map, surface, StateResult};

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

struct AgentState {
    my_id: u64,
    game_state: gamestate::GameState,
    map: map::Map,
    last_time: Instant,
}

impl AgentState {
    fn new(my_id: u64) -> AgentState {
        AgentState {
            my_id,
            game_state: gamestate::GameState::new(),
            map: map::Map::new(),
            last_time: Instant::now(),
        }
    }

    fn update(
        &mut self,
        sounds: &SoundAssets,
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
                        SoundEffect::Powerup => play_sound(&sounds.powerup),
                        SoundEffect::Gun => play_sound(&sounds.gun),
                        SoundEffect::Explosion => play_sound(&sounds.explosion),
                        SoundEffect::LaserCharge => play_sound(&sounds.laser_charge_sound),
                        SoundEffect::LaserFire => play_sound(&sounds.laser_fire_sound),
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
        crate::send_client_message(&input_message, &mut server_reader.stream);

        StateResult::Continue
    }
}

pub fn gameloop(
    sdl: sdl2::Sdl,
    event_pump: &mut EventPump,
    server_reader: &mut MessageReader,
    sounds: &SoundAssets,
    my_id: u64,
) -> (StateResult, sdl2::Sdl) {
    let mut surface = surface::Sdl2Surface::build_with(sdl, |video| {
        let mut wb = video.window(
            "very nice gem",
            constants::WINDOW_SIZE as u32,
            constants::WINDOW_SIZE as u32,
        );
        wb.resizable();
        wb
    })
    .expect("Could not create rendering surface");

    let mut back_buffer = surface.back_buffer().expect("Could not get back buffer");

    let mut sprite_program = {
        let vs = include_str!("../shaders/quad.vert");
        let fs = include_str!("../shaders/quad.frag");
        let BuiltProgram { program, warnings } = surface
            .new_shader_program::<(), (), sprite::SpriteInterface>()
            .from_strings(vs, None, None, fs)
            .expect("Failed to compile shaders");

        for warning in warnings {
            eprintln!("{}", warning);
        }

        program
    };

    let quad_tess = surface
        .new_tess()
        .set_vertex_nb(4)
        .set_mode(luminance::tess::Mode::TriangleFan)
        .build()
        .unwrap();

    let mut glyph_brush = {
        let ttf = include_bytes!("../resources/yoster.ttf");
        let font = ab_glyph::FontArc::try_from_slice(ttf).expect("Could not load font");
        GlyphBrushBuilder::using_font(font).build(&mut surface)
    };

    let agent_state = &mut AgentState::new(my_id);

    let (width, height) = surface.window().size();

    fn make_projection_matrix(width: f32, height: f32) -> Mat4 {
        let fov = 90_f32.to_radians();
        let aspect_ratio = width / height;
        ultraviolet::projection::perspective_gl(fov, aspect_ratio, 0.01, 100.0)
    }

    let mut projection = make_projection_matrix(width as _, height as _);
    let mut resize = false;

    let mut flower_sprite = sprite::load_sprite(&mut surface, "resources/flower.png");

    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Window {
                    win_event: WindowEvent::Close,
                    ..
                }
                | Event::Quit { .. } => {
                    let (sdl, ..) = surface.into_parts();
                    return (StateResult::Quit, sdl);
                }
                Event::Window {
                    win_event: WindowEvent::SizeChanged(..),
                    ..
                } => resize = true,
                _ => {}
            }
        }

        if resize {
            back_buffer = surface.back_buffer().unwrap();
            resize = false;
        }

        agent_state.update(sounds, server_reader, &event_pump.keyboard_state());
        glyph_brush.process_queued(&mut surface);

        // Create a new dynamic pipeline that will render to the back buffer and must clear it
        // with pitch black prior to do any render to it.
        surface
            .new_pipeline_gate()
            .pipeline(
                &back_buffer,
                &PipelineState::default(),
                |mut pipeline, mut shd_gate| {
                    // Draw text.
                    glyph_brush.draw_queued(&mut pipeline, &mut shd_gate, 1024, 720)?;

                    let bound_tex = pipeline.bind_texture(&mut flower_sprite)?;

                    // Start shading with our program.
                    shd_gate.shade(&mut sprite_program, |mut iface, uni, mut rdr_gate| {
                        iface.set(&uni.tex, bound_tex.binding());

                        // Start rendering things with the default render state provided by
                        // luminance.
                        rdr_gate.render(&RenderState::default(), |mut tess_gate| {
                            tess_gate.render(&quad_tess)
                        })
                    })?;

                    Ok(())
                },
            )
            .assume()
            .into_result()
            .expect("Failed to render");

        surface.window().gl_swap_window();
    }
}
