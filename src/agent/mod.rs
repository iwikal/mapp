mod room;
mod shader;
mod sprite;

use std::time::Instant;

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Scancode;
use sdl2::EventPump;

use luminance::blending::{Blending, Equation, Factor};
use luminance::context::GraphicsContext;
use luminance::pipeline::PipelineState;
use luminance::render_state::RenderState;
use luminance::shader::BuiltProgram;
use luminance_derive::{Semantics, Vertex};
use luminance_glyph::{GlyphBrushBuilder, Section, Text};

use ultraviolet::{Mat4, Vec3};

use libplen::level::{self, Level};
use libplen::messages::{ClientInput, ClientMessage, MessageReader, ServerMessage, SoundEffect};
use libplen::player;

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
            map: map::Map::new(level::example_level()),
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

    fn myself(&self) -> &player::Player {
        let Self {
            my_id, game_state, ..
        } = self;
        game_state.get_player_by_id(*my_id).unwrap()
    }
}

pub fn gameloop(
    sdl: sdl2::Sdl,
    event_pump: &mut EventPump,
    server_reader: &mut MessageReader,
    sounds: &SoundAssets,
    my_id: u64,
) -> (StateResult, sdl2::Sdl) {
    sdl.mouse().set_relative_mouse_mode(true);

    let mut surface = surface::Sdl2Surface::build_with(sdl, |video| {
        let mut wb = video.window(
            "MAPP",
            constants::WINDOW_SIZE as u32,
            constants::WINDOW_SIZE as u32,
        );
        wb.fullscreen_desktop();
        wb.resizable();
        wb
    })
    .expect("Could not create rendering surface");

    let mut back_buffer = surface.back_buffer().expect("Could not get back buffer");

    let mut sprite_program = {
        let vs = include_str!("../../shaders/sprite.vert");
        let fs = include_str!("../../shaders/sprite.frag");
        shader::compile_shader::<(), (), sprite::SpriteInterface>(&mut surface, vs, fs)
    };

    let mut room_model = room::RoomModel::new(&mut surface);

    let sprite_tess = surface
        .new_tess()
        .set_vertex_nb(4)
        .set_mode(luminance::tess::Mode::TriangleFan)
        .build()
        .unwrap();

    let mut glyph_brush = {
        let ttf = include_bytes!("../../resources/yoster.ttf");
        let font = ab_glyph::FontArc::try_from_slice(ttf).expect("Could not load font");
        GlyphBrushBuilder::using_font(font).build(&mut surface)
    };

    let agent_state = &mut AgentState::new(my_id);

    fn make_projection_matrix(surface: &surface::Sdl2Surface) -> Mat4 {
        let (width, height) = surface.window().size();
        let aspect_ratio = width as f32 / height as f32;
        let fov = 60_f32.to_radians();
        ultraviolet::projection::perspective_gl(fov, aspect_ratio, 0.01, 100.0)
    }

    let mut projection = make_projection_matrix(&surface);
    let mut resize = false;

    let mut flower_sprite = sprite::load_sprite(&mut surface, "resources/flower.png");

    let mut rotation = 0_f32;

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
                Event::MouseMotion { xrel, .. } => {
                    rotation -= xrel as f32 * 0.001;
                }
                _ => {}
            }
        }

        if resize {
            back_buffer = surface.back_buffer().unwrap();
            projection = make_projection_matrix(&surface);
            resize = false;
        }

        agent_state.update(sounds, server_reader, &event_pump.keyboard_state());
        glyph_brush.process_queued(&mut surface);

        // let myself = agent_state.myself();

        let my_pos = Vec3::new(0., 0., 1.); // FIXME
        let view = Mat4::from_rotation_y(rotation) * Mat4::from_translation(-my_pos);

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

                    room_model.draw(
                        &mut pipeline,
                        &mut shd_gate,
                        Mat4::identity(),
                        view,
                        projection,
                    )?;

                    let bound_tex = pipeline.bind_texture(&mut flower_sprite)?;

                    // Start shading with our program.
                    shd_gate.shade(&mut sprite_program, |mut iface, uni, mut rdr_gate| {
                        iface.set(&uni.tex, bound_tex.binding());
                        iface.set(&uni.view, view.into());
                        iface.set(&uni.projection, projection.into());

                        // Start rendering things with the default render state provided by
                        // luminance.
                        let render_state = RenderState::default()
                            .set_blending(Blending {
                                equation: Equation::Additive,
                                src: Factor::SrcAlpha,
                                dst: Factor::SrcAlphaComplement,
                            });
                        rdr_gate.render(&render_state, |mut tess_gate| {
                            tess_gate.render(&sprite_tess)
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
