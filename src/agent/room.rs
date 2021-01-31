use luminance::context::GraphicsContext;
use luminance::face_culling::{FaceCulling, FaceCullingMode};
use luminance::pipeline::{Pipeline, PipelineError};
use luminance::render_state::RenderState;
use luminance::shader::{Program, Uniform};
use luminance::shading_gate::ShadingGate;
use luminance::tess::{Mode, Tess};
use luminance_derive::{Semantics, UniformInterface, Vertex};
use luminance_gl::GL33;

use ultraviolet::{Mat3, Mat4, Vec3};

use super::shader::compile_shader;
use super::surface::Sdl2Surface;
use crate::constants;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Semantics)]
pub enum WallSemantics {
    #[sem(name = "position", repr = "[f32; 3]", wrapper = "VertexPosition")]
    Position,
    #[sem(name = "uv", repr = "[f32; 2]", wrapper = "VertexUv")]
    Uv,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "WallSemantics")]
struct WallVertex {
    position: VertexPosition,
    uv: VertexUv,
}

#[derive(UniformInterface)]
pub struct WallInterface {
    pub model: Uniform<[[f32; 4]; 4]>,
    pub view: Uniform<[[f32; 4]; 4]>,
    pub projection: Uniform<[[f32; 4]; 4]>,
}

struct Material {
    shader: Program<GL33, WallSemantics, (), WallInterface>,
}

pub struct RoomModel {
    wall_tess: Tess<GL33, WallVertex, u8>,
    material: Material,
}

impl RoomModel {
    pub fn new(surface: &mut Sdl2Surface) -> Self {
        let shader = compile_shader(
            surface,
            include_str!("../../shaders/wall.vert"),
            include_str!("../../shaders/wall.frag"),
        );

        Self {
            wall_tess: wall_tess(surface),
            material: Material { shader },
        }
    }

    pub fn draw(
        &mut self,
        _pipeline: &mut Pipeline<GL33>,
        shd_gate: &mut ShadingGate<GL33>,
        model_mat: Mat4,
        view_mat: Mat4,
        projection_mat: Mat4,
    ) -> Result<(), PipelineError> {
        let Self {
            wall_tess,
            material: Material { shader },
        } = self;

        shd_gate.shade(shader, |mut int, uni, mut rdr_gate| {
            int.set(&uni.model, model_mat.into());
            int.set(&uni.view, view_mat.into());
            int.set(&uni.projection, projection_mat.into());

            let render_state = RenderState::default().set_face_culling(FaceCulling {
                mode: FaceCullingMode::Back,
                ..Default::default()
            });
            rdr_gate.render(&render_state, |mut tess_gate| tess_gate.render(&*wall_tess))
        })
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
                let pos = rot_matrix
                    * (Vec3::new(x as f32, y as f32, 1.) - Vec3::broadcast(0.5))
                    * Vec3::new(
                        constants::ROOM_WIDTH,
                        constants::CEILING_HEIGHT,
                        constants::ROOM_LENGTH,
                    );

                vertices.push(WallVertex {
                    position: VertexPosition::new(pos.into()),
                    uv: VertexUv::new([0., 0.]),
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
