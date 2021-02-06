use luminance::context::GraphicsContext;
use luminance::depth_test::DepthWrite;
use luminance::face_culling::{FaceCulling, FaceCullingMode};
use luminance::pipeline::{Pipeline, PipelineError};
use luminance::render_state::RenderState;
use luminance::shader::{Program, Uniform};
use luminance::shading_gate::ShadingGate;
use luminance::tess::{Mode, Tess};
use luminance_derive::{Semantics, UniformInterface, Vertex};
use luminance_gl::GL33;

use ultraviolet::{Mat3, Mat4, Vec2, Vec3, Vec4};

use libplen::level;

use super::shader::compile_shader;
use super::surface::Sdl2Surface;
use crate::constants;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Semantics)]
pub enum WallSemantics {
    #[sem(name = "position", repr = "[f32; 3]", wrapper = "WallVertexPosition")]
    Position,
    #[sem(name = "uv", repr = "[f32; 2]", wrapper = "WallVertexUv")]
    Uv,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "WallSemantics")]
struct WallVertex {
    position: WallVertexPosition,
    uv: WallVertexUv,
}

#[derive(UniformInterface)]
pub struct WallInterface {
    pub model: Uniform<[[f32; 4]; 4]>,
    pub view: Uniform<[[f32; 4]; 4]>,
    pub projection: Uniform<[[f32; 4]; 4]>,
}

struct WallMaterial {
    shader: Program<GL33, WallSemantics, (), WallInterface>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Semantics)]
pub enum DoorSemantics {
    #[sem(name = "position", repr = "[f32; 3]", wrapper = "DoorVertexPosition")]
    Position,
}

#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "DoorSemantics")]
struct DoorVertex {
    position: DoorVertexPosition,
}

#[derive(UniformInterface)]
struct DoorInterface {
    pub model: Uniform<[[f32; 4]; 4]>,
    pub view: Uniform<[[f32; 4]; 4]>,
    pub projection: Uniform<[[f32; 4]; 4]>,
}

struct DoorMaterial {
    shader: Program<GL33, DoorSemantics, (), DoorInterface>,
}

pub struct RoomModel {
    wall_tess: Tess<GL33, WallVertex, u8>,
    wall_material: WallMaterial,
    door_tess: Tess<GL33, DoorVertex, u8>,
    door_material: DoorMaterial,
}

impl RoomModel {
    pub fn new(surface: &mut Sdl2Surface) -> Self {
        let wall_shader = compile_shader(
            surface,
            include_str!("../../shaders/wall.vert"),
            include_str!("../../shaders/wall.frag"),
        );

        let door_shader = compile_shader(
            surface,
            include_str!("../../shaders/door.vert"),
            include_str!("../../shaders/door.frag"),
        );

        Self {
            wall_tess: wall_tess(surface),
            door_tess: door_tess(surface),
            wall_material: WallMaterial {
                shader: wall_shader,
            },
            door_material: DoorMaterial {
                shader: door_shader,
            },
        }
    }

