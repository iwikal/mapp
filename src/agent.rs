use std::time::Instant;

use sdl2::EventPump;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{Keycode, Scancode};

use luminance::context::GraphicsContext as _;
use luminance::pipeline::PipelineState;
use luminance::shader::BuiltProgram;
use luminance::render_state::RenderState;
use luminance_derive::{Vertex, Semantics};
use luminance_glyph::{GlyphBrushBuilder, Section, Text};

use ultraviolet::{Vec4, Mat4, Vec2};

use libplen::messages::{ClientInput, ClientMessage, MessageReader, ServerMessage, SoundEffect};

use crate::{gamestate, map, assets::Assets, constants, surface, StateResult};

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
        crate::send_client_message(&input_message, &mut server_reader.stream);

        StateResult::Continue
    }
}

pub fn gameloop(
    sdl: sdl2::Sdl,
    event_pump: &mut EventPump,
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

    let sdl = surface.sdl();

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

    let agent_state = &mut AgentState::new(my_id);

    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Window { win_event: WindowEvent::Close, .. } |
                Event::Quit { .. } => {
                    let (sdl, ..) = surface.into_parts();
                    return (StateResult::Quit, sdl);
                }
                Event::Window { win_event: WindowEvent::SizeChanged(..), .. } => {
                    back_buffer = surface.back_buffer().unwrap();
                }
                _ => {}
            }
        }

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

                    // Start shading with our program.
                    shd_gate.shade(&mut program, |_, _, mut rdr_gate| {
                        // Start rendering things with the default render state provided by
                        // luminance.
                        rdr_gate.render(&RenderState::default(), |mut tess_gate| {
                            tess_gate.render(&direct_triangles)
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
