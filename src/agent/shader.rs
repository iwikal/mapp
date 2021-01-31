use luminance::context::GraphicsContext;
use luminance::shader::{BuiltProgram, Program, UniformInterface};
use luminance::vertex::Semantics;
use luminance_gl::GL33;
use crate::surface::Sdl2Surface;

pub fn compile_shader<Sem, Out, Uni>(
    surface: &mut Sdl2Surface,
    vertex_shader: &str,
    fragment_shader: &str,
) -> Program<GL33, Sem, Out, Uni>
where
    Sem: Semantics,
    Uni: UniformInterface<GL33>,
{
    let result = surface
        .new_shader_program()
        .from_strings(vertex_shader, None, None, fragment_shader);
    match result {
        Ok(BuiltProgram { program, warnings}) => {
            for warning in warnings {
                eprintln!("{}", warning);
            }

            program
        }
        Err(e) => {
            eprintln!("{}", e);
            panic!("failed to compile shader");
        }
    }
}