    pub fn draw<'r, I, J>(
        &mut self,
        pipeline: &mut Pipeline<GL33>,
        shd_gate: &mut ShadingGate<GL33>,
        view_mat: Mat4,
        projection_mat: Mat4,
        rooms: I,
    ) -> Result<(), PipelineError>
    where
        I: IntoIterator<Item = J>,
        J: IntoIterator<Item = &'r level::Room>,
    {
        unsafe {
            gl::Enable(gl::STENCIL_TEST);
            gl::StencilFunc(gl::EQUAL, 0, 0xFF);
            gl::StencilOp(gl::KEEP, gl::KEEP, gl::INCR);
        };

        for (column, rooms) in rooms.into_iter().enumerate() {
            for (row, room) in rooms.into_iter().enumerate() {
                match room {
                    crate::level::Room::FullRoom(doorways) => {
                        self.draw_one(
                            pipeline,
                            shd_gate,
                            view_mat,
                            projection_mat,
                            (column, row),
                            doorways,
                        )?;
                    }
                    _ => {
                        // TODO render hallways
                    }
                }
            }
        }

        unsafe {
            gl::StencilOp(gl::KEEP, gl::KEEP, gl::KEEP);
        }

        Ok(())
    }

    fn draw_one(
        &mut self,
        _pipeline: &mut Pipeline<GL33>,
        shd_gate: &mut ShadingGate<GL33>,
        view_mat: Mat4,
        projection_mat: Mat4,
        room_coord: (usize, usize),
        doors: &[(i8, i8)],
    ) -> Result<(), PipelineError> {
        let Self {
            wall_tess,
            door_tess,
            wall_material: WallMaterial {
                shader: wall_shader,
            },
            door_material: DoorMaterial {
                shader: door_shader,
            },
        } = self;

        let (column, row) = room_coord;
        let pos = crate::level::room_corner_position(column, row)
            + Vec2::new(constants::ROOM_WIDTH, constants::ROOM_LENGTH) * 0.5;
        let translation = Vec3::new(pos.x, 0., pos.y);
        let room_model_mat = Mat4::from_translation(translation);

        for &door_coord in doors {
            let (rotation, translation) = level::doorway_transform(room_coord, door_coord);
            let rotation = {
                let &[column_a, column_b] = rotation.as_component_array();
                Mat4::new(
                    Vec4::new(column_a.x, 0., column_a.y, 0.),
                    Vec4::new(0., 1., 0., 0.),
                    Vec4::new(column_b.x, 0., column_b.y, 0.),
                    Vec4::new(0., 0., 0., 1.),
                )
            };

            let translation = Vec3::new(translation.x, 0., translation.y);
            let door_model_mat = room_model_mat.translated(&translation) * rotation;

            shd_gate.shade(door_shader, |mut int, uni, mut rdr_gate| {
                int.set(&uni.model, door_model_mat.into());
                int.set(&uni.view, view_mat.into());
                int.set(&uni.projection, projection_mat.into());

                let render_state = RenderState::default()
                    .set_depth_write(DepthWrite::Off)
                    .set_face_culling(FaceCulling {
                        mode: FaceCullingMode::Back,
                        ..Default::default()
                    });
                rdr_gate.render(&render_state, |mut tess_gate| tess_gate.render(&*door_tess))
            })?;
        }

        shd_gate.shade(wall_shader, |mut int, uni, mut rdr_gate| {
            int.set(&uni.model, room_model_mat.into());
            int.set(&uni.view, view_mat.into());
            int.set(&uni.projection, projection_mat.into());

            let render_state = RenderState::default().set_face_culling(FaceCulling {
                mode: FaceCullingMode::Back,
                ..Default::default()
            });
            rdr_gate.render(&render_state, |mut tess_gate| tess_gate.render(&*wall_tess))
        })?;

        unsafe {
            gl::Clear(gl::STENCIL_BUFFER_BIT);
        }

        Ok(())
    }
}

fn wall_tess(surface: &mut impl GraphicsContext<Backend = GL33>) -> Tess<GL33, WallVertex, u8> {
    let mut vertices: Vec<WallVertex> = vec![];
    let mut indices: Vec<u8> = vec![];

    let rot_ninety_degrees = Mat3::new(
        Vec3::new(0., 0., 1.),
        Vec3::new(0., 1., 0.),
        Vec3::new(-1., 0., 0.),
    );

    let mut rot_matrix = Mat3::identity();
    for _ in 0..4 {
        let index = vertices.len() as u8;

        for x in 0..2 {
            for y in 0..2 {
                let (x, y) = (x as f32, y as f32);

                let pos = rot_matrix
                    * (Vec3::new(x, y, 1.) - Vec3::new(0.5, 0., 0.5))
                    * Vec3::new(
                        constants::ROOM_WIDTH,
                        constants::CEILING_HEIGHT,
                        constants::ROOM_LENGTH,
                    );

                let uv = Vec2::new(pos.x + pos.z, pos.y);

                vertices.push(WallVertex {
                    position: WallVertexPosition::new(pos.into()),
                    uv: WallVertexUv::new(uv.into()),
                });
            }
        }

        indices.push(index + 0);
        indices.push(index + 1);
        indices.push(index + 2);
        indices.push(index + 3);
        indices.push(index + 2);
        indices.push(index + 1);

        rot_matrix = rot_matrix * rot_ninety_degrees;
    }

    surface
        .new_tess()
        .set_mode(Mode::Triangle)
        .set_vertices(vertices)
        .set_indices(indices)
        .build()
        .unwrap()
}

fn door_tess(surface: &mut Sdl2Surface) -> Tess<GL33, DoorVertex, u8> {
    let mut vertices = vec![];

    for x in 0..2 {
        for y in 0..2 {
            let x = (x * 2 - 1) as f32 * constants::DOOR_WIDTH / 2.;
            let y = y as f32 * constants::DOOR_HEIGHT;
            vertices.push(DoorVertex {
                position: DoorVertexPosition::new([x, y, 0.]),
            });
        }
    }

    let indices = vec![0, 1, 2, 3, 2, 1];

    surface
        .new_tess()
        .set_mode(Mode::Triangle)
        .set_vertices(vertices)
        .set_indices(indices)
        .build()
        .unwrap()
}
