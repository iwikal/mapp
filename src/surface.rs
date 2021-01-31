pub use luminance::context::GraphicsContext;
use luminance::framebuffer::Framebuffer;
use luminance::framebuffer::FramebufferError;
use luminance::texture::Dim2;
use luminance_gl::gl33::StateQueryError;
use luminance_gl::GL33;
use sdl2;
use std::fmt;
use std::os::raw::c_void;

/// Error that can be risen while creating a surface.
#[non_exhaustive]
#[derive(Debug)]
pub enum Sdl2SurfaceError {
    /// Window creation failed.
    WindowCreationFailed(sdl2::video::WindowBuildError),
    /// Failed to create an OpenGL context.
    GlContextInitFailed(String),
    /// No available video mode.
    VideoInitError(String),
    /// The graphics state is not available.
    ///
    /// This error is generated when the initialization code is called on a thread on which the
    /// graphics state has already been acquired.
    GraphicsStateError(StateQueryError),
}

impl fmt::Display for Sdl2SurfaceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Sdl2SurfaceError::WindowCreationFailed(ref e) => {
                write!(f, "failed to create window: {}", e)
            }
            Sdl2SurfaceError::GlContextInitFailed(ref e) => {
                write!(f, "failed to create OpenGL context: {}", e)
            }
            Sdl2SurfaceError::VideoInitError(ref e) => {
                write!(f, "failed to initialize video system: {}", e)
            }
            Sdl2SurfaceError::GraphicsStateError(ref e) => {
                write!(f, "failed to get graphics state: {}", e)
            }
        }
    }
}

/// A [luminance] GraphicsContext backed by SDL2 and OpenGL 3.3 Core.
///
/// ```ignore
/// use luminance_sdl2::Sdl2Surface;
///
/// let surface = Sdl2Surface::build_with(|video| video.window("My app", 800, 600))
///     .expect("failed to create surface");
///
/// let sdl = surface.sdl();
/// ```
///
/// [luminance]: https://crates.io/crates/luminance
pub struct Sdl2Surface {
    gl: GL33,
    // This struct needs to stay alive until we are done with OpenGL stuff.
    _gl_context: sdl2::video::GLContext,
}

impl Sdl2Surface {
    /// Create a new [`Sdl2Surface`] from a [`sdl2::video::WindowBuilder`].
    ///
    /// The callback is passed a reference to [`sdl2::VideoSubsystem`].
    /// This is your chance to change GL attributes before creating the window with your preferred
    /// settings.
    ///
    /// ```ignore
    /// use luminance_sdl2::Sdl2Surface;
    ///
    /// let surface = Sdl2Surface::build_with(|video| {
    ///     let gl_attr = video.gl_attr();
    ///     gl_attr.set_multisample_buffers(1);
    ///     gl_attr.set_multisample_samples(4);
    ///
    ///     let mut builder = video.window("My app", 800, 600);
    ///     builder.fullscreen_desktop();
    ///     builder
    /// })
    ///   .expect("failed to build window");
    /// ```
    pub fn build_with(
        window: &sdl2::video::Window,
        video_system: &sdl2::VideoSubsystem
    ) -> Result<Self, Sdl2SurfaceError>
    {
        let _gl_context = window
            .gl_create_context()
            .map_err(Sdl2SurfaceError::GlContextInitFailed)?;

        gl::load_with(|s| video_system.gl_get_proc_address(s) as *const c_void);

        let gl = GL33::new().map_err(Sdl2SurfaceError::GraphicsStateError)?;

        let surface = Sdl2Surface {
            gl,
            _gl_context,
        };

        Ok(surface)
    }
}

unsafe impl GraphicsContext for Sdl2Surface {
    type Backend = GL33;

    fn backend(&mut self) -> &mut Self::Backend {
        &mut self.gl
    }
}
